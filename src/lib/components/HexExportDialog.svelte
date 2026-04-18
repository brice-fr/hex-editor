<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  /**
   * HexExportDialog — options dialog for exporting the main hex view as HTML.
   *
   * Props
   *   open          – controls visibility
   *   records       – current parsed records array
   *   bytesPerRow   – current bytes-per-row setting
   *   currentFile   – full path of the open file (used for default filename + path display)
   *   currentFormat – format string shown in the report header
   *   onClose       – called when the dialog should be dismissed
   */
  import { writeTextFile } from '$lib/api.js';
  import { generateHexHtml } from '$lib/hexHtmlExport.js';

  let {
    open          = false,
    records       = [],
    bytesPerRow   = 16,
    currentFile   = '',
    currentFormat = '',
    onClose       = () => {},
  } = $props();

  let showAscii   = $state(true);
  let showPath    = $state(false);
  let exportBpr   = $state(16);
  let exporting   = $state(false);
  let exportError = $state('');

  // Reset state each time the dialog opens
  $effect(() => {
    if (open) {
      exportError = '';
      exporting   = false;
      exportBpr   = bytesPerRow;
    }
  });

  async function doExport() {
    exporting   = true;
    exportError = '';
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const base = currentFile
        ? currentFile.split('/').at(-1).replace(/\.[^.]+$/, '')
        : 'report';
      const path = await save({
        defaultPath: `${base}.html`,
        filters: [{ name: 'HTML Report', extensions: ['html'] }],
      });
      if (!path) { exporting = false; return; }

      const html = generateHexHtml({ records, bytesPerRow: exportBpr, currentFile, showAscii, showPath, currentFormat });
      await writeTextFile(path, html);
      onClose();
    } catch (e) {
      exportError = String(e);
    } finally {
      exporting = false;
    }
  }

  function handleBackdrop(e) {
    if (e.target === e.currentTarget) onClose();
  }

  function handleKey(e) {
    if (e.key === 'Escape') onClose();
    if (e.key === 'Enter' && !exporting) doExport();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Export as HTML"
    onclick={handleBackdrop}
    onkeydown={handleKey}
  >
    <div class="card">
      <h2 class="title">Export as HTML</h2>
      <p class="subtitle">Configure the report output</p>

      <label class="checkbox-row">
        <input type="checkbox" bind:checked={showAscii}>
        Show ASCII column
      </label>

      <label class="checkbox-row">
        <input type="checkbox" bind:checked={showPath}>
        Show full file path in report
      </label>

      <div class="field-row">
        <span class="field-label">Columns</span>
        <div class="bpr-group">
          {#each [8, 16, 32] as n}
            <label class="bpr-option" class:selected={exportBpr === n}>
              <input type="radio" name="export-bpr" value={n} bind:group={exportBpr}>
              {n}
            </label>
          {/each}
        </div>
      </div>

      {#if exportError}
        <p class="error-msg">{exportError}</p>
      {/if}

      <div class="actions">
        <button class="btn-cancel" onclick={onClose} disabled={exporting}>Cancel</button>
        <button class="btn-ok" onclick={doExport} disabled={exporting}>
          {#if exporting}Exporting…{:else}Export…{/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    backdrop-filter: blur(2px);
  }

  .card {
    background: var(--c-raised);
    border: 1px solid var(--c-border2);
    border-radius: 10px;
    padding: 24px 28px 20px;
    width: 300px;
    box-shadow: 0 20px 60px rgba(0,0,0,0.6);
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  }

  .title {
    font-size: 15px;
    font-weight: 600;
    color: var(--c-text);
    margin: 0;
  }

  .subtitle {
    font-size: 12px;
    color: var(--c-muted);
    margin: 0 0 8px;
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--c-text);
    cursor: pointer;
    padding: 3px 0;
  }

  .field-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 3px 0;
  }

  .field-label {
    font-size: 13px;
    color: var(--c-text);
    flex-shrink: 0;
  }

  .bpr-group {
    display: flex;
    gap: 4px;
  }

  .bpr-option {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 24px;
    border: 1px solid var(--c-dim);
    border-radius: 4px;
    font-size: 12px;
    color: var(--c-muted);
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }

  .bpr-option input[type="radio"] {
    display: none;
  }

  .bpr-option:hover {
    background: var(--c-hover);
    color: var(--c-text);
  }

  .bpr-option.selected {
    background: var(--c-accent-b);
    border-color: var(--c-accent-b);
    color: #fff;
  }

  .error-msg {
    font-size: 11px;
    color: var(--c-err);
    margin-top: 4px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 10px;
  }

  .btn-cancel, .btn-ok {
    padding: 5px 16px;
    font-size: 13px;
    border-radius: 5px;
    cursor: pointer;
    font-family: inherit;
    transition: background 0.1s, color 0.1s;
  }

  .btn-cancel {
    background: transparent;
    border: 1px solid var(--c-dim);
    color: var(--c-muted);
  }
  .btn-cancel:hover:not(:disabled) { background: var(--c-hover); color: var(--c-text); }

  .btn-ok {
    background: var(--c-accent-b);
    border: 1px solid transparent;
    color: #fff;
  }
  .btn-ok:hover:not(:disabled) { background: var(--c-accent-h); }

  .btn-ok:disabled, .btn-cancel:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>
