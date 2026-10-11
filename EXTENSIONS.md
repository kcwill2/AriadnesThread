# Ariadne's Thread - Architecture Overview

This is a C++ dependency analysis and visualization platform. It consists of:

## 🔧 Rust Core (crates/ui)
- **Web Server** - Analysis backend with HTTP API
- **Static Frontend** - Graph visualization UI (HTML/JS/CSS)
- **Analysis Engine** - C++ dependency calculation

## 💻 VS Code Extension (extensions/ariadne-extension)
- **Extension Host** - VS Code integration
- **Navigation** - Jump to definition, file opening
- **Local Server** - Assumes server runs locally (same machine)

## 🏗️ Development Setup

### Local Development

1. **Start the Analysis Server**:
   ```bash
   cd crates/ui
   cargo build --release --features server
   cargo run --release --bin cli --features server
   ```
   Quick debug alternative (debug indexing measured ~20x slower than
   release at TU scale - M27 T1a launch-profile finding, 2026-10-05):
   `cargo run --bin cli --features server`

2. **Build the VS Code Extension**:
   ```bash
   cd extensions/ariadne-extension
   npm install
   npm run compile
   ```

3. **Install Extension in VS Code**:
   ```bash
   code extensions/ariadne-extension
   ```

## 🔄 Path Reconciliation

### Local-First Architecture

Since the server runs locally (same machine as the editor), paths are shared. No path reconciliation is needed.

When the extension connects:
1. The server reports its `repo_root` path
2. The extension confirms it matches the workspace
3. Navigation uses server paths directly (they're the same as local paths)

### Why This Works

- **Same repo root**: Direct navigation works perfectly
- **Different repo root**: The extension can still navigate if you've configured `ariadne.serverUrl` to point to the correct server
- **No cross-machine complexity**: Symbols exist because the server analyzed the same code the editor is viewing

### When Server Runs Locally

| Scenario | Server Path | Local Path | Result |
|----------|-------------|------------|--------|
| Same machine | /home/dev/project/src/main.cpp | /home/dev/project/src/main.cpp | ✅ Direct |
| Different drive | C:\project\src\main.cpp | C:\project\src\main.cpp | ✅ Direct |
| Symlinked repo | /real/path/src/main.cpp | /symlink/path/src/main.cpp | ⚠️ Needs config |

### Cross-Machine Note

The local-first approach means the server and editor must be on the same machine. If you need to analyze code from a different machine, you'd need to run the server on that machine and configure the extension URL accordingly. However, this breaks path sharing and requires manual file lookup.

## 🔍 API Endpoints

### Server (crates/ui)

- **GET /api/repo-info** - Repository metadata
- **GET /api/graph/search** - Search for symbols/files
- **GET /api/justification** - Justification for symbol dependencies
- **GET /api/collateral** - Collateral analysis for files

### Extension (ariadne-extension)

- **GET /api/repo-info** - Fetch repo root to verify server
- **GET /api/graph/search** - Search for symbols/files

## 🛤️ Development Workflow

1. **Edit Code** → Analysis Engine processes changes → Rust server computes dependencies → UI updates

2. **Select Symbol in UI** → Extension fetches symbol info → VS Code opens file (same paths, no mapping needed)

3. **Navigate File in VS Code** → Extension queries server → Shows in analysis UI

## 📊 Architecture Diagram

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│    Local IDE    │    │  Extension     │    │    Server       │
│ (VS Code)     │◄──►│  (TypeScript)  │◄──►│  (Rust/Axum)   │
│                 │    │                 │    │                 │
│ • File Editor   │    │ • Path Mapping  │    │ • Analysis      │
│ • Symbol Browser│    │ • Navigation    │    │ • Graph API     │
│ • Settings      │    │ • Config        │    │ • Files         │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 📋 Current Status

✅ **Rust Backend** - Complete with repo-info endpoint
✅ **VS Code Extension** - Implemented with navigation
❌ **Visual Studio Extension** - TBD

## 🔧 Troubleshooting

### Connection Issues

```bash
# Check if server is running
cargo build --release --features server
cargo run --release --bin cli --features server

# Test server health
curl http://localhost:8080/api/repo-info
```

Quick debug alternative (debug indexing measured ~20x slower than release
at TU scale - M27 T1a launch-profile finding, 2026-10-05):
`cargo run --bin cli --features server`

### Path Mismatch

If the server and workspace have different repo roots:

1. Use `ariadne.configureServer` to set the correct URL
2. Or restart the server with the same repo root as your workspace

### Symbol Navigation Not Working

1. Verify server is running
2. Check VS Code extension is installed and enabled
3. Confirm server path is accessible from the extension
4. Review browser console for error messages
