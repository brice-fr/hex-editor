# Hex Editor

A fast, cross-platform hex editor for **Intel HEX** and **Motorola S-record** files, built with Tauri 2, SvelteKit, and Rust.

---

## Features

### File Handling
- **Open** Intel HEX (`.hex`, `.ihex`) and Motorola S-record (`.srec`, `.mot`, `.s19`, `.s28`, `.s37`) files via native OS file picker
- **Save As** to either format with automatic format conversion — choose Intel HEX or S-record at save time regardless of the source format
- Format auto-detection from file extension and magic bytes
- Graceful handling of non-standard S6 records (silently ignored)

### Hex Viewer
- **Virtual scrolling** — renders only visible rows, handles files of any size without performance degradation
- 16 bytes per row with address column, hex byte columns, and ASCII representation
- Alternating column shading for readability
- Solid separator lines between address / hex / ASCII columns
- Row hover highlighting
- Non-printable bytes rendered as `.`

### Search & Navigation
- **Find** panel (⌘F / Ctrl+F) — floating, movable, non-blocking
  - **Text search** with case-sensitive option
  - **Hex pattern search** (e.g. `DE AD BE EF`)
  - Forward / Backward direction with Wrap Around
  - **Find** / **Find Next** for sequential navigation
  - **Find All** — lists every match address; click any entry to navigate
- **Go to Address** (⌘G / Ctrl+G) — jump to any hex address in the loaded file

### User Interface
- Native **macOS menu bar** with File, Search, Edit menus
- **Toolbar** with icon buttons: Open, Save As, Find, Go to Address
- **Status bar** at bottom — loading progress, navigation results, errors as native OS dialogs
- **OS window title** updated with the currently open filename
- **About dialog** with app icon, version, and copyright

### Cross-Platform
- macOS `.app` + `.dmg` (Apple Silicon)
- Windows `.msi` + `.exe` (via GitHub Actions Windows runner)
- Linux `.deb` / `.AppImage` (via GitHub Actions Ubuntu runner)
- Full icon set: `.icns` (macOS), `.ico` (Windows, 7 sizes), `.png` (Linux)

---

## Development Toolchain

| Component | Technology | Version |
|-----------|-----------|---------|
| App framework | [Tauri 2](https://tauri.app) | 2.x |
| Frontend | [SvelteKit](https://kit.svelte.dev) + [Svelte 5](https://svelte.dev) | 5.x |
| Build tool | [Vite](https://vitejs.dev) | 6.x |
| Backend / commands | [Rust](https://rust-lang.org) | stable (1.94+) |
| IHex parsing | [`ihex`](https://crates.io/crates/ihex) crate | 3.0 |
| File I/O | [`memmap2`](https://crates.io/crates/memmap2) crate | 0.9 |
| Serialisation | [`serde`](https://crates.io/crates/serde) + `serde_json` | 1.0 |
| Native dialogs | `@tauri-apps/plugin-dialog` | 2.x |
| Package manager | npm | 11.x |

---

## Getting Started

### Prerequisites

| Tool | macOS | Windows | Linux |
|------|-------|---------|-------|
| Rust toolchain | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | [rustup.rs](https://rustup.rs) | same |
| Node.js LTS | [nvm](https://github.com/nvm-sh/nvm) or [nodejs.org](https://nodejs.org) | [nvm-windows](https://github.com/coreybutler/nvm-windows) | nvm |
| Xcode CLT | `xcode-select --install` | — | — |
| WebView2 | — | pre-installed on Win 10/11 | — |
| `webkit2gtk` | — | — | `sudo apt install libwebkit2gtk-4.1-dev` |

### Run in development mode

```bash
git clone https://github.com/brice-dev/hex-editor.git
cd hex-editor
npm install
npm run tauri dev
```

Hot-reload is active for both the Svelte frontend and Rust backend.

### Build a release

**macOS DMG:**
```bash
npm run tauri build
# → src-tauri/target/release/bundle/dmg/hex-editor_0.2.0_aarch64.dmg
```

**Windows MSI** (requires Windows or GitHub Actions):
```bash
npm run tauri build
# → src-tauri/target/release/bundle/msi/hex-editor_0.2.0_x64_en-US.msi
```

### Automated releases via GitHub Actions

Push a version tag to trigger a multi-platform build:

```bash
git tag v0.2.0
git push origin v0.2.0
```

The workflow (`.github/workflows/release.yml`) builds macOS and Windows bundles and publishes them as GitHub Release assets automatically.

---

## Project Structure

```
hex-editor/
├── src/                          # SvelteKit frontend
│   ├── lib/
│   │   ├── api.js                # Tauri invoke abstraction layer
│   │   └── components/
│   │       ├── HexViewer.svelte  # Virtual-scrolling hex display
│   │       ├── FileMenu.svelte   # Toolbar icon buttons
│   │       ├── FindDialog.svelte # Floating search panel
│   │       ├── GoToDialog.svelte # Go-to-address modal
│   │       ├── SaveFormatDialog.svelte  # Format picker modal
│   │       └── AboutDialog.svelte       # About modal
│   └── routes/
│       └── +page.svelte          # App shell and native menu
├── src-tauri/
│   ├── src/
│   │   ├── main.rs               # Entry point
│   │   ├── lib.rs                # Tauri builder + plugin registration
│   │   ├── commands.rs           # Tauri command handlers
│   │   ├── file_operations.rs    # File I/O, IHex & SREC writers
│   │   ├── hex_parser.rs         # Intel HEX parser
│   │   └── srec_parser.rs        # Motorola S-record parser
│   ├── icons/                    # Full icon set (icns, ico, png)
│   ├── capabilities/             # Tauri ACL permissions
│   └── tauri.conf.json           # App configuration
├── static/                       # Web-accessible static assets
├── .github/workflows/            # CI/CD release pipeline
├── LICENSE                       # MIT
└── README.md
```

---

## License

This project is released under the **MIT License** — see [LICENSE](LICENSE) for full text.

```
SPDX-License-Identifier: MIT
Copyright (c) 2026 Brice LECOLE
```
