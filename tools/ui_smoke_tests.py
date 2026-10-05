"""Unit tests for tools/ui_smoke.py (UI runtime smoke tool).

Run directly: python tools\\ui_smoke_tests.py
Also runs under: python -m unittest discover -s tools -p "*_tests.py"

All offline: the UI server is exercised through a stdlib http.server stub,
the cold-strip logic against a fake fixture tree in a temp dir. No cargo.
"""

import contextlib
import http.server
import io
import json
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import ui_smoke

# Real /api/status body shape, verified against crates/ui/src/server.rs
# StatusResponse and crates/ui/src/dto.rs ServerProgress (2026-10-01):
# readiness lives at the nested path progress.state.
def status_payload(state_value):
    return {
        "repo_name": "basic",
        "stats": None,
        "progress": {
            "state": state_value,
            "phase": "Done" if state_value == "ready" else "Analyzing",
            "percent": 100 if state_value == "ready" else 40,
            "message": "Ready" if state_value == "ready" else "Parsing",
        },
        "stages": [],
        "last_refresh": None,
        "rebuilding": False,
        "reused_tus": 0,
        "reanalyzed_tus": 0,
    }


class StubRequestHandler(http.server.BaseHTTPRequestHandler):
    status_poll_count = 0
    ready_after_polls = 0
    routes = {}

    def do_GET(self):
        handler_class = type(self)
        if self.path == "/api/status":
            handler_class.status_poll_count += 1
            is_ready = (handler_class.status_poll_count
                        > handler_class.ready_after_polls)
            body = json.dumps(
                status_payload("ready" if is_ready else "loading")
            ).encode("utf-8")
            self._respond(200, "application/json; charset=utf-8", "", body)
            return
        entry = handler_class.routes.get(self.path)
        if entry is None:
            self.send_error(404, "no such route")
            return
        content_type, cache_control, body = entry
        self._respond(200, content_type, cache_control, body)

    def _respond(self, status, content_type, cache_control, body):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        if cache_control:
            self.send_header("Cache-Control", cache_control)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, fmt, *args):
        return


def fully_compliant_routes():
    routes = {}
    for route, expected_type, expect_no_store in ui_smoke.ROUTE_ASSERTIONS:
        body = b"<!doctype html><p>ok</p>" if route == "/" else b"payload-body"
        routes[route] = (
            expected_type + "; charset=utf-8",
            "no-store" if expect_no_store else "",
            body,
        )
    return routes


class StubUiServer:
    """Context manager: stdlib stub server for /api/status + static routes."""

    def __init__(self, ready_after_polls=0, routes=None):
        handler_routes = routes if routes is not None else fully_compliant_routes()

        class Handler(StubRequestHandler):
            pass

        Handler.status_poll_count = 0
        Handler.ready_after_polls = ready_after_polls
        Handler.routes = handler_routes
        self._handler_class = Handler
        self._server = http.server.ThreadingHTTPServer(
            ("127.0.0.1", 0), Handler)
        self._thread = threading.Thread(
            target=self._server.serve_forever, daemon=True)

    def __enter__(self):
        self._thread.start()
        return self

    def __exit__(self, exc_type, exc_value, exc_traceback):
        self._server.shutdown()
        self._server.server_close()
        self._thread.join(timeout=5)
        return False

    @property
    def base_url(self):
        host, port = self._server.server_address
        return "http://" + host + ":" + str(port)

    @property
    def status_poll_count(self):
        return self._handler_class.status_poll_count


def run_main_captured(arguments):
    captured = io.StringIO()
    with contextlib.redirect_stdout(captured):
        code = ui_smoke.main(arguments)
    return code, captured.getvalue()


def write_fixture_tree(root: Path):
    (root / "src").mkdir(parents=True)
    (root / "src" / "main.cpp").write_text("int main(){return 0;}\n",
                                           encoding="utf-8")
    (root / "artifacts" / "cache").mkdir(parents=True)
    (root / "artifacts" / "cache" / "blob.bin").write_bytes(b"cached")
    (root / ".ariadne" / "index").mkdir(parents=True)
    (root / ".ariadne" / "index" / "store.bin").write_bytes(b"stale")


class StatusParsingTests(unittest.TestCase):
    def test_ready_state_extracted_from_real_product_shape(self):
        payload = status_payload("ready")
        self.assertEqual(ui_smoke.extract_progress_state(payload), "ready")

    def test_loading_state_extracted_from_real_product_shape(self):
        payload = status_payload("loading")
        self.assertEqual(ui_smoke.extract_progress_state(payload), "loading")

    def test_missing_or_malformed_fields_read_as_unknown(self):
        for payload in ({},
                        {"progress": None},
                        {"progress": {}},
                        {"progress": {"state": 7}},
                        {"progress": "ready"},
                        [],
                        None):
            self.assertIsNone(ui_smoke.extract_progress_state(payload))


class ColdStripTests(unittest.TestCase):
    def setUp(self):
        self._temp_dir = Path(tempfile.mkdtemp(prefix="ui-smoke-tests-"))
        self.addCleanup(shutil.rmtree, str(self._temp_dir), True)
        self.fixture = self._temp_dir / "fixture"
        self.fixture.mkdir()
        write_fixture_tree(self.fixture)

    def test_cold_strip_removes_cache_dirs_from_copy(self):
        copy_root = self._temp_dir / "copy"
        ui_smoke.copy_fixture_into_copy(self.fixture, copy_root)
        removed = ui_smoke.strip_cache_directories(copy_root)
        self.assertEqual(sorted(removed), [".ariadne", "artifacts"])
        self.assertFalse((copy_root / "artifacts").exists())
        self.assertFalse((copy_root / ".ariadne").exists())
        self.assertTrue((copy_root / "src" / "main.cpp").is_file())

    def test_strip_never_touches_the_original_fixture(self):
        copy_root = self._temp_dir / "copy"
        ui_smoke.copy_fixture_into_copy(self.fixture, copy_root)
        ui_smoke.strip_cache_directories(copy_root)
        self.assertTrue((self.fixture / "artifacts" / "cache" / "blob.bin").is_file())
        self.assertTrue((self.fixture / ".ariadne" / "index" / "store.bin").is_file())

    def test_warm_copy_keeps_artifacts_and_guard_reports_residual(self):
        copy_root = self._temp_dir / "warm-copy"
        ui_smoke.copy_fixture_into_copy(self.fixture, copy_root)
        # --warm skips both the strip and the guard, so a --warm copy is
        # exactly this: cache dirs still present and the guard would flag it.
        self.assertTrue((copy_root / "artifacts" / "cache" / "blob.bin").is_file())
        guard_reason = ui_smoke.verify_cold_copy_clean(copy_root)
        self.assertIsNotNone(guard_reason)
        self.assertIn("artifacts", guard_reason)

    def test_guard_fails_loudly_when_planted_artifacts_remain(self):
        planted = self._temp_dir / "planted"
        (planted / "artifacts").mkdir(parents=True)
        guard_reason = ui_smoke.verify_cold_copy_clean(planted)
        self.assertIsNotNone(guard_reason)
        self.assertIn("artifacts", guard_reason)

    def test_guard_passes_clean_copy(self):
        clean = self._temp_dir / "clean"
        (clean / "src").mkdir(parents=True)
        self.assertIsNone(ui_smoke.verify_cold_copy_clean(clean))


class FreePortProbeTests(unittest.TestCase):
    def test_probe_returns_a_bindable_loopback_port(self):
        port = ui_smoke.probe_free_port()
        self.assertIsInstance(port, int)
        self.assertGreater(port, 0)
        verify_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        try:
            verify_socket.bind(("127.0.0.1", port))
        finally:
            verify_socket.close()

    def test_probe_ports_are_usable_sequentially(self):
        first_port = ui_smoke.probe_free_port()
        second_port = ui_smoke.probe_free_port()
        for candidate_port in (first_port, second_port):
            verify_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            try:
                verify_socket.bind(("127.0.0.1", candidate_port))
            finally:
                verify_socket.close()


class ReadinessPollingTests(unittest.TestCase):
    def test_ready_transition_after_n_polls(self):
        with StubUiServer(ready_after_polls=3) as stub:
            ready = ui_smoke.poll_until_ready(
                stub.base_url, timeout_seconds=10.0, poll_interval=0.02)
            self.assertTrue(ready)
            self.assertGreaterEqual(stub.status_poll_count, 4)

    def test_timeout_path_exits_fast(self):
        with StubUiServer(ready_after_polls=10 ** 9) as stub:
            started = time.monotonic()
            ready = ui_smoke.poll_until_ready(
                stub.base_url, timeout_seconds=0.3, poll_interval=0.05)
            elapsed = time.monotonic() - started
            self.assertFalse(ready)
            self.assertLess(elapsed, 5.0)
            self.assertGreaterEqual(stub.status_poll_count, 1)

    def test_dead_server_stops_polling_immediately(self):
        with StubUiServer(ready_after_polls=10 ** 9) as stub:
            started = time.monotonic()
            ready = ui_smoke.poll_until_ready(
                stub.base_url, timeout_seconds=30.0, poll_interval=0.5,
                is_server_running=lambda: False)
            elapsed = time.monotonic() - started
            self.assertFalse(ready)
            self.assertLess(elapsed, 1.0)


class RouteAssertionTests(unittest.TestCase):
    def test_fully_compliant_stub_yields_no_failures(self):
        with StubUiServer() as stub:
            self.assertEqual(ui_smoke.check_routes(stub.base_url), [])

    def test_missing_no_store_on_index_is_a_failure(self):
        routes = fully_compliant_routes()
        content_type, _cache_control, body = routes["/"]
        routes["/"] = (content_type, "", body)
        with StubUiServer(routes=routes) as stub:
            failures = ui_smoke.check_routes(stub.base_url)
        self.assertTrue(any("no-store" in entry and "GET /:" in entry
                            for entry in failures), failures)

    def test_wrong_content_type_is_a_failure(self):
        routes = fully_compliant_routes()
        _type, cache_control, body = routes["/styles.css"]
        routes["/styles.css"] = ("application/octet-stream", cache_control, body)
        with StubUiServer(routes=routes) as stub:
            failures = ui_smoke.check_routes(stub.base_url)
        self.assertTrue(any("/styles.css" in entry for entry in failures),
                        failures)

    def test_missing_route_404_is_a_failure(self):
        routes = fully_compliant_routes()
        del routes["/app/main.js"]
        with StubUiServer(routes=routes) as stub:
            failures = ui_smoke.check_routes(stub.base_url)
        self.assertTrue(any("/app/main.js" in entry for entry in failures),
                        failures)

    def test_empty_body_is_a_failure(self):
        routes = fully_compliant_routes()
        content_type, cache_control, _body = routes["/app/api.js"]
        routes["/app/api.js"] = (content_type, cache_control, b"")
        with StubUiServer(routes=routes) as stub:
            failures = ui_smoke.check_routes(stub.base_url)
        self.assertTrue(any("/app/api.js" in entry for entry in failures),
                        failures)


class CliValidationTests(unittest.TestCase):
    def test_unknown_flag_exits_2(self):
        code, output = run_main_captured(["--definitely-not-a-flag"])
        self.assertEqual(code, 2)
        self.assertIn("ERROR", output)

    def test_conflicting_temperature_flags_exit_2(self):
        code, _output = run_main_captured(["--cold", "--warm"])
        self.assertEqual(code, 2)

    def test_zero_timeout_exits_2(self):
        code, output = run_main_captured(["--timeout=0"])
        self.assertEqual(code, 2)
        self.assertIn("ERROR", output)

    def test_non_numeric_timeout_exits_2(self):
        code, _output = run_main_captured(["--timeout=later"])
        self.assertEqual(code, 2)

    def test_missing_exe_with_no_build_exits_2(self):
        missing = Path(tempfile.gettempdir()) / "ui-smoke-definitely-missing.exe"
        self.assertFalse(missing.exists())
        code, output = run_main_captured(["--no-build", "--exe", str(missing)])
        self.assertEqual(code, 2)
        self.assertIn("ERROR", output)


