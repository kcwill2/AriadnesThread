# Ariadne Extension for VS Code

![Ariadne's Thread](media/banner.jpg)

The Ariadne Extension allows you to:

## 🚀 Core Features

- **Connect to Analysis Server**: Establish connection to the local Ariadne analysis server
- **Go to Definition**: Click on symbols in analysis results to jump to source code
- **Navigate Files**: Open files from the analysis tree directly in the editor
- **View Dependency Graphs**: Open the interactive dependency graph viewer in your browser

## 🔧 Configuration

Open VS Code settings and configure:

```json
{
  "ariadne.serverUrl": "http://localhost:7742",
  "ariadne.toolPath": "",
  "ariadne.autoConnect": true
}
```

### Server URL
- Where the Ariadne analysis server is running (default: localhost:7742, matching the CLI `tool ui` default port)
- The server must run locally for path sharing to work

### Tool Binary Path
- Path to the Ariadne `tool` binary used by "Start Ariadne Server"
- Leave empty to auto-detect `target/release/tool` or `target/debug/tool` under the project root

### Auto Connect
- Whether to automatically connect to server on VS Code startup

## 🛠️ Commands

### Ariadne: Connect to Server
Establishes connection to the analysis server.

### Ariadne: Start Ariadne Server (project root as input)
Launches the AT server (`tool ui <project root> --port <port> --no-open`) with the open workspace's project root as the analysis input, waits until it answers, then marks the extension connected. If no `tool` binary is found it offers to run `cargo build --release --bin tool` or to pick an existing binary. Server output goes to the "Ariadne Server" output channel.

### Ariadne: Stop Ariadne Server
Stops the server started by "Start Ariadne Server" from this window.

### Ariadne: Go to Definition
Jumps to the definition of a symbol you select (either by input or IDE).

### Ariadne: Navigate to File
Opens a file from the analysis directly in the editor.

### Ariadne: Show Dependency Graph
Opens the web-based graph viewer.

### Ariadne: Configure Server URL
Manually update the server URL.

## 🔄 Path Reconciliation

Since the server runs locally (same machine as the editor), paths are shared. No path reconciliation is needed.

When the extension connects:
1. The server reports its `repo_root` path
2. The extension confirms it matches the workspace
3. Navigation uses server paths directly (they're the same as local paths)

This means:
- **Same repo root**: Direct navigation works perfectly
- **Different repo root**: The extension can still navigate if you've configured `ariadne.serverUrl` to point to the correct server

## 📋 Architecture

**Server (crates/ui)**:
- Rust-based web server with endpoint `/api/repo-info`
- Returns server-side repository metadata
- Runs locally on the same machine

**Extension (ariadne-extension)**:
- TypeScript VS Code extension
- Connects to local server
- Opens files in VS Code editor

**Flow**:
```
User selects symbol → Extension fetches symbol info → VS Code opens file
```

## 🛠️ How to Use

1. **Start the Analysis Server** - two options:
   - From VS Code: run "Ariadne: Start Ariadne Server" - it uses the current
     workspace's project root as the analysis input (no terminal needed).
   - From the terminal:
     ```bash
     cargo build --release --bin tool
     cargo run --release --bin tool -- ui /path/to/project --port 7742 --no-open
     ```
    Release is strongly recommended (debug indexing measured ~20x slower than
    release at TU scale - M27 T1a launch-profile finding, 2026-10-05).

2. **Install the Extension**:
   - Run `code .` in this directory
   - Open the Command Palette (`Ctrl+Shift+P`)
   - Choose "Extensions: Install Extension"
   - Select "Ariadne's Thread" from the list

3. **Connect**: Run "Ariadne: Connect to Server".

4. **Navigate**: Use "Go to Definition" or "Navigate to File" commands.

## 🎯 Example

When you select a symbol in the Ariadne analysis results:
1. Server reports: `repo_root: /home/user/project`
2. Extension confirms this matches your workspace
3. Extension opens file in VS Code: `/home/user/project/src/main.cpp`
4. You can edit and inspect the local code directly

## 🔒 Security & Privacy

- The extension only sends HTTP requests to the configured server
- No private configuration data is stored
- Server connection is optional and opt-in

## 🛠️ Technical Details

### Path Reconciliation (Local-First)
Since the server runs locally, the extension can rely on shared paths:

```typescript
// No mapping needed - paths are shared
const repoInfo = await fetchRepoInfo();
// repo_info.repo_root should match vscode.workspace.workspaceFolders[0].uri.fsPath
await openFileInEditor(filePath);
```

### API Endpoints Used
- `GET /api/repo-info` - Repository metadata (to verify server is running)
- `GET /api/graph/search` - Search for symbols/files

## 📝 License

MIT
