import * as vscode from "vscode";
import * as cp from "child_process";
import * as fs from "fs";
import * as path from "path";

/**
 * Ariadne Extension for VS Code
 * 
 * Connects to a local Ariadne analysis server and provides navigation
 * from the dependency graph to the local source code.
 * 
 * This extension assumes the server runs on the same machine as the
 * editor, so paths are shared and no reconciliation is needed.
 */

interface RepoInfoResponse {
  repo_root: string;
  file_paths: string[];
}

/** Global reference to the connected server URL. */
let serverUrl: string | undefined;

/** The server child process started by this window, if any. */
let serverProcess: cp.ChildProcess | undefined;

/** Output channel showing the server's stdout/stderr. */
let serverChannel: vscode.OutputChannel | undefined;

/** Milliseconds to wait for the server to answer /api/repo-info after start. */
const SERVER_START_TIMEOUT_MS = 180000;

/** Poll interval while waiting for the server to come up. */
const SERVER_POLL_INTERVAL_MS = 2000;

function getServerChannel(): vscode.OutputChannel {
  if (!serverChannel) {
    serverChannel = vscode.window.createOutputChannel("Ariadne Server");
  }
  return serverChannel;
}

/**
 * Extract the port from the configured server URL (CLI 'tool ui' default: 7742).
 */
function resolvePort(): number {
  try {
    const parsed = new URL(resolveServerUrl());
    if (parsed.port) {
      return Number(parsed.port);
    }
    return parsed.protocol === "https:" ? 443 : 80;
  } catch {
    return 7742;
  }
}

/**
 * Resolve the server URL from VS Code settings, with a fallback default.
 */
function resolveServerUrl(): string {
  const config = vscode.workspace.getConfiguration("ariadne");
  return config.get<string>("serverUrl", "http://localhost:8080");
}

/**
 * GET /api/repo-info – fetch the server's repository root.
 * Used to verify the server is running and to know what paths it serves.
 */
async function fetchRepoInfo(): Promise<RepoInfoResponse | undefined> {
  const url = resolveServerUrl();
  try {
    const resp = await fetch(`${url}/api/repo-info`);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = (await resp.json()) as RepoInfoResponse;
    return data;
  } catch (err) {
    console.error("Failed to fetch repo info:", err);
    return undefined;
  }
}

/**
 * Pick the project root to hand to the server as analysis input.
 */
async function pickProjectRoot(): Promise<string | undefined> {
  const folders = vscode.workspace.workspaceFolders ?? [];
  if (folders.length === 0) {
    vscode.window.showErrorMessage(
      "Ariadne: no workspace folder open - open the project you want to analyze first."
    );
    return undefined;
  }
  if (folders.length === 1) {
    return folders[0].uri.fsPath;
  }
  const picked = await vscode.window.showWorkspaceFolderPick({
    placeHolder: "Select the Ariadne project root to analyze"
  });
  return picked ? picked.uri.fsPath : undefined;
}

/** Candidate binary locations under a project root (release preferred, per AGENTS.md launch guidance). */
function toolBinaryCandidates(root: string): string[] {
  const exe = process.platform === "win32" ? "tool.exe" : "tool";
  return [
    path.join(root, "target", "release", exe),
    path.join(root, "target", "debug", exe)
  ];
}

/**
 * Resolve the Ariadne 'tool' binary: explicit ariadne.toolPath setting first,
 * then target/release/tool, then target/debug/tool under the project root.
 */
function resolveToolBinary(root: string): string | undefined {
  const configured = vscode.workspace
    .getConfiguration("ariadne")
    .get<string>("toolPath", "")
    .trim();
  if (configured) {
    return fs.existsSync(configured) ? configured : undefined;
  }
  return toolBinaryCandidates(root).find((candidate) => fs.existsSync(candidate));
}

/** Run 'cargo build --release --bin tool' in the project root, streaming to the output channel. */
function buildToolBinary(root: string): Promise<boolean> {
  const channel = getServerChannel();
  channel.show(true);
  channel.appendLine(`$ cargo build --release --bin tool (cwd ${root})`);
  return new Promise((resolve) => {
    const child = cp.spawn("cargo", ["build", "--release", "--bin", "tool"], {
      cwd: root,
      windowsHide: true
    });
    child.stdout?.on("data", (chunk) => channel.append(String(chunk)));
    child.stderr?.on("data", (chunk) => channel.append(String(chunk)));
    child.on("error", (err) => {
      channel.appendLine(`cargo failed to start: ${err}`);
      resolve(false);
    });
    child.on("exit", (code) => resolve(code === 0));
  });
}

/**
 * Find the tool binary, or offer to build it / pick a path when missing.
 */
