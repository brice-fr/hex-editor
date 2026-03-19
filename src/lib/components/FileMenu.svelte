<script>
  let { onOpen = () => {}, loading = false, status = '' } = $props();
</script>

<div class="toolbar">
  <button
    class="icon-btn"
    onclick={() => onOpen()}
    disabled={loading}
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

  <div class="divider"></div>

  {#if status}
    <span class="status" class:error={status.startsWith('Error')}>{status}</span>
  {/if}
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

  .status {
    font-size: 12px;
    color: #9cdcfe;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status.error {
    color: #f44747;
  }

  @media (prefers-color-scheme: light) {
    .toolbar        { background: #f3f3f3; border-bottom-color: #ddd; }
    .icon-btn       { color: #424242; }
    .icon-btn:hover:not(:disabled) { background: #e0e0e0; color: #1e1e1e; }
    .divider        { background: #ddd; }
    .status         { color: #0070c1; }
    .status.error   { color: #cd3131; }
  }
</style>
