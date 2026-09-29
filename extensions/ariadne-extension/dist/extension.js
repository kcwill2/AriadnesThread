"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
/** Global reference to the connected server URL. */
let serverUrl;
/**
 * Resolve the server URL from VS Code settings, with a fallback default.
 */
function resolveServerUrl() {
    const config = vscode.workspace.getConfiguration("ariadne");
    return config.get("serverUrl", "http://localhost:8080");
}
/**
 * GET /api/repo-info – fetch the server's repository root.
 * Used to verify the server is running and to know what paths it serves.
 */
async function fetchRepoInfo() {
    const url = resolveServerUrl();
    try {
        const resp = await fetch(`${url}/api/repo-info`);
        if (!resp.ok)
            throw new Error(`HTTP ${resp.status}`);
        const data = (await resp.json());
        return data;
    }
    catch (err) {
        console.error("Failed to fetch repo info:", err);
        return undefined;
    }
}
/**
 * Open a file in the editor at the given line (1-based).
 */
async function openFileInEditor(filePath, line = 1) {
    const uri = vscode.Uri.file(filePath);
    try {
        const doc = await vscode.workspace.openTextDocument(uri);
        const editor = await vscode.window.showTextDocument(doc);
        // Move cursor to the line.
        if (line > 0) {
            const pos = new vscode.Position(line - 1, 0);
            editor.selection = new vscode.Selection(pos, pos);
            // Reveal the line in the viewport.
            editor.revealRange(new vscode.Range(pos, pos), vscode.TextEditorRevealType.InCenter);
        }
    }
    catch (err) {
        vscode.window.showErrorMessage(`Failed to open ${filePath}: ${err}`);
    }
}
/**
 * Extension entry-point – register commands and optionally auto-connect.
 */
function activate(context) {
    console.log("Ariadne extension activating …");
    const connectCmd = vscode.commands.registerCommand("ariadne.connect", async () => {
        const repo = await fetchRepoInfo();
        if (!repo) {
            vscode.window.showErrorMessage("Could not reach Ariadne server – is it running?");
            return;
        }
        await vscode.commands.executeCommand("setContext", "ariadne.connected", true);
        vscode.window.showInformationMessage(`Ariadne: connected to ${repo.repo_root}`);
    });
    /**
     * Go to Definition – given a symbol identifier, query the server for its
     * metadata and open the corresponding file locally.
     *
     * Since the server runs locally, paths are shared and no reconciliation is needed.
     */
    const goDefCmd = vscode.commands.registerCommand("ariadne.goToDefinition", async (symbolId) => {
        const url = resolveServerUrl();
        // If no symbol supplied, ask the user.
        if (!symbolId) {
            const id = await vscode.window.showInputBox({
                prompt: "Enter symbol ID to jump to",
                placeHolder: "e.g. 42 or MyNamespace::myFunc"
            });
            if (!id)
                return;
            symbolId = id;
        }
        // Build query – server accepts `file=<path>` for source lookup.
        // Here we use a heuristic: try to resolve the symbol to a file path.
        // The server's /api/graph/search endpoint can locate the file by name.
        const searchResp = await fetch(`${url}/api/graph/search?q=${encodeURIComponent(symbolId)}&graph_type=symbol`);
        if (!searchResp.ok) {
            vscode.window.showErrorMessage(`Server search failed: ${searchResp.status}`);
            return;
        }
        const results = (await searchResp.json());
        if (!results.length) {
            vscode.window.showInformationMessage(`No results for symbol '${symbolId}'`);
            return;
        }
        const first = results[0];
        const filePath = first.id ?? first.label ?? symbolId;
        const line = first.line ?? 1;
        await openFileInEditor(filePath, line);
    });
    /**
     * Navigate to File – open an arbitrary file path in the editor.
     */
    const goFileCmd = vscode.commands.registerCommand("ariadne.navigateToFile", async (filePath) => {
        if (!filePath) {
            const input = await vscode.window.showInputBox({
                prompt: "Enter file path",
                placeHolder: "e.g. /home/user/project/src/main.cpp"
            });
            if (!input)
                return;
            filePath = input;
        }
        await openFileInEditor(filePath, 1);
    });
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
        if (!input)
            return;
        await vscode.workspace.getConfiguration("ariadne").update("serverUrl", input, vscode.ConfigurationTarget.Global);
        vscode.window.showInformationMessage(`Ariadne server URL set to ${input}`);
    });
    context.subscriptions.push(connectCmd, goDefCmd, goFileCmd, showGraphCmd, configureServerCmd);
    // Auto-connect if enabled.
    const config = vscode.workspace.getConfiguration("ariadne");
    const autoConnect = config.get("autoConnect", true);
    if (autoConnect) {
        // Fire-and-forget so activation isn't blocked.
        vscode.commands.executeCommand("ariadne.connect").then(undefined, (err) => {
            // Ignore errors during auto-connect
            console.debug("Auto-connect failed:", err);
        });
    }
}
function deactivate() {
    console.log("Ariadne extension deactivated");
}
//# sourceMappingURL=extension.js.map