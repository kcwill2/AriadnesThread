#!/usr/bin/env python3
"""Runtime smoke for the `tool ui` server (incident E3: no hand-written probe again).

Boots the real UI server against a disposable copy of tests/fixtures/basic,
polls /api/status until progress.state == "ready", then asserts the real
static route table (index + styles.css + every /app/*.js the page loads)
answers 200 with the product's verified content types and Cache-Control.

Usage: python tools/ui_smoke.py [--fixture PATH] [--no-build] [--cold|--warm]
                                [--timeout SECONDS] [--exe PATH]
                                [--target-dir DIR] [--release]
Exit codes: 0 PASS, 1 smoke failure, 2 internal/usage error.

--release is opt-in (M28 T3 rider): it builds `cargo build --release
--bin tool` and launches target/release/tool(.exe) with the same cold/warm
routes and route + Cache-Control assertions. Without it the behavior is
exactly the debug-profile default (cargo build --bin tool, target/debug).

Verified product contract (2026-10-01, read-only inspection):
- crates/cli/src/lib.rs Ui command: positional `repo`, `--port`, `--no-open`,
  `--no-watch`.
- crates/ui/src/server.rs StatusResponse: readiness is the nested field
  `progress.state` (dto::ServerProgress), value "ready" when booted.
- crates/ui/src/server.rs build_router + ui_asset: every static route
  (`/`, `/styles.css`, `/app/*.js`) sets `Cache-Control: no-store`; the
  JSON API routes do not. Because the product DOES set no-store on the
  asset routes, a missing header there is a product regression, so the
  smoke FAILS on it rather than warning.
"""

import argparse
import json
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.request
from collections import deque
from pathlib import Path
from typing import Optional

EXIT_PASS = 0
EXIT_SMOKE_FAILURE = 1
EXIT_INTERNAL_ERROR = 2

READY_PROGRESS_STATE = "ready"
DEFAULT_TIMEOUT_SECONDS = 60.0
POLL_INTERVAL_SECONDS = 0.35
HTTP_POLL_TIMEOUT_SECONDS = 2.0
MAX_PORT_ATTEMPTS = 3
TAIL_LINES_KEPT = 40
COLD_STRIPPED_DIR_NAMES = ("artifacts", ".ariadne")

# Route table copied from build_router() in crates/ui/src/server.rs and the
# script tags in crates/ui/assets/index.html: the page loads /styles.css and
# six /app/*.js scripts, so the probe list must mirror exactly these routes.
ROUTE_ASSERTIONS = (
    ("/", "text/html", True),
    ("/styles.css", "text/css", True),
    ("/app/api.js", "text/javascript", True),
    ("/app/graph.js", "text/javascript", True),
    ("/app/file-view.js", "text/javascript", True),
    ("/app/hotspots.js", "text/javascript", True),
    ("/app/preview.js", "text/javascript", True),
    ("/app/main.js", "text/javascript", True),
    ("/icon.svg", "image/svg+xml", True),
)


class UsageError(Exception):
    """Bad command line: exit 2, mirroring hygiene_check.py's contract."""


class _StrictArgumentParser(argparse.ArgumentParser):
    def error(self, message: str):
        raise UsageError(message)


def parse_arguments(argv):
    parser = _StrictArgumentParser(
        prog="ui_smoke.py",
        description="Runtime smoke for the tool ui server.",
    )
    parser.add_argument("--fixture", default=None,
                        help="fixture repo to copy (default tests/fixtures/basic)")
    parser.add_argument("--no-build", action="store_true",
                        help="skip cargo build and reuse the existing binary")
    parser.add_argument("--release", action="store_true",
                        help="build and launch the release profile "
                             "(cargo build --release, target/release)")
    temperature = parser.add_mutually_exclusive_group()
    temperature.add_argument("--cold", action="store_true",
                             help="strip artifacts/ and .ariadne from the copy (default)")
    temperature.add_argument("--warm", action="store_true",
                             help="keep prebuilt artifacts in the copy (cache-hit path)")
    parser.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT_SECONDS,
                        help="seconds to wait for the server to reach ready state")
    parser.add_argument("--exe", default=None,
                        help="explicit path to the tool binary (overrides --target-dir)")
    parser.add_argument("--target-dir", default=None,
                        help="directory holding tool.exe (default target/debug,"
                             " or target/release with --release)")
    options = parser.parse_args(argv)
    if options.timeout <= 0:
        raise UsageError("--timeout must be a positive number of seconds")
    return options