async function resolveOrBuildToolBinary(root: string): Promise<string | undefined> {
  const found = resolveToolBinary(root);
  if (found) {
    return found;
  }
  const BUILD = "Build (cargo build --release --bin tool)";
  const PICK = "Select binary path...";
  const choice = await vscode.window.showWarningMessage(
    `Ariadne: no 'tool' binary found under ${root}. Build it now or select an existing binary.`,
    BUILD,
    PICK
  );
  if (choice === BUILD) {
    const ok = await buildToolBinary(root);
    if (!ok) {
      vscode.window.showErrorMessage("Ariadne: cargo build failed - see the Ariadne Server output.");
      return undefined;
    }
    return toolBinaryCandidates(root)[0];
  }
  if (choice === PICK) {
    const picked = await vscode.window.showOpenDialog({
      canSelectFiles: true,
      canSelectFolders: false,
      canSelectMany: false,
      title: "Select the Ariadne 'tool' binary"
    });
    if (!picked || picked.length === 0) {
      return undefined;
    }
    const chosen = picked[0].fsPath;
    await vscode.workspace
      .getConfiguration("ariadne")
      .update("toolPath", chosen, vscode.ConfigurationTarget.Workspace);
    return chosen;
  }
  return undefined;
}

/** Poll /api/repo-info until the server answers or the deadline passes. */
async function waitForServer(): Promise<RepoInfoResponse | undefined> {
  const deadline = Date.now() + SERVER_START_TIMEOUT_MS;
  while (Date.now() < deadline) {
    if (!serverProcess) {
      return undefined;
    }
    const repo = await fetchRepoInfo();
    if (repo) {
      return repo;
    }
    await new Promise((resolve) => setTimeout(resolve, SERVER_POLL_INTERVAL_MS));
  }
  return undefined;
}

/** Whether the tracked child process is still running. */
function isServerRunning(): boolean {
  return serverProcess !== undefined && serverProcess.exitCode === null;
}

/** Terminate the server process (and its tree on Windows). */
function stopServerProcess(): void {
  if (!serverProcess) {
    return;
  }
  const pid = serverProcess.pid;
  if (process.platform === "win32" && pid) {
    try {
      cp.execSync(`taskkill /pid ${pid} /T /F`, { stdio: "ignore" });
    } catch {
      // Process already gone; nothing to kill.
    }
  } else {
    serverProcess.kill();
  }
  serverProcess = undefined;
}

/**
 * Start 'tool ui <projectRoot> --port <port> --no-open' and wait until it answers.
 */
async function startAriadneServer(): Promise<void> {
  if (isServerRunning()) {
    const restart = await vscode.window.showInformationMessage(
      "Ariadne: server is already running in this window.",
      "Stop and restart"
    );
    if (restart !== "Stop and restart") {
      return;
    }
    stopServerProcess();
  }

  const root = await pickProjectRoot();
  if (!root) {
    return;
  }
  const binary = await resolveOrBuildToolBinary(root);
  if (!binary) {
    return;
  }

  const port = resolvePort();
  const channel = getServerChannel();
  channel.show(true);
  channel.appendLine(`$ ${binary} ui "${root}" --port ${port} --no-open`);

  serverProcess = cp.spawn(
    binary,
    ["ui", root, "--port", String(port), "--no-open"],
    { cwd: root, windowsHide: true }
  );
  serverProcess.stdout?.on("data", (chunk) => channel.append(String(chunk)));
  serverProcess.stderr?.on("data", (chunk) => channel.append(String(chunk)));
  serverProcess.on("exit", (code) => {
    channel.appendLine(`Ariadne server exited with code ${code ?? "null"}`);
    serverProcess = undefined;
    void vscode.commands.executeCommand("setContext", "ariadne.connected", false);
  });

  const repo = await waitForServer();
  if (repo) {
    await vscode.commands.executeCommand("setContext", "ariadne.connected", true);
    vscode.window.showInformationMessage(
      `Ariadne: server started for ${repo.repo_root} at ${resolveServerUrl()}`
    );
  } else if (isServerRunning()) {
    vscode.window.showWarningMessage(
      "Ariadne: server is running but did not answer yet (large repos may still be indexing) - see the Ariadne Server output."
    );
  } else {
    vscode.window.showErrorMessage(
      "Ariadne: server exited during startup - see the Ariadne Server output."
    );
  }
}

/**
 * Open a file in the editor at the given line (1-based).
 */
async function openFileInEditor(
  filePath: string,
  line: number = 1
): Promise<void> {
  const uri = vscode.Uri.file(filePath);
  try {
    const doc = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(doc);
    // Move cursor to the line.
    if (line > 0) {
      const pos = new vscode.Position(line - 1, 0);
      editor.selection = new vscode.Selection(pos, pos);
      // Reveal the line in the viewport.
      editor.revealRange(
        new vscode.Range(pos, pos),
        vscode.TextEditorRevealType.InCenter
      );
    }
  } catch (err) {
    vscode.window.showErrorMessage(`Failed to open ${filePath}: ${err}`);
  }
}

