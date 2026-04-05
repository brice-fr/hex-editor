<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  import { onMount, onDestroy } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { PhysicalSize } from '@tauri-apps/api/dpi';
  import { Menu, Submenu, MenuItem, CheckMenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';
  import { open, save, message } from '@tauri-apps/plugin-dialog';
  import { openFile, parseIntelHex, parseSrec, detectFileFormat, saveFile, saveBinary, getStartupFile } from '$lib/api.js';
  import { listen } from '@tauri-apps/api/event';
  import FileMenu from '$lib/components/FileMenu.svelte';
  import HexViewer from '$lib/components/HexViewer.svelte';
  import SegmentList   from '$lib/components/SegmentList.svelte';
  import DataInspector from '$lib/components/DataInspector.svelte';
  import SaveFormatDialog from '$lib/components/SaveFormatDialog.svelte';
  import GoToDialog  from '$lib/components/GoToDialog.svelte';
  import FindDialog  from '$lib/components/FindDialog.svelte';
  import AboutDialog from '$lib/components/AboutDialog.svelte';
  import ImportBinaryDialog from '$lib/components/ImportBinaryDialog.svelte';
  import PreferencesDialog from '$lib/components/PreferencesDialog.svelte';
  import FileAssocDialog from '$lib/components/FileAssocDialog.svelte';
  import CompareDialog  from '$lib/components/CompareDialog.svelte';

  // ── Persistent settings — read synchronously before first render ──────────
  const LS = 'hex-editor.';
  const lsGet = (key, fallback) => { const v = localStorage.getItem(LS + key); return v !== null ? v : fallback; };
  const lsSet = (key, val)      => localStorage.setItem(LS + key, String(val));

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
  let showCompare      = $state(false);
  let compareFile      = $state('');
  let pendingBinaryPath = $state('');
  let hexTopAddress    = $state(0);        // tracks topmost visible address in HexViewer
  let gotoTarget       = $state(null);     // { addr, seq } — seq ensures reactivity on repeat
  let gotoSeq          = 0;

  // ── Display preferences ────────────────────────────────────────────────────
  let fontSize        = $state(parseInt(lsGet('fontSize',    '13')));
  let bytesPerRow     = $state(parseInt(lsGet('bytesPerRow', '16')));
  let theme           = $state(lsGet('theme', 'system'));
  let showPreferences = $state(false);
  let showFileAssoc = $state(false);

  $effect(() => { lsSet('fontSize',    fontSize); });
  $effect(() => { lsSet('bytesPerRow', bytesPerRow); });
  $effect(() => { lsSet('theme',       theme); });

  $effect(() => {
    document.documentElement.setAttribute('data-theme', theme === 'system' ? '' : theme);
  });

  // ── Side-panel visibility — defaults to true on first launch ─────────────
  let showSegmentList   = $state(lsGet('showSegmentList',   'true') === 'true');
  let showDataInspector = $state(lsGet('showDataInspector', 'true') === 'true');

  // Persist panel state immediately whenever it changes
  $effect(() => { lsSet('showSegmentList',   showSegmentList); });
  $effect(() => { lsSet('showDataInspector', showDataInspector); });

  // References to native CheckMenuItems so we can sync their checked state
  let segmentListMenuItem   = null;
  let dataInspectorMenuItem = null;
  let compareMenuItem       = null;

  // Keep native menu checkmarks in sync with state.
  // NOTE: the value must be read into a local variable BEFORE the ?. call —
  // optional chaining short-circuits argument evaluation when the object is
  // null, so Svelte would never track the dependency otherwise.
  $effect(() => { const v = showSegmentList;   segmentListMenuItem?.setChecked(v); });
  $effect(() => { const v = showDataInspector; dataInspectorMenuItem?.setChecked(v); });
  $effect(() => { const v = records.length > 0; compareMenuItem?.setEnabled(v); });

  // ── Data Inspector address — follows scroll unless pinned by a byte click ─
  let inspectorAddress = $state(0);
  let inspectorPinned  = $state(false);

  $effect(() => { if (!inspectorPinned) inspectorAddress = hexTopAddress; });

  function handleByteClick(addr) {
    inspectorAddress = addr;
    inspectorPinned  = true;
  }

  let hexSelection = $state(/** @type {{start:number,end:number,count:number,focus:number}|null} */ (null));
  function handleSelectionChange(sel) {
    hexSelection = sel;
    if (sel) { inspectorAddress = sel.focus; inspectorPinned = true; }
  }

  let unlistenDragDrop;
  let unlistenOpenFile;
  let resizeDebounce = null;

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
    gotoTarget       = { addr, seq: ++gotoSeq };
    inspectorAddress = addr;
    inspectorPinned  = true;
    status           = `Match at 0x${addr.toString(16).toUpperCase().padStart(8, '0')}`;
  }

  function handleGotoOpen() {
    if (records.length === 0) return;
    showGoto = true;
  }

  function handleCompareOpen() {
    if (records.length === 0) return;
    compareFile = '';
    showCompare = true;
  }

  async function handleCompare(refPath, cmpPath) {
    showCompare = false;
    const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
    const label = `compare_${Date.now()}`;
    // Estimate minimum width to show all hex columns without horizontal scrolling.
    // Each byte cell is ~3ch wide; at 12px Cascadia Code 1ch ≈ 7.2px → ~22px per cell.
    const cellPx    = 22;
    const bpr       = bytesPerRow;
    const hexSideW  = bpr * cellPx + 8 + 8;   // bytes + mid-gap + side-padding
    const centerW   = 2 + 90 + 2;              // v-sep + addr-col + v-sep
    const outerPad  = 2 * 12;                  // container left+right padding
    const w = Math.max(700, 2 * hexSideW + centerW + outerPad + 40);
    new WebviewWindow(label, {
      url: `compare?ref=${encodeURIComponent(refPath)}&cmp=${encodeURIComponent(cmpPath)}`,
      title: `Compare — ${refPath.split('/').at(-1)} ↔ ${cmpPath.split('/').at(-1)}`,
      width: w,
      height: 800,
      minWidth: w,
      minHeight: 400,
    });
  }

  function handleGotoConfirm(addr) {
    showGoto         = false;
    gotoTarget       = { addr, seq: ++gotoSeq };
    inspectorAddress = addr;
    inspectorPinned  = true;
    status           = `Navigated to 0x${addr.toString(16).toUpperCase().padStart(8, '0')}`;
  }

  function handleSegmentJump(addr) {
    gotoTarget = { addr, seq: ++gotoSeq };
    status     = `Segment at 0x${addr.toString(16).toUpperCase().padStart(8, '0')}`;
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

      records          = parsed.records;
      inspectorPinned  = false;
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
      records          = [{ record_type: 'Data', address: baseAddr, data: bytes }];
      inspectorPinned  = false;
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

      // ── View menu items — created here so we can call setChecked later ──
      segmentListMenuItem = await CheckMenuItem.new({
        id: 'view-segment-list',
        text: 'Segment List',
        checked: showSegmentList,
        accelerator: 'CmdOrCtrl+Shift+L',
        action: () => { showSegmentList = !showSegmentList; },
      });
      dataInspectorMenuItem = await CheckMenuItem.new({
        id: 'view-data-inspector',
        text: 'Data Inspector',
        checked: showDataInspector,
        accelerator: 'CmdOrCtrl+Shift+I',
        action: () => { showDataInspector = !showDataInspector; },
      });

      const aboutItem = await MenuItem.new({
        id: 'about',
        text: 'About Hex Editor',
        action: () => (showAbout = true),
      });

      const preferencesItem = await MenuItem.new({
        id: 'preferences',
        text: 'Preferences…',
        accelerator: 'CmdOrCtrl+,',
        action: () => { showPreferences = true; },
      });

      const fileAssocItem = await MenuItem.new({
        id: 'file-associations',
        text: 'File Associations…',
        action: () => { showFileAssoc = true; },
      });

      const menu = await Menu.new({
        items: [
          // ① App menu — macOS only (Services / Hide / Quit)
          ...(isMac ? [
            await Submenu.new({
              text: 'Hex Editor',
              items: [
                preferencesItem,
                aboutItem,
                await PredefinedMenuItem.new({ item: 'Separator' }),
                await PredefinedMenuItem.new({ item: 'Services' }),
                await PredefinedMenuItem.new({ item: 'Separator' }),
                await PredefinedMenuItem.new({ item: 'Hide' }),
                await PredefinedMenuItem.new({ item: 'HideOthers' }),
                await PredefinedMenuItem.new({ item: 'ShowAll' }),
                await PredefinedMenuItem.new({ item: 'Separator' }),
                await PredefinedMenuItem.new({ item: 'Quit' }),
              ],
            }),
          ] : []),
          // ② File
          await Submenu.new({
            text: 'File',
            items: [
              await MenuItem.new({ id: 'open', text: 'Open…', accelerator: 'CmdOrCtrl+O', action: handleOpen }),
              await MenuItem.new({ id: 'save-as', text: 'Save as…', accelerator: 'CmdOrCtrl+Shift+S', action: handleSave }),
              await MenuItem.new({ id: 'import-binary', text: 'Import Binary…', accelerator: 'CmdOrCtrl+B', action: handleImportBinaryOpen }),
              await PredefinedMenuItem.new({ item: 'Separator' }),
              (compareMenuItem = await MenuItem.new({ id: 'compare', text: 'Compare with…', enabled: false, action: handleCompareOpen })),
              await PredefinedMenuItem.new({ item: 'Separator' }),
              await PredefinedMenuItem.new({ item: 'CloseWindow' }),
            ],
          }),
          // ④ Search
          await Submenu.new({
            text: 'Search',
            items: [
              await MenuItem.new({ id: 'find', text: 'Find…', accelerator: 'CmdOrCtrl+F', action: handleFindOpen }),
              await PredefinedMenuItem.new({ item: 'Separator' }),
              await MenuItem.new({ id: 'goto-address', text: 'Go to Address…', accelerator: 'CmdOrCtrl+G', action: handleGotoOpen }),
            ],
          }),
          // ⑤ View — toggle side panels + native fullscreen
          await Submenu.new({
            text: 'View',
            items: [
              segmentListMenuItem,
              dataInspectorMenuItem,
              await PredefinedMenuItem.new({ item: 'Separator' }),
              await PredefinedMenuItem.new({ item: 'Fullscreen' }),
              ...(!isMac ? [
                await PredefinedMenuItem.new({ item: 'Separator' }),
                preferencesItem,
                await PredefinedMenuItem.new({ item: 'Separator' }),
                fileAssocItem,
              ] : [
                await PredefinedMenuItem.new({ item: 'Separator' }),
                fileAssocItem,
              ]),
            ],
          }),
          // ⑥ Help — Windows / Linux only (About lives here)
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

    // Window size save/restore is handled by tauri-plugin-window-state on the
    // Rust side — no JS code needed here.

    // ── Drag-and-drop support ──
    try {
      unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === 'drop') {
          isDragging = false;
          const paths = event.payload.paths;
          if (paths.length > 0) {
            if (showCompare) compareFile = paths[0];
            else handleOpenPath(paths[0]);
          }
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

    // ── OS file-association open ──────────────────────────────────────────────
    // Warm launch: app already running, macOS sends the file via an event
    try {
      unlistenOpenFile = await listen('open-file', (event) => {
        handleOpenPath(event.payload);
      });
    } catch (e) {
      console.warn('open-file listener failed:', e);
    }
    // Cold launch: path was captured by Rust before the webview was ready
    try {
      const startupPath = await getStartupFile();
      if (startupPath) handleOpenPath(startupPath);
    } catch (e) {
      console.warn('get_startup_file failed:', e);
    }
  });

  onDestroy(() => {
    if (unlistenDragDrop) unlistenDragDrop();
    if (unlistenOpenFile) unlistenOpenFile();
    clearTimeout(resizeDebounce);
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

<PreferencesDialog
  open={showPreferences}
  {fontSize}
  {bytesPerRow}
  {theme}
  onFontSize={(n) => { fontSize = n; }}
  onBytesPerRow={(n) => { bytesPerRow = n; }}
  onTheme={(t) => { theme = t; }}
  onClose={() => { showPreferences = false; }}
/>

<FileAssocDialog open={showFileAssoc} onClose={() => { showFileAssoc = false; }} />

<CompareDialog
  open={showCompare}
  referenceFile={currentFile}
  bind:comparedFile={compareFile}
  {isDragging}
  onCompare={handleCompare}
  onCancel={() => { showCompare = false; }}
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
  <FileMenu onOpen={handleOpen} onSave={handleSave} onFind={handleFindOpen} onGoto={handleGotoOpen} onCompare={handleCompareOpen} {loading} {saving} hasFile={records.length > 0} />

  <div class="content-area">
    <main class="viewer-area">
      <HexViewer
        {records}
        {bytesPerRow}
        {fontSize}
        {gotoTarget}
        onScrolled={() => { if (!loading && !saving) status = ''; }}
        onTopAddress={(addr) => { hexTopAddress = addr; }}
        onByteClick={handleByteClick}
        onSelectionChange={handleSelectionChange}
      />
    </main>

    {#if showSegmentList || showDataInspector}
      <aside class="side-panel">
        {#if showSegmentList}
          <div class="side-section">
            <SegmentList {records} topAddress={hexTopAddress} onJump={handleSegmentJump} onClose={() => (showSegmentList = false)} />
          </div>
        {/if}
        {#if showSegmentList && showDataInspector}
          <div class="side-divider"></div>
        {/if}
        {#if showDataInspector}
          <div class="side-section">
            <DataInspector
              {records}
              address={inspectorAddress}
              pinned={inspectorPinned}
              onUnpin={() => (inspectorPinned = false)}
              onClose={() => (showDataInspector = false)}
            />
          </div>
        {/if}
      </aside>
    {/if}
  </div>

  <footer class="statusbar">
    {#if status}
      <span>{status}</span>
    {:else if !currentFile}
      <span class="hint">Open a HEX, S-record or Binary file to get started</span>
    {/if}
    {#if hexSelection}
      <span class="sel-info">
        Sel&nbsp;{hexSelection.start.toString(16).padStart(8,'0').toUpperCase()}
        –&nbsp;{hexSelection.end.toString(16).padStart(8,'0').toUpperCase()}
        &nbsp;·&nbsp;{hexSelection.count.toLocaleString()} byte{hexSelection.count === 1 ? '' : 's'}
      </span>
    {/if}
  </footer>
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  /* ── Content area: hex viewer + optional side panel ── */
  .content-area {
    flex: 1;
    display: flex;
    flex-direction: row;
    overflow: hidden;
    min-height: 0;
  }

  .viewer-area {
    flex: 1;
    overflow: hidden;
    min-width: 0;
  }

  /* ── Side panel ── */
  .side-panel {
    width: 280px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    border-left: 1px solid var(--c-hover);
    overflow: hidden;
    background: var(--c-bg);
  }

  .side-section {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .side-divider {
    height: 1px;
    flex-shrink: 0;
    background: var(--c-hover);
  }

  .statusbar {
    display: flex;
    align-items: center;
    height: 22px;
    padding: 0 10px;
    background: var(--c-accent);
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

  .statusbar .sel-info {
    margin-left: auto;
    opacity: 0.9;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.02em;
  }

  /* Drag-and-drop overlay */
  .drop-overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
    background: var(--c-accent-bg);
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
    border: 2.5px dashed var(--c-accent);
    border-radius: 16px;
    color: var(--c-accent);
    background: var(--c-accent-bg);
  }

  .drop-card svg {
    width: 56px;
    height: 56px;
  }

  .drop-card p {
    font-size: 18px;
    font-weight: 600;
    letter-spacing: 0.01em;
    color: var(--c-accent);
  }
</style>