def probe_free_port() -> int:
    """Ask the OS for a free loopback port via ephemeral bind-then-close.

    The CLI requires an explicit fixed --port, so there is an unavoidable
    TOCTOU window between closing the probe socket and the server binding.
    The caller retries the whole spawn with a fresh port up to
    MAX_PORT_ATTEMPTS times, which absorbs a squatter winning the race.
    """
    probe_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        probe_socket.bind(("127.0.0.1", 0))
        return probe_socket.getsockname()[1]
    finally:
        probe_socket.close()


def copy_fixture_into_copy(fixture_path: Path, copy_root: Path) -> None:
    shutil.copytree(str(fixture_path), str(copy_root), symlinks=False)


def strip_cache_directories(copy_root: Path):
    removed = []
    for dir_name in COLD_STRIPPED_DIR_NAMES:
        stale_path = copy_root / dir_name
        if stale_path.is_dir():
            shutil.rmtree(str(stale_path))
            removed.append(dir_name)
    return removed


def verify_cold_copy_clean(copy_root: Path) -> Optional[str]:
    """Post-strip guard: a --cold copy must not carry any cache directory.

    Returning a reason here fails the smoke loudly (plan validation #3):
    if strip silently no-oped, the run would measure the cache-hit path
    while claiming cold coverage.
    """
    residuals = [dir_name for dir_name in COLD_STRIPPED_DIR_NAMES
                 if (copy_root / dir_name).exists()]
    if residuals:
        return ("cold copy still carries cache dir(s): "
                + ", ".join(residuals) + " under " + str(copy_root))
    return None


class StreamTail:
    """Keeps the last N lines of a pipe on a daemon reader thread."""

    def __init__(self, max_lines: int = TAIL_LINES_KEPT):
        self._lines = deque(maxlen=max_lines)
        self._reader_thread = None

    def _pump(self, stream):
        try:
            for line in stream:
                self._lines.append(line.rstrip())
        except (ValueError, OSError):
            pass  # stream closed during child teardown
        finally:
            try:
                stream.close()
            except OSError:
                pass

    def start(self, stream) -> None:
        self._reader_thread = threading.Thread(
            target=self._pump, args=(stream,), daemon=True)
        self._reader_thread.start()

    def text(self) -> str:
        return "\n".join(self._lines)


def spawn_ui_server(exe_path: Path, repo_copy: Path, port: int, working_dir: Path):
    argv = [str(exe_path), "ui", str(repo_copy),
            "--no-open", "--no-watch", "--port", str(port)]
    child = subprocess.Popen(
        argv, cwd=str(working_dir),
        stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        text=True, errors="replace", bufsize=1)
    stdout_tail = StreamTail()
    stderr_tail = StreamTail()
    stdout_tail.start(child.stdout)
    stderr_tail.start(child.stderr)
    return child, stdout_tail, stderr_tail


def terminate_process(child, grace_seconds: float = 10.0) -> None:
    if child.poll() is not None:
        return
    child.terminate()
    try:
        child.wait(timeout=grace_seconds)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait(timeout=grace_seconds)


def fetch_status(base_url: str) -> Optional[dict]:
    """One GET /api/status; any transport/parse failure reads as not-ready."""
    try:
        with urllib.request.urlopen(base_url + "/api/status",
                                    timeout=HTTP_POLL_TIMEOUT_SECONDS) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as http_error:
        http_error.close()
        return None
    except (urllib.error.URLError, OSError, ValueError):
        return None


def extract_progress_state(payload) -> Optional[str]:
    """Readiness path verified on the product: StatusResponse.progress.state."""
    if not isinstance(payload, dict):
        return None
    progress = payload.get("progress")
    if not isinstance(progress, dict):
        return None
    state = progress.get("state")
    return state if isinstance(state, str) else None


def poll_until_ready(base_url: str, timeout_seconds: float,
                     poll_interval: float = POLL_INTERVAL_SECONDS,
                     is_server_running=None) -> bool:
    deadline = time.monotonic() + timeout_seconds
    while True:
        if is_server_running is not None and not is_server_running():
            return False
        state = extract_progress_state(fetch_status(base_url))
        if state == READY_PROGRESS_STATE:
            return True
        if time.monotonic() >= deadline:
            return False
        time.sleep(poll_interval)


class RouteResponse:
    def __init__(self, status, content_type, cache_control, body):
        self.status = status
        self.content_type = content_type
        self.cache_control = cache_control
        self.body = body


def fetch_route(base_url: str, route: str) -> RouteResponse:
    try:
        with urllib.request.urlopen(base_url + route,
                                    timeout=HTTP_POLL_TIMEOUT_SECONDS) as response:
            return RouteResponse(
                response.status,
                response.headers.get("Content-Type", ""),
                response.headers.get("Cache-Control", ""),
                response.read())
    except urllib.error.HTTPError as http_error:
        try:
            body = http_error.read()
        finally:
            http_error.close()
        return RouteResponse(
            http_error.code,
            http_error.headers.get("Content-Type", ""),
            http_error.headers.get("Cache-Control", ""),
            body)


def check_routes(base_url: str):
    """Assert every static route against the product behavior verified in server.rs.

    no-store is required wherever expect_no_store is set. server.rs ui_asset()
    attaches no-store to EVERY static asset route, so a missing header is a
    regression worth failing on; if that ever changes in the product, this is
    where to relax an asset row to a WARN instead.
    """
    failures = []
    for route, expected_type, expect_no_store in ROUTE_ASSERTIONS:
        try:
            response = fetch_route(base_url, route)
        except (urllib.error.URLError, OSError) as io_error:
            failures.append("GET " + route + ": " + str(io_error))
            continue
        if response.status != 200:
            failures.append("GET " + route + ": status " + str(response.status))
        if not response.content_type.lower().startswith(expected_type):
            failures.append("GET " + route + ": content-type "
                            + repr(response.content_type)
                            + " does not start with " + expected_type)
        if expect_no_store and response.cache_control.strip().lower() != "no-store":
            failures.append("GET " + route + ": Cache-Control "
                            + repr(response.cache_control)
                            + " is not no-store (product sets it on all static routes)")
        elif not response.body:
            failures.append("GET " + route + ": empty body")
    return failures


def cargo_build_argv(release: bool):
    """Cargo argv for the smoke build; default profile stays exactly debug."""
    argv = ["cargo", "build"]
    if release:
        argv.append("--release")
    argv += ["--bin", "tool"]
    return argv


def build_binary(repo_root: Path, release: bool = False):
    finished = subprocess.run(
        cargo_build_argv(release), cwd=str(repo_root),
        capture_output=True, text=True, errors="replace")
    output = (finished.stdout or "") + (finished.stderr or "")
    return finished.returncode == 0, output


def resolve_exe_path(options, repo_root: Path) -> Path:
    if options.exe is not None:
        return Path(options.exe).resolve()
    if options.target_dir is not None:
        target_dir = Path(options.target_dir)
    elif options.release:
        target_dir = Path("target/release")
    else:
        target_dir = Path("target/debug")
    if not target_dir.is_absolute():
        target_dir = repo_root / target_dir
    exe_name = "tool.exe" if sys.platform == "win32" else "tool"
    return target_dir / exe_name


def _describe_attempt_failure(child, stdout_tail, stderr_tail, timeout_seconds):
    if child.poll() is not None:
        head = "server exited early with code " + str(child.returncode)
    else:
        head = ("server did not reach ready state within "
                + str(timeout_seconds) + "s")
    tails = ("stdout tail:\n" + stdout_tail.text()
             + "\nstderr tail:\n" + stderr_tail.text())
    return head + "\n" + tails


class AttemptResult:
    """Outcome of one copy -> spawn -> poll -> route-check cycle.

    retryable is False for deterministic failures (cold guard) where
    re-spawning on another port cannot help.
    """

    def __init__(self, temp_root: Path, ready: bool, failures, detail: str,
                 retryable: bool):
        self.temp_root = temp_root
        self.ready = ready
        self.failures = failures
        self.detail = detail
        self.retryable = retryable


def _smoke_attempt(options, repo_root: Path, exe_path: Path) -> AttemptResult:
    """One full attempt: copy -> (cold strip) -> spawn -> poll -> route checks.

    Always terminates the child; the caller owns temp-dir cleanup.
    """
    temp_root = Path(tempfile.mkdtemp(prefix="ariadne-ui-smoke-"))
    repo_copy = temp_root / "repo"
    child = None
    try:
        copy_fixture_into_copy(options.fixture_path, repo_copy)
        if options.cold:
            strip_cache_directories(repo_copy)
            guard_reason = verify_cold_copy_clean(repo_copy)
            if guard_reason is not None:
                return AttemptResult(temp_root, False, [guard_reason],
                                     "cold guard tripped", retryable=False)
        port = probe_free_port()
        base_url = "http://127.0.0.1:" + str(port)
        child, stdout_tail, stderr_tail = spawn_ui_server(
            exe_path, repo_copy, port, repo_root)
        ready = poll_until_ready(
            base_url, options.timeout,
            is_server_running=lambda: child.poll() is None)
        if not ready:
            return AttemptResult(
                temp_root, False,
                [_describe_attempt_failure(
                    child, stdout_tail, stderr_tail, options.timeout)],
                "poll timeout/early exit", retryable=True)
        failures = check_routes(base_url)
        detail = ("route checks "
                  + ("FAILED" if failures else "OK")
                  + "; stdout tail:\n" + stdout_tail.text())
        return AttemptResult(temp_root, True, failures, detail, retryable=False)
    except Exception as attempt_error:  # unexpected attempt crash -> smoke failure
        return AttemptResult(temp_root, False, [repr(attempt_error)],
                             "attempt raised", retryable=False)
    finally:
        if child is not None:
            terminate_process(child)


def _remove_tree_quietly(path: Path) -> None:
    shutil.rmtree(str(path), ignore_errors=True)


def run_smoke(options, repo_root: Path) -> int:
    if not options.fixture_path.is_dir():
        print("ERROR fixture " + str(options.fixture_path) + " is not a directory")
        return EXIT_INTERNAL_ERROR
    exe_path = resolve_exe_path(options, repo_root)
    if options.build:
        built, build_output = build_binary(repo_root, options.release)
        if not built:
            print("ERROR " + " ".join(cargo_build_argv(options.release))
                  + " failed; output tail:\n"
                  + _tail_text(build_output, TAIL_LINES_KEPT))
            return EXIT_INTERNAL_ERROR
    if not exe_path.is_file():
        print("ERROR tool binary " + str(exe_path)
              + " not found (build it or pass --exe)")
        return EXIT_INTERNAL_ERROR
    started = time.monotonic()
    last_attempt = None
    for attempt_number in range(1, MAX_PORT_ATTEMPTS + 1):
        attempt = _smoke_attempt(options, repo_root, exe_path)
        try:
            if attempt.ready and not attempt.failures:
                elapsed = time.monotonic() - started
                print("boot+route checks clean "
                      "(" + ("cold" if options.cold else "warm")
                      + " fixture, attempt " + str(attempt_number)
                      + ", " + format(elapsed, ".1f") + "s)")
                print("ui_smoke: PASS")
                return EXIT_PASS
        finally:
            _remove_tree_quietly(attempt.temp_root)
        last_attempt = attempt
        if attempt.ready or not attempt.retryable:
            break
        if attempt_number < MAX_PORT_ATTEMPTS:
            print("RETRY attempt " + str(attempt_number) + " did not boot; "
                  "trying a fresh port")
    reasons = last_attempt.failures if last_attempt else ["no attempt ran"]
    for reason in reasons:
        print("FAIL " + reason)
    print("ui_smoke: FAIL " + reasons[0].replace("\n", " | "))
    return EXIT_SMOKE_FAILURE


def _tail_text(text: str, max_lines: int) -> str:
    lines = text.splitlines()
    return "\n".join(lines[-max_lines:])


def main(argv=None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    try:
        options = parse_arguments(arguments)
    except UsageError as usage_error:
        print("ERROR " + str(usage_error))
        return EXIT_INTERNAL_ERROR
    options.cold = not options.warm
    options.build = not options.no_build
    repo_root = Path(__file__).resolve().parents[1]
    if options.fixture is not None:
        options.fixture_path = Path(options.fixture).resolve()
    else:
        options.fixture_path = repo_root / "tests" / "fixtures" / "basic"
    return run_smoke(options, repo_root)


if __name__ == "__main__":
    sys.exit(main())
