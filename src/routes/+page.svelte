<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  import { onMount, onDestroy } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { Menu, Submenu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';
  import { open, save, message } from '@tauri-apps/plugin-dialog';
  import { openFile, parseIntelHex, parseSrec, detectFileFormat, saveFile, saveBinary } from '$lib/api.js';
  import FileMenu from '$lib/components/FileMenu.svelte';
  import HexViewer from '$lib/components/HexViewer.svelte';
  import SaveFormatDialog from '$lib/components/SaveFormatDialog.svelte';
  import GoToDialog  from '$lib/components/GoToDialog.svelte';
  import FindDialog  from '$lib/components/FindDialog.svelte';
  import AboutDialog from '$lib/components/AboutDialog.svelte';
  import ImportBinaryDialog from '$lib/components/ImportBinaryDialog.svelte';

  let records       = $state([]);
  let currentFile   = $state('');
  let currentFormat = $state('');
  let status        = $state('');
  let loading       = $state(false);
  let saving           = $state(false);
  let showFormatPicker = $state(false);
  let showGoto         = $state(false);
  let showFind         = $state(false);
  let showAbout        = $state(false);
  let isDragging       = $state(false);
  let showImportBinary = $state(false);
  let pendingBinaryPath = $state('');
  let hexTopAddress    = $state(0);        // tracks topmost visible address in HexViewer
  let gotoTarget       = $state(null);     // { addr, seq } — seq ensures reactivity on repeat
  let gotoSeq          = 0;

  let unlistenDragDrop;

  // Address range of the loaded file (for GoToDialog validation)
  const addrRange = $derived((() => {
    let min = Infinity, max = -Infinity;
    for (const r of records) {
      const isData = r.record_type === 'Data' || r.record_type === 'S1'
                  || r.record_type === 'S2'   || r.record_type === 'S3';
      if (!isData || r.data.length === 0) continue;
      if (r.address < min) min = r.address;
      if (r.address + r.data.length - 1 > max) max = r.address + r.data.length - 1;
    }
    return min === Infinity ? { min: 0, max: 0 } : { min, max };
  })());

  function handleFindOpen() {
    if (records.length === 0) return;
    showFind = true;
  }

  function handleFindNavigate(addr) {
    gotoTarget = { addr, seq: ++gotoSeq };
    status     = `Match at 0x${addr.toString(16).toUpperCase().padStart(8, '0')}`;
  }

  function handleGotoOpen() {
    if (records.length === 0) return;
    showGoto = true;
  }

  function handleGotoConfirm(addr) {
    showGoto    = false;
    gotoTarget  = { addr, seq: ++gotoSeq };
    status      = `Navigated to 0x${addr.toString(16).toUpperCase().padStart(8, '0')}`;
  }

  // ── Shared open logic — called by both dialog and drag-drop ──────────────
  async function handleOpenPath(path) {
    loading = true;
    status  = 'Loading…';
    try {
      const format = await detectFileFormat(path);

      if (format === 'binary') {
        // Show the import dialog to ask for base address
        pendingBinaryPath = path;
        showImportBinary  = true;
        loading = false;
        return;
      }

      const bytes = await openFile(path);

      let parsed;
      if (format === 'ihex') {
        parsed = JSON.parse(await parseIntelHex(bytes));
      } else if (format === 'srec') {
        parsed = JSON.parse(await parseSrec(bytes));
      } else {
        await message(`Unsupported format: ${format}`, { kind: 'error', title: 'Cannot open file' });
        loading = false;
        return;
      }

      records       = parsed.records;
      currentFile   = path;
      currentFormat = format;
      const fileName = path.split('/').at(-1);
      await getCurrentWindow().setTitle(`Hex Editor — ${fileName}`);

      let statusMsg = `Loaded ${parsed.total_data_bytes.toLocaleString()} bytes · ${format.toUpperCase()}`;
      if (parsed.checksum_warnings > 0) {
        statusMsg += ` · ⚠ ${parsed.checksum_warnings} checksum error${parsed.checksum_warnings > 1 ? 's' : ''} corrected`;
      }
      status = statusMsg;
    } catch (err) {
      await message(String(err), { kind: 'error', title: 'Cannot open file' });
    } finally {
      loading = false;
    }
  }

  // ── Open via file dialog ─────────────────────────────────────────────────
  async function handleOpen() {
    const selected = await open({
      multiple: false,
      filters: [
        { name: 'Firmware files', extensions: ['hex', 'ihex', 'srec', 'mot', 's19', 's28', 's37', 'bin'] },
        { name: 'All files', extensions: ['*'] },
      ],
    });

    if (!selected) return; // user cancelled
    await handleOpenPath(selected);
  }

  // ── Import Binary — opens dialog filtered to .bin only ───────────────────
  async function handleImportBinaryOpen() {
    const selected = await open({
      multiple: false,
      filters: [
        { name: 'Binary files', extensions: ['bin'] },
        { name: 'All files', extensions: ['*'] },
      ],
    });
    if (!selected) return;
    await handleOpenPath(selected);
  }

  // ── Called when user confirms the import binary dialog ───────────────────
  async function handleImportBinary(baseAddr) {
    showImportBinary = false;
    const path = pendingBinaryPath;
    pendingBinaryPath = '';
    loading = true;
    status  = 'Loading…';
    try {
      const bytes = await openFile(path);
      records       = [{ record_type: 'Data', address: baseAddr, data: bytes }];
      currentFile   = path;
      currentFormat = 'binary';
      const fileName = path.split('/').at(-1);
      await getCurrentWindow().setTitle(`Hex Editor — ${fileName}`);
      status = `Loaded ${bytes.length.toLocaleString()} bytes · Binary @ 0x${baseAddr.toString(16).toUpperCase().padStart(8, '0')}`;
    } catch (err) {
      await message(String(err), { kind: 'error', title: 'Cannot open file' });
    } finally {
      loading = false;
    }
  }

  // ── Save as — step 1: open the format picker modal ───────────────────────
  function handleSave() {
    showFormatPicker = true;
  }

  // ── Save as — step 2: format chosen → open native file-save dialog ───────
  async function handleFormatPicked({ fmt, fillByte = 0xFF }) {
    showFormatPicker = false;

    const stem = currentFile
      ? currentFile.replace(/\.[^/.]+$/, '')   // strip original extension
      : 'output';

    if (fmt === 'binary') {
      const dest = await save({
        filters: [{ name: 'Binary', extensions: ['bin'] }],
        defaultPath: stem + '.bin',
      });
      if (!dest) return;

      saving = true;
      status  = 'Saving…';
      try {
        const sizeBytes = await saveBinary(records, dest, fillByte);
        const name = dest.split('/').at(-1);
        status = `Saved ${name} · Binary (fill=0x${fillByte.toString(16).toUpperCase().padStart(2, '0')}) · ${(sizeBytes / 1024).toFixed(1)} KB`;
      } catch (err) {
        await message(String(err), { kind: 'error', title: 'Cannot save file' });
      } finally {
        saving = false;
      }
      return;
    }

    const filters = fmt === 'ihex'
      ? [{ name: 'Intel HEX',         extensions: ['hex', 'ihex'] }]
      : [{ name: 'Motorola S-record', extensions: ['srec', 'mot', 's19', 's28', 's37'] }];

    const defaultExt = fmt === 'ihex' ? '.hex' : '.srec';

    const dest = await save({ filters, defaultPath: stem + defaultExt });
    if (!dest) return; // user cancelled

    saving = true;
    status  = 'Saving…';
    try {
      await saveFile(records, dest, fmt);
      const name = dest.split('/').at(-1);
      status = `Saved ${name} · ${fmt.toUpperCase()}`;
    } catch (err) {
      await message(String(err), { kind: 'error', title: 'Cannot save file' });
    } finally {
      saving = false;
    }
  }

  // ── Native macOS menu bar ──
  onMount(async () => {
    try {
      const isMac = /Mac/i.test(navigator.platform);

      const aboutItem = await MenuItem.new({
        id: 'about',
        text: 'About Hex Editor',
        action: () => (showAbout = true),
      });

      const menu = await Menu.new({
        items: [
          // ① App menu — About lives here on macOS, in Help on other OSes
          await Submenu.new({
            text: 'Hex Editor',
            items: [
              ...(isMac ? [aboutItem, await PredefinedMenuItem.new({ item: 'Separator' })] : []),
              await PredefinedMenuItem.new({ item: 'Services' }),
              await PredefinedMenuItem.new({ item: 'Separator' }),
              await PredefinedMenuItem.new({ item: 'Hide' }),
              await PredefinedMenuItem.new({ item: 'HideOthers' }),
              await PredefinedMenuItem.new({ item: 'ShowAll' }),
              await PredefinedMenuItem.new({ item: 'Separator' }),
              await PredefinedMenuItem.new({ item: 'Quit' }),
            ],
          }),
          // ② File menu
          await Submenu.new({
            text: 'File',
            items: [
              await MenuItem.new({
                id: 'open',
                text: 'Open…',
                accelerator: 'CmdOrCtrl+O',
                action: handleOpen,
              }),
              await MenuItem.new({
                id: 'save-as',
                text: 'Save as…',
                accelerator: 'CmdOrCtrl+Shift+S',
                action: handleSave,
              }),
              await MenuItem.new({
                id: 'import-binary',
                text: 'Import Binary…',
                accelerator: 'CmdOrCtrl+B',
                action: handleImportBinaryOpen,
              }),
              await PredefinedMenuItem.new({ item: 'Separator' }),
              await PredefinedMenuItem.new({ item: 'CloseWindow' }),
            ],
          }),
          // ③ Search menu
          await Submenu.new({
            text: 'Search',
            items: [
              await MenuItem.new({
                id: 'find',
                text: 'Find…',
                accelerator: 'CmdOrCtrl+F',
                action: handleFindOpen,
              }),
              await PredefinedMenuItem.new({ item: 'Separator' }),
              await MenuItem.new({
                id: 'goto-address',
                text: 'Go to Address…',
                accelerator: 'CmdOrCtrl+G',
                action: handleGotoOpen,
              }),
            ],
          }),
          // ④ Edit menu (copy / paste / select-all for the hex viewer)
          await Submenu.new({
            text: 'Edit',
            items: [
              await PredefinedMenuItem.new({ item: 'Undo' }),
              await PredefinedMenuItem.new({ item: 'Redo' }),
              await PredefinedMenuItem.new({ item: 'Separator' }),
              await PredefinedMenuItem.new({ item: 'Cut' }),
              await PredefinedMenuItem.new({ item: 'Copy' }),
              await PredefinedMenuItem.new({ item: 'Paste' }),
              await PredefinedMenuItem.new({ item: 'SelectAll' }),
            ],
          }),
          // ⑤ Help menu — About lives here on Windows / Linux
          ...(!isMac ? [
            await Submenu.new({
              text: 'Help',
              items: [aboutItem],
            }),
          ] : []),
        ],
      });

      await menu.setAsAppMenu();
    } catch (err) {
      // Non-fatal: native menu is best-effort
      console.warn('Could not build native menu:', err);
    }

    // ── Drag-and-drop support ──
    try {
      unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === 'drop') {
          isDragging = false;
          const paths = event.payload.paths;
          if (paths.length > 0) handleOpenPath(paths[0]);
        } else if (event.payload.type === 'enter') {
          isDragging = true;
        } else if (event.payload.type === 'leave') {
          isDragging = false;
        }
      });
      document.addEventListener('dragover', (e) => e.preventDefault(), { passive: false });
      document.addEventListener('drop', (e) => e.preventDefault(), { passive: false });
    } catch (e) {
      console.warn('Drag-drop setup failed:', e);
    }
  });

  onDestroy(() => {
    if (unlistenDragDrop) unlistenDragDrop();
  });