/**
 * Extension entry-point – register commands and optionally auto-connect.
 */
export function activate(context: vscode.ExtensionContext) {
  console.log("Ariadne extension activating …");

  const connectCmd = vscode.commands.registerCommand("ariadne.connect", async () => {
    const repo = await fetchRepoInfo();
    if (!repo) {
      vscode.window.showErrorMessage("Could not reach Ariadne server – is it running?");
      return;
    }

    await vscode.commands.executeCommand("setContext", "ariadne.connected", true);
    vscode.window.showInformationMessage(
      `Ariadne: connected to ${repo.repo_root}`
    );
  });

  /**
   * Go to Definition – given a symbol identifier, query the server for its
   * metadata and open the corresponding file locally.
   * 
   * Since the server runs locally, paths are shared and no reconciliation is needed.
   */
  const goDefCmd = vscode.commands.registerCommand(
    "ariadne.goToDefinition",
    async (symbolId?: string) => {
      const url = resolveServerUrl();

      // If no symbol supplied, ask the user.
      if (!symbolId) {
        const id = await vscode.window.showInputBox({
          prompt: "Enter symbol ID to jump to",
          placeHolder: "e.g. 42 or MyNamespace::myFunc"
        });
        if (!id) return;
        symbolId = id;
      }

      // Build query – server accepts `file=<path>` for source lookup.
      // Here we use a heuristic: try to resolve the symbol to a file path.
      // The server's /api/graph/search endpoint can locate the file by name.
      const searchResp = await fetch(
        `${url}/api/graph/search?q=${encodeURIComponent(symbolId)}&graph_type=symbol`
      );
      if (!searchResp.ok) {
        vscode.window.showErrorMessage(`Server search failed: ${searchResp.status}`);
        return;
      }
      const results = (await searchResp.json()) as any[];
      if (!results.length) {
        vscode.window.showInformationMessage(`No results for symbol '${symbolId}'`);
        return;
      }

      const first = results[0];
      const filePath = first.id ?? first.label ?? symbolId;
      const line = first.line ?? 1;

      await openFileInEditor(filePath, line);
    }
  );

  /**
   * Navigate to File – open an arbitrary file path in the editor.
   */
  const goFileCmd = vscode.commands.registerCommand(
    "ariadne.navigateToFile",
    async (filePath?: string) => {
      if (!filePath) {
        const input = await vscode.window.showInputBox({
          prompt: "Enter file path",
          placeHolder: "e.g. /home/user/project/src/main.cpp"
        });
        if (!input) return;
        filePath = input;
      }

      await openFileInEditor(filePath, 1);
    }
  );

  /**
   * Show Dependency Graph – open the server's graph UI in the browser.
   */
  const showGraphCmd = vscode.commands.registerCommand("ariadne.showGraph", async () => {
    const url = resolveServerUrl();
    await vscode.env.openExternal(vscode.Uri.parse(url));
  });

  const configureServerCmd = vscode.commands.registerCommand("ariadne.configureServer", async () => {
    const current = resolveServerUrl();
    const input = await vscode.window.showInputBox({
      prompt: "Enter Ariadne server URL",
      value: current
    });
    if (!input) return;
    await vscode.workspace.getConfiguration("ariadne").update("serverUrl", input, vscode.ConfigurationTarget.Global);
    vscode.window.showInformationMessage(`Ariadne server URL set to ${input}`);
  });

  const startServerCmd = vscode.commands.registerCommand(
    "ariadne.startServer",
    async () => {
      await startAriadneServer();
    }
  );

  const stopServerCmd = vscode.commands.registerCommand("ariadne.stopServer", () => {
    if (!isServerRunning()) {
      vscode.window.showInformationMessage("Ariadne: no server is running in this window.");
      return;
    }
    stopServerProcess();
    void vscode.commands.executeCommand("setContext", "ariadne.connected", false);
    vscode.window.showInformationMessage("Ariadne: server stopped.");
  });

  context.subscriptions.push(connectCmd, startServerCmd, stopServerCmd, goDefCmd, goFileCmd, showGraphCmd, configureServerCmd);

  // Auto-connect if enabled.
  const config = vscode.workspace.getConfiguration("ariadne");
  const autoConnect = config.get<boolean>("autoConnect", true);
  if (autoConnect) {
    // Fire-and-forget so activation isn't blocked.
    vscode.commands.executeCommand("ariadne.connect").then(undefined, (err) => {
      // Ignore errors during auto-connect
      console.debug("Auto-connect failed:", err);
    });
  }
}

export function deactivate() {
  stopServerProcess();
  serverChannel?.dispose();
  serverChannel = undefined;
  console.log("Ariadne extension deactivated");
}