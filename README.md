# Hex Editor

A fast, cross-platform hex editor for **Intel HEX**, **Motorola S-record**, and **raw binary** files, built with Tauri 2, SvelteKit, and Rust.

---

## Features

### File Handling
- **Open** Intel HEX (`.hex`, `.ihex`), Motorola S-record (`.srec`, `.mot`, `.s19`, `.s28`, `.s37`), and raw binary (`.bin`) files via native OS file picker
- **Drag and drop** a file directly onto the window to open it
- **Save As** to Intel HEX or S-record with automatic format conversion; export to flat binary with configurable fill byte
- **Import Binary** — load a raw `.bin` file at a user-specified base address
- Format auto-detection from file extension and magic bytes
- **OS file-association open** — double-clicking an associated file in the OS file manager launches the app and loads the file directly

### Hex Viewer
- **Virtual scrolling** — renders only visible rows, handles files of any size without performance degradation
- Configurable **bytes per row**: 8, 16, or 32
- Address column, hex byte columns (with mid-row gap at byte 8), and ASCII representation
- Alternating column shading and row hover highlighting; clickable bytes pin the Data Inspector
- Non-printable bytes rendered as `.`
- **Segment boundary visualisation** — non-contiguous memory regions are separated by a gap row showing the gap size and address range; leading blank cells show where a segment starts mid-row

### Search & Navigation
- **Find** panel (⌘F / Ctrl+F) — floating, draggable, non-blocking
  - **Text search** with case-sensitive option
  - **Hex pattern search** (e.g. `DE AD BE EF`)
  - Forward / Backward direction with Wrap Around
  - **Find** / **Find Next** for sequential navigation
  - **Find All** — lists every match address; click any entry to navigate
- **Go to Address** (⌘G / Ctrl+G) — jump to any hex address in the loaded file

### Side Panels (View menu or ⌘⇧L / ⌘⇧I)
- **Segment List** — lists all non-contiguous memory segments with start address, end address, and size; click a row to scroll the viewer to that segment
- **Data Inspector** — displays the bytes at the current address decoded as u8, i8, u16/u32/u64 (LE & BE), f32/f64 (LE & BE); address follows the scroll position or is pinned by clicking a byte
- Both panels are independently togglable; their visibility is persisted across sessions

### Preferences (⌘,)
- **Theme**: System / Dark / Light (CSS custom-property based, applied globally)
- **Font size**: 10 – 20 px slider
- **Bytes per row**: 8, 16, or 32

### OS File Associations
- **Build-time** associations for Intel HEX and S-record extensions (registered via OS installer / `Info.plist`)
- **Runtime dialog** (View → File Associations…) — shows the current association status for each extension including `.bin`, with checkboxes and an Apply button
  - Windows: `HKCU\Software\Classes` registry + `SHChangeNotify`
  - macOS: Launch Services `LSSetDefaultRoleHandlerForContentType` (association only; deassociation not supported by the OS API)
  - Linux: `xdg-mime default`

### User Interface
- Native **macOS menu bar** — File, Edit, Search, View (Segment List, Data Inspector, Preferences, File Associations, Full Screen); app menu on macOS only, Help menu on Windows/Linux
- **Toolbar** with icon buttons: Open, Save As, Find, Go to Address
- **Status bar** — loading progress, navigation results; errors shown as native OS dialogs
- **Window size and position** persisted across sessions via `tauri-plugin-window-state`; default launch size 925 × 460
- OS window title updated with the currently open filename

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
| Window state | `tauri-plugin-window-state` | 2.x |
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
# → src-tauri/target/release/bundle/dmg/hex-editor_0.2.1_aarch64.dmg
```

**Windows MSI** (requires Windows or GitHub Actions):
```bash
npm run tauri build
# → src-tauri/target/release/bundle/msi/hex-editor_0.2.1_x64_en-US.msi
```

### Automated releases via GitHub Actions

Push a version tag to trigger a multi-platform build:

```bash
git tag v0.2.1
git push origin v0.2.1
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
│   │       ├── HexViewer.svelte        # Virtual-scrolling hex display
│   │       ├── FileMenu.svelte         # Toolbar icon buttons
│   │       ├── FindDialog.svelte       # Floating search panel
│   │       ├── GoToDialog.svelte       # Go-to-address modal
│   │       ├── SaveFormatDialog.svelte # Format picker modal
│   │       ├── AboutDialog.svelte      # About modal
│   │       ├── SegmentList.svelte      # Segment list side panel
│   │       ├── DataInspector.svelte    # Data inspector side panel
│   │       ├── PreferencesDialog.svelte# Preferences modal
│   │       └── FileAssocDialog.svelte  # File associations modal
│   └── routes/
│       └── +page.svelte          # App shell and native menu
├── src-tauri/
│   ├── src/
│   │   ├── main.rs               # Entry point
│   │   ├── lib.rs                # Tauri builder + plugin registration
│   │   ├── commands.rs           # Tauri command handlers
│   │   ├── file_operations.rs    # File I/O, IHex & SREC writers
│   │   ├── hex_parser.rs         # Intel HEX parser
│   │   ├── srec_parser.rs        # Motorola S-record parser
│   │   └── file_assoc.rs         # OS file association management
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