</script>

<AboutDialog open={showAbout} onClose={() => (showAbout = false)} />

<FindDialog
  open={showFind}
  {records}
  topAddress={hexTopAddress}
  onNavigate={handleFindNavigate}
  onClose={() => (showFind = false)}
/>

<GoToDialog
  open={showGoto}
  prefill={hexTopAddress}
  minAddr={addrRange.min}
  maxAddr={addrRange.max}
  onGoto={handleGotoConfirm}
  onCancel={() => (showGoto = false)}
/>

<SaveFormatDialog
  open={showFormatPicker}
  onPick={handleFormatPicked}
  onCancel={() => (showFormatPicker = false)}
/>

<ImportBinaryDialog
  open={showImportBinary}
  onImport={handleImportBinary}
  onCancel={() => { showImportBinary = false; pendingBinaryPath = ''; }}
/>

{#if isDragging}
  <div class="drop-overlay">
    <div class="drop-card">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" fill="none"
           stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <!-- Box bottom -->
        <rect x="8" y="28" width="32" height="12" rx="3"/>
        <!-- Arrow downward into box -->
        <line x1="24" y1="8" x2="24" y2="28"/>
        <polyline points="16,20 24,28 32,20"/>
      </svg>
      <p>Drop to open</p>
    </div>
  </div>
{/if}

<div class="app-shell" onclick={() => { if (!loading && !saving) status = ''; }}>
  <!-- Toolbar: open + save icons -->
  <FileMenu onOpen={handleOpen} onSave={handleSave} onFind={handleFindOpen} onGoto={handleGotoOpen} {loading} {saving} hasFile={records.length > 0} />

  <main class="viewer-area">
    <HexViewer
      {records}
      {gotoTarget}
      onScrolled={() => { if (!loading && !saving) status = ''; }}
      onTopAddress={(addr) => { hexTopAddress = addr; }}
    />
  </main>

  <footer class="statusbar">
    {#if status}
      <span>{status}</span>
    {:else if !currentFile}
      <span class="hint">Open a HEX, S-record or Binary file to get started</span>
    {/if}
  </footer>
</div>

<style>
  :global(*, *::before, *::after) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    background: #1e1e1e;
    color: #d4d4d4;
    overflow: hidden;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI',
                 'Helvetica Neue', Arial, sans-serif;
    font-feature-settings: 'cv02', 'cv03', 'cv04', 'tnum';
    -webkit-font-smoothing: antialiased;
  }

  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .viewer-area {
    flex: 1;
    overflow: hidden;
  }

  .statusbar {
    display: flex;
    align-items: center;
    height: 22px;
    padding: 0 10px;
    background: #007acc;
    color: #fff;
    font-size: 11.5px;
    font-weight: 400;
    letter-spacing: 0.01em;
    user-select: none;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 0;
  }

  .statusbar .hint {
    opacity: 0.75;
  }

  /* Drag-and-drop overlay */
  .drop-overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
    background: rgba(0, 122, 204, 0.18);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
  }

  .drop-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    padding: 36px 48px;
    border: 2.5px dashed #007acc;
    border-radius: 16px;
    color: #007acc;
    background: rgba(0, 122, 204, 0.08);
  }

  .drop-card svg {
    width: 56px;
    height: 56px;
  }

  .drop-card p {
    font-size: 18px;
    font-weight: 600;
    letter-spacing: 0.01em;
    color: #007acc;
  }

  @media (prefers-color-scheme: light) {
    :global(body) { background: #fff; color: #1e1e1e; }
    .statusbar { background: #005f9e; }
    .drop-card { border-color: #005f9e; color: #005f9e; background: rgba(0, 95, 158, 0.08); }
    .drop-card p { color: #005f9e; }
  }
</style>
