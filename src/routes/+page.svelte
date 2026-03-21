<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Menu, Submenu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';
  import { open, save, message } from '@tauri-apps/plugin-dialog';
  import { openFile, parseIntelHex, parseSrec, detectFileFormat, saveFile } from '$lib/api.js';
  import FileMenu from '$lib/components/FileMenu.svelte';
  import HexViewer from '$lib/components/HexViewer.svelte';
  import SaveFormatDialog from '$lib/components/SaveFormatDialog.svelte';
  import GoToDialog  from '$lib/components/GoToDialog.svelte';
  import FindDialog  from '$lib/components/FindDialog.svelte';
  import AboutDialog from '$lib/components/AboutDialog.svelte';

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
  let hexTopAddress    = $state(0);        // tracks topmost visible address in HexViewer
  let gotoTarget       = $state(null);     // { addr, seq } — seq ensures reactivity on repeat
  let gotoSeq          = 0;

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

  // ── Shared open logic (toolbar icon + native menu item both call this) ──
  async function handleOpen() {
    const selected = await open({
      multiple: false,
      filters: [
        { name: 'HEX / S-record files', extensions: ['hex', 'ihex', 'srec', 'mot', 's19', 's28', 's37'] },
        { name: 'All files', extensions: ['*'] },
      ],
    });

    if (!selected) return; // user cancelled

    loading = true;
    status  = 'Loading…';
    try {
      const format = await detectFileFormat(selected);
      const bytes  = await openFile(selected);

      let parsed;
      if (format === 'ihex') {
        parsed = JSON.parse(await parseIntelHex(bytes));
      } else if (format === 'srec') {
        parsed = JSON.parse(await parseSrec(bytes));
      } else {
        await message(`Unsupported format: ${format}`, { kind: 'error', title: 'Cannot open file' });
        return;
      }

      records       = parsed.records;
      currentFile   = selected;
      currentFormat = format;
      const fileName = selected.split('/').at(-1);
      await getCurrentWindow().setTitle(`Hex Editor — ${fileName}`);
      status = `Loaded ${parsed.total_data_bytes.toLocaleString()} bytes · ${format.toUpperCase()}`;
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
  async function handleFormatPicked(fmt) {
    showFormatPicker = false;

    const filters = fmt === 'ihex'
      ? [{ name: 'Intel HEX',         extensions: ['hex', 'ihex'] }]
      : [{ name: 'Motorola S-record', extensions: ['srec', 'mot', 's19', 's28', 's37'] }];

    const defaultExt = fmt === 'ihex' ? '.hex' : '.srec';
    const stem = currentFile
      ? currentFile.replace(/\.[^/.]+$/, '')   // strip original extension
      : 'output';

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
      <span class="hint">Open a HEX or S-record file to get started</span>
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

  @media (prefers-color-scheme: light) {
    :global(body) { background: #fff; color: #1e1e1e; }
    .statusbar { background: #005f9e; }
  }
</style>
