<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  let { onOpen = () => {}, onSave = () => {}, onExport = () => {}, onFind = () => {}, onGoto = () => {}, loading = false, saving = false, hasFile = false } = $props();
</script>

<div class="toolbar">
  <!-- Open button -->
  <button
    class="icon-btn"
    onclick={() => onOpen()}
    disabled={loading || saving}
    title="Open file… (⌘O)"
    aria-label="Open file"
  >
    {#if loading}
      <!-- Spinner -->
      <svg class="spin" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
      </svg>
    {:else}
      <!-- Folder-open icon -->
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        <polyline points="16 13 12 9 8 13"/>
        <line x1="12" y1="9" x2="12" y2="17"/>
      </svg>
    {/if}
  </button>

  <!-- Save as button -->
  <button
    class="icon-btn"
    onclick={() => onSave()}
    disabled={loading || saving || !hasFile}
    title="Save as… (⌘⇧S)"
    aria-label="Save file as"
  >
    {#if saving}
      <!-- Spinner -->
      <svg class="spin" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
      </svg>
    {:else}
      <!-- Floppy-disk / save icon -->
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
        <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
        <polyline points="17 21 17 13 7 13 7 21"/>
        <polyline points="7 3 7 8 15 8"/>
      </svg>
    {/if}
  </button>

  <!-- Export as HTML button -->
  <button
    class="icon-btn"
    onclick={() => onExport()}
    disabled={!hasFile}
    title="Export as HTML report…"
    aria-label="Export as HTML report"
  >
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
      <polyline points="14 2 14 8 20 8"/>
      <line x1="12" y1="18" x2="12" y2="12"/>
      <polyline points="9 15 12 12 15 15"/>
    </svg>
  </button>

  <div class="divider"></div>

  <!-- Find button -->
  <button
    class="icon-btn"
    onclick={() => onFind()}
    disabled={!hasFile}
    title="Find… (⌘F)"
    aria-label="Find"
  >
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"
         stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="11" cy="11" r="7"/>
      <line x1="16.5" y1="16.5" x2="22" y2="22"/>
    </svg>
  </button>

  <!-- Go to address button -->
  <button
    class="icon-btn"
    onclick={() => onGoto()}
    disabled={!hasFile}
    title="Go to address… (⌘G)"
    aria-label="Go to address"
  >
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
      <!-- Downward arrow -->
      <line x1="12" y1="3" x2="12" y2="15"/>
      <polyline points="7 10 12 15 17 10"/>
      <!-- Target line -->
      <line x1="3" y1="20" x2="21" y2="20"/>
    </svg>
  </button>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: #2d2d2d;
    border-bottom: 1px solid #3c3c3c;
    height: 36px;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: #cccccc;
    padding: 0;
    transition: background 0.1s, color 0.1s;
  }

  .icon-btn:hover:not(:disabled) {
    background: #3c3c3c;
    color: #ffffff;
  }

  .icon-btn:active:not(:disabled) {
    background: #4a4a4a;
  }

  .icon-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .icon-btn svg {
    width: 18px;
    height: 18px;
  }

  .spin {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .divider {
    width: 1px;
    height: 18px;
    background: #3c3c3c;
    margin: 0 4px;
  }

  @media (prefers-color-scheme: light) {
    .toolbar  { background: #f3f3f3; border-bottom-color: #ddd; }
    .icon-btn { color: #424242; }
    .icon-btn:hover:not(:disabled) { background: #e0e0e0; color: #1e1e1e; }
    .divider  { background: #ddd; }
  }
</style>
