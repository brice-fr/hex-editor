<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  /**
   * SaveFormatDialog — lightweight modal that asks the user which output
   * format they want before the native file-save dialog is opened.
   *
   * Props
   *   open     – controls visibility
   *   onPick   – called with "ihex" | "srec" when the user confirms
   *   onCancel – called when the user dismisses without choosing
   */
  let { open = false, onPick = (_fmt) => {}, onCancel = () => {} } = $props();

  function pick(fmt) { onPick(fmt); }

  function handleBackdrop(e) {
    if (e.target === e.currentTarget) onCancel();
  }

  function handleKey(e) {
    if (e.key === 'Escape') onCancel();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Choose save format"
    onclick={handleBackdrop}
    onkeydown={handleKey}
  >
    <div class="card">
      <h2 class="title">Save as…</h2>
      <p class="subtitle">Choose the output format</p>

      <div class="options">
        <button class="fmt-btn" onclick={() => pick('ihex')}>
          <span class="fmt-icon">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
              <line x1="16" y1="13" x2="8" y2="13"/>
              <line x1="16" y1="17" x2="8" y2="17"/>
            </svg>
          </span>
          <span class="fmt-body">
            <span class="fmt-name">Intel HEX</span>
            <span class="fmt-ext">.hex · .ihex</span>
          </span>
        </button>

        <button class="fmt-btn" onclick={() => pick('srec')}>
          <span class="fmt-icon">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
              <line x1="16" y1="13" x2="8" y2="13"/>
              <line x1="16" y1="17" x2="8" y2="17"/>
            </svg>
          </span>
          <span class="fmt-body">
            <span class="fmt-name">Motorola S-record</span>
            <span class="fmt-ext">.srec · .mot · .s19</span>
          </span>
        </button>
      </div>

      <div class="actions">
        <button class="btn-cancel" onclick={onCancel}>Cancel</button>
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
    background: #2d2d2d;
    border: 1px solid #444;
    border-radius: 10px;
    padding: 24px 28px 20px;
    width: 320px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .title {
    font-size: 15px;
    font-weight: 600;
    color: #e8e8e8;
    margin: 0;
  }

  .subtitle {
    font-size: 12px;
    color: #888;
    margin: 0 0 14px;
  }

  .options {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
  }

  .fmt-btn {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 14px;
    background: #383838;
    border: 1px solid #4a4a4a;
    border-radius: 7px;
    cursor: pointer;
    color: inherit;
    text-align: left;
    transition: background 0.1s, border-color 0.1s;
  }

  .fmt-btn:hover {
    background: #0e639c;
    border-color: #1177bb;
  }

  .fmt-btn:active {
    background: #0a5080;
  }

  .fmt-icon {
    flex-shrink: 0;
    color: #9cdcfe;
    display: flex;
  }

  .fmt-btn:hover .fmt-icon {
    color: #fff;
  }

  .fmt-icon svg {
    width: 22px;
    height: 22px;
  }

  .fmt-body {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .fmt-name {
    font-size: 13px;
    font-weight: 500;
    color: #e0e0e0;
  }

  .fmt-ext {
    font-size: 11px;
    color: #888;
    font-family: 'SF Mono', 'Cascadia Code', 'Fira Code', monospace;
  }

  .fmt-btn:hover .fmt-name,
  .fmt-btn:hover .fmt-ext {
    color: #fff;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
  }

  .btn-cancel {
    padding: 5px 16px;
    font-size: 13px;
    background: transparent;
    border: 1px solid #555;
    border-radius: 5px;
    color: #aaa;
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
    font-family: inherit;
  }

  .btn-cancel:hover {
    background: #3a3a3a;
    color: #e0e0e0;
  }

  @media (prefers-color-scheme: light) {
    .card          { background: #f8f8f8; border-color: #ddd; box-shadow: 0 20px 60px rgba(0,0,0,0.2); }
    .title         { color: #1e1e1e; }
    .fmt-btn       { background: #fff; border-color: #ddd; }
    .fmt-btn:hover { background: #0e639c; border-color: #0e639c; }
    .fmt-name      { color: #1e1e1e; }
    .fmt-ext       { color: #888; }
    .btn-cancel    { border-color: #ccc; color: #555; }
    .btn-cancel:hover { background: #eee; color: #1e1e1e; }
  }
</style>
