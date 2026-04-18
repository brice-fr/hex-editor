<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  /**
   * CompareDialog — lets the user pick a second file to compare against the
   * currently loaded reference file.
   *
   * Props
   *   open          – controls visibility
   *   referenceFile – full path of the already-loaded file
   *   comparedFile  – $bindable — path of the file to compare; updated by
   *                   Browse button (internal) or by the parent when a file
   *                   is drag-dropped onto the window while the dialog is open
   *   isDragging    – true while the user is dragging a file over the window
   *   onCompare(refPath, cmpPath) – called when "Compare" is confirmed
   *   onCancel()    – called when the dialog is dismissed
   */
  import { open } from '@tauri-apps/plugin-dialog';

  let {
    open:         dialogOpen  = false,
    referenceFile             = '',
    comparedFile  = $bindable(''),
    isDragging                = false,
    onCompare                 = () => {},
    onCancel                  = () => {},
  } = $props();

  const refName = $derived(referenceFile ? referenceFile.split('/').at(-1) : '—');
  const cmpName = $derived(comparedFile  ? comparedFile.split('/').at(-1)  : '');

  // Reset compared file each time the dialog closes
  $effect(() => {
    if (!dialogOpen) comparedFile = '';
  });

  async function browse() {
    const path = await open({
      multiple: false,
      filters: [
        { name: 'Firmware files', extensions: ['hex', 'ihex', 'srec', 'mot', 's19', 's28', 's37', 'bin'] },
        { name: 'All files', extensions: ['*'] },
      ],
    });
    if (path) comparedFile = path;
  }
</script>

{#if dialogOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="backdrop" onclick={onCancel}>
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="card" onclick={(e) => e.stopPropagation()}>

      <div class="title">Compare Files</div>

      <!-- Reference (already-loaded) file -->
      <div class="section-label">Reference file</div>
      <div class="file-chip ref-chip" title={referenceFile}>{refName}</div>

      <!-- Drop zone / file picker for the compared file -->
      <div class="section-label">Compare with</div>
      <div class="drop-zone" class:drag-over={isDragging}>
        {#if cmpName}
          <span class="cmp-name" title={comparedFile}>{cmpName}</span>
        {:else}
          <span class="drop-hint">Drop a file here</span>
        {/if}
        <button class="browse-btn" onclick={browse}>Browse…</button>
      </div>

      <!-- Footer buttons -->
      <div class="footer">
        <button class="btn btn-cancel" onclick={onCancel}>Cancel</button>
        <button
          class="btn btn-compare"
          onclick={() => onCompare(referenceFile, comparedFile)}
          disabled={!comparedFile}
        >Compare</button>
      </div>

    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
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
    padding: 22px 24px 18px;
    width: 360px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .title {
    font-size: 14px;
    font-weight: 600;
    color: var(--c-text);
    margin-bottom: 6px;
  }

  .section-label {
    font-size: 10.5px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--c-muted);
    margin-top: 6px;
  }

  /* Reference file display */
  .file-chip {
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', monospace;
    font-size: 12px;
    color: var(--c-accent-t);
    background: var(--c-bg);
    border: 1px solid var(--c-border);
    border-radius: 5px;
    padding: 6px 10px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Drop zone */
  .drop-zone {
    display: flex;
    align-items: center;
    gap: 10px;
    border: 1.5px dashed var(--c-dim);
    border-radius: 6px;
    padding: 10px 12px;
    background: var(--c-bg);
    transition: border-color 0.15s, background 0.15s;
    min-height: 42px;
  }

  .drop-zone.drag-over {
    border-color: var(--c-accent);
    background: var(--c-accent-bg);
  }

  .drop-hint {
    flex: 1;
    font-size: 12px;
    color: var(--c-dim);
    font-style: italic;
  }

  .cmp-name {
    flex: 1;
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', monospace;
    font-size: 12px;
    color: var(--c-accent-t);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .browse-btn {
    flex-shrink: 0;
    padding: 4px 10px;
    font-size: 11.5px;
    background: var(--c-hover);
    border: 1px solid var(--c-border2);
    border-radius: 4px;
    color: var(--c-text2);
    cursor: pointer;
    transition: background 0.1s;
  }

  .browse-btn:hover {
    background: var(--c-border2);
    color: var(--c-text);
  }

  /* Footer */
  .footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 10px;
  }

  .btn {
    padding: 6px 16px;
    font-size: 12.5px;
    border-radius: 5px;
    border: 1px solid transparent;
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }

  .btn-cancel {
    background: var(--c-hover);
    border-color: var(--c-border2);
    color: var(--c-text2);
  }

  .btn-cancel:hover {
    background: var(--c-border2);
    color: var(--c-text);
  }

  .btn-compare {
    background: var(--c-accent-b);
    color: #fff;
  }

  .btn-compare:hover:not(:disabled) {
    background: var(--c-accent-h);
  }

  .btn-compare:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