class ReleaseProfilePlumbingTests(unittest.TestCase):
    """M28 T3 rider: the --release flag's build-argv and target wiring.

    Plumbing-only by design (no server boot, no cargo): the honest ceiling
    for a doc/flag change is the flag plumbing itself; the live release-boot
    witness belongs to the reviewer's `ui_smoke --release` run.
    """

    def setUp(self):
        self.repo_root = Path(tempfile.mkdtemp(prefix="ui-smoke-profile-"))
        self.addCleanup(shutil.rmtree, str(self.repo_root), True)

    def captured_build_argv(self, cargo_call):
        """Run one build_binary call with subprocess.run faked.

        Returns (build_ok, cargo_argv) recorded from the fake.
        """
        recorded = []

        def fake_run(command, **_kwargs):
            recorded.append(list(command))
            return subprocess.CompletedProcess(command, returncode=0,
                                               stdout="", stderr="")

        with mock.patch.object(ui_smoke.subprocess, "run", fake_run):
            build_ok, _output = cargo_call()
        self.assertEqual(len(recorded), 1,
                         "expected exactly one cargo invocation")
        return build_ok, recorded[0]

    def test_default_build_argv_and_exe_stay_debug_profile(self):
        # Characterization floor (written BEFORE the flag existed): the
        # default route must keep exactly today's argv and target/debug.
        options = ui_smoke.parse_arguments([])
        build_ok, argv = self.captured_build_argv(
            lambda: ui_smoke.build_binary(self.repo_root))
        self.assertTrue(build_ok)
        self.assertEqual(argv, ["cargo", "build", "--bin", "tool"])
        exe_path = ui_smoke.resolve_exe_path(options, self.repo_root)
        self.assertEqual(exe_path,
                         self.repo_root / "target" / "debug" / exe_path.name)
        self.assertIn(exe_path.name, ("tool", "tool.exe"))

    def test_release_flag_builds_cargo_build_release(self):
        options = ui_smoke.parse_arguments(["--release"])
        self.assertTrue(options.release)
        build_ok, argv = self.captured_build_argv(
            lambda: ui_smoke.build_binary(self.repo_root, options.release))
        self.assertTrue(build_ok)
        self.assertEqual(argv, ["cargo", "build", "--release", "--bin", "tool"])

    def test_release_resolves_exe_under_target_release(self):
        options = ui_smoke.parse_arguments(["--release"])
        exe_path = ui_smoke.resolve_exe_path(options, self.repo_root)
        self.assertEqual(exe_path,
                         self.repo_root / "target" / "release" / exe_path.name)
        self.assertIn(exe_path.name, ("tool", "tool.exe"))

    def test_release_no_build_resolves_release_without_cargo_call(self):
        # The skip-build branch itself is the pre-existing `if options.build`
        # wiring (locked by test_missing_exe_with_no_build_exits_2); this
        # test pins the release-profile half: parse + resolution, zero cargo.
        options = ui_smoke.parse_arguments(["--release", "--no-build"])
        self.assertTrue(options.no_build)
        recorded = []
        with mock.patch.object(ui_smoke.subprocess, "run",
                               lambda *args, **kwargs: recorded.append(args)):
            exe_path = ui_smoke.resolve_exe_path(options, self.repo_root)
        self.assertEqual(recorded, [], "--no-build must not invoke cargo")
        self.assertEqual(exe_path.parent.name, "release")

    def test_exe_option_wins_over_release_and_target_dir(self):
        explicit_exe = self.repo_root / "somewhere" / "tool.exe"
        options = ui_smoke.parse_arguments(
            ["--release", "--target-dir", "target/other",
             "--exe", str(explicit_exe)])
        exe_path = ui_smoke.resolve_exe_path(options, self.repo_root)
        self.assertEqual(exe_path, explicit_exe.resolve())

    def test_target_dir_option_wins_over_release_default(self):
        options = ui_smoke.parse_arguments(
            ["--release", "--target-dir", "custom/out"])
        exe_path = ui_smoke.resolve_exe_path(options, self.repo_root)
        self.assertEqual(exe_path,
                         self.repo_root / "custom" / "out" / exe_path.name)
        self.assertIn(exe_path.name, ("tool", "tool.exe"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
