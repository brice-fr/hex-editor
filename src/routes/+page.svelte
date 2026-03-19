<script>
  import { onMount } from 'svelte';
  import { Menu, Submenu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';
  import { open } from '@tauri-apps/plugin-dialog';
  import { openFile, parseIntelHex, parseSrec, detectFileFormat } from '$lib/api.js';
  import FileMenu from '$lib/components/FileMenu.svelte';
  import HexViewer from '$lib/components/HexViewer.svelte';

  let records     = $state([]);
  let currentFile = $state('');
  let currentFormat = $state('');
  let status      = $state('');
  let loading     = $state(false);

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
        status = `Unsupported format: ${format}`;
        return;
      }

      records       = parsed.records;
      currentFile   = selected;
      currentFormat = format;
      status = `Loaded ${parsed.total_data_bytes.toLocaleString()} bytes  ·  ${format.toUpperCase()}`;
    } catch (err) {
      status = `Error: ${err}`;
    } finally {
      loading = false;
    }
  }

  // ── Native macOS menu bar ──
  onMount(async () => {
    try {
      const menu = await Menu.new({
        items: [
          // ① App menu (About / Quit / Hide / …)
          await Submenu.new({
            text: 'Hex Editor',
            items: [
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
              await PredefinedMenuItem.new({ item: 'Separator' }),
              await PredefinedMenuItem.new({ item: 'CloseWindow' }),
            ],
          }),
          // ③ Edit menu (copy / paste / select-all for the hex viewer)
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
        ],
      });

      await menu.setAsAppMenu();
    } catch (err) {
      // Non-fatal: native menu is best-effort
      console.warn('Could not build native menu:', err);
    }
  });
</script>

<div class="app-shell">
  <header class="titlebar">
    <span class="app-title">Hex Editor</span>
    {#if currentFile}
      <span class="file-info">
        {currentFile.split('/').at(-1)}
        <span class="badge">{currentFormat.toUpperCase()}</span>
      </span>
    {/if}
  </header>

  <!-- Toolbar: icon button + status -->
  <FileMenu onOpen={handleOpen} {loading} {status} />

  <main class="viewer-area">
    <HexViewer {records} />
  </main>
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
  }

  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .titlebar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    background: #323233;
    border-bottom: 1px solid #3c3c3c;
    user-select: none;
    -webkit-app-region: drag;
  }

  .app-title {
    font-size: 13px;
    font-weight: 600;
    color: #ccc;
  }

  .file-info {
    font-size: 12px;
    color: #9cdcfe;
    font-family: 'Courier New', Courier, monospace;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .badge {
    font-size: 10px;
    background: #0e639c;
    color: #fff;
    padding: 1px 5px;
    border-radius: 3px;
  }

  .viewer-area {
    flex: 1;
    overflow: hidden;
  }

  @media (prefers-color-scheme: light) {
    :global(body) { background: #fff; color: #1e1e1e; }
    .titlebar { background: #f3f3f3; border-bottom-color: #ddd; }
    .app-title { color: #333; }
    .file-info { color: #0070c1; }
  }
</style>
