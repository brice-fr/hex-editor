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

  // Keep native menu checkmarks in sync with state.
  // NOTE: the value must be read into a local variable BEFORE the ?. call —
  // optional chaining short-circuits argument evaluation when the object is
  // null, so Svelte would never track the dependency otherwise.
  $effect(() => { const v = showSegmentList;   segmentListMenuItem?.setChecked(v); });
  $effect(() => { const v = showDataInspector; dataInspectorMenuItem?.setChecked(v); });

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
              await PredefinedMenuItem.new({ item: 'CloseWindow' }),
            ],
          }),
          // ③ Edit
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
  :global(:root) {
    --c-bg:        #1e1e1e;
    --c-surface:   #252526;
    --c-raised:    #2d2d2d;
    --c-hover:     #3c3c3c;
    --c-border:    #3a3a3a;
    --c-border2:   #454545;
    --c-text:      #e0e0e0;
    --c-text2:     #ccc;
    --c-muted:     #888;
    --c-dim:       #555;
    --c-accent:    #007acc;
    --c-accent-t:  #9cdcfe;
    --c-accent-b:  #0e639c;
    --c-accent-h:  #1177bb;
    --c-accent-bg: rgba(0,122,204,0.15);
    --c-err:       #f47171;
    --c-err-bg:    rgba(244,71,71,0.15);
    --c-gap-bg:    #1a1a1a;
    --c-gap-text:  #555;
    --c-addr:      #569cd6;
    --c-null:      #3a3a3a;
    --c-null-text: #3a3a3a;
    --c-ec0:       transparent;
    --c-ec1:       rgba(255,255,255,0.03);
    --c-header-bg: #1e1e1e;
    --c-sel:       rgba(58,110,165,0.60);
    --c-sel-text:  #ffffff;
  }

  @media (prefers-color-scheme: light) {
    :global(:root:not([data-theme="dark"])) {
      --c-bg:        #ffffff;
      --c-surface:   #f8f8f8;
      --c-raised:    #f0f0f0;
      --c-hover:     #e8e8e8;
      --c-border:    #e0e0e0;
      --c-border2:   #d4d4d4;
      --c-text:      #1e1e1e;
      --c-text2:     #444;
      --c-muted:     #666;
      --c-dim:       #999;
      --c-accent:    #0070c1;
      --c-accent-t:  #0070c1;
      --c-accent-b:  #0e639c;
      --c-accent-h:  #0080d4;
      --c-accent-bg: rgba(0,112,193,0.10);
      --c-err:       #d32f2f;
      --c-err-bg:    rgba(211,47,47,0.10);
      --c-gap-bg:    #f0f0f0;
      --c-gap-text:  #aaa;
      --c-addr:      #0451a5;
      --c-null:      #e8e8e8;
      --c-null-text: #e8e8e8;
      --c-ec0:       transparent;
      --c-ec1:       rgba(0,0,0,0.03);
      --c-header-bg: #f3f3f3;
      --c-sel:       rgba(0,95,204,0.22);
      --c-sel-text:  #1a1a1a;
    }
  }

  :global(:root[data-theme="light"]) {
    --c-bg:        #ffffff;
    --c-surface:   #f8f8f8;
    --c-raised:    #f0f0f0;
    --c-hover:     #e8e8e8;
    --c-border:    #e0e0e0;
    --c-border2:   #d4d4d4;
    --c-text:      #1e1e1e;
    --c-text2:     #444;
    --c-muted:     #666;
    --c-dim:       #999;
    --c-accent:    #0070c1;
    --c-accent-t:  #0070c1;
    --c-accent-b:  #0e639c;
    --c-accent-h:  #0080d4;
    --c-accent-bg: rgba(0,112,193,0.10);
    --c-err:       #d32f2f;
    --c-err-bg:    rgba(211,47,47,0.10);
    --c-gap-bg:    #f0f0f0;
    --c-gap-text:  #aaa;
    --c-addr:      #0451a5;
    --c-null:      #e8e8e8;
    --c-null-text: #e8e8e8;
    --c-ec0:       transparent;
    --c-ec1:       rgba(0,0,0,0.03);
    --c-header-bg: #f3f3f3;
    --c-sel:       rgba(0,95,204,0.22);
    --c-sel-text:  #1a1a1a;
  }

  :global(*, *::before, *::after) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    background: var(--c-bg);
    color: var(--c-text);
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
