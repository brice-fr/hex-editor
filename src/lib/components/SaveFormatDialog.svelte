<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  /**
   * SaveFormatDialog — lightweight modal that asks the user which output
   * format they want before the native file-save dialog is opened.
   *
   * Props
   *   open     – controls visibility
   *   onPick   – called with { fmt, fillByte? } when the user confirms:
   *                { fmt: 'ihex' }
   *                { fmt: 'srec' }
   *                { fmt: 'binary', fillByte: number }
   *   onCancel – called when the user dismisses without choosing
   */
  let { open = false, onPick = (_choice) => {}, onCancel = () => {} } = $props();

  // 'format-select' | 'binary-config'
  let view     = $state('format-select');
  let fillHex  = $state('FF');
  let fillError = $state('');

  // Decimal preview of the fill byte
  const fillDecimal = $derived((() => {
    const v = parseInt(fillHex, 16);
    return isNaN(v) ? '—' : String(v);
  })());

  const fillValid = $derived((() => {
    const v = parseInt(fillHex, 16);
    return !isNaN(v) && v >= 0 && v <= 255 && /^[0-9a-fA-F]{1,2}$/.test(fillHex.trim());
  })());

  // Reset view each time dialog opens
  $effect(() => {
    if (open) {
      view     = 'format-select';
      fillHex  = 'FF';
      fillError = '';
    }
  });

  function pick(fmt) {
    if (fmt === 'ihex') { onPick({ fmt: 'ihex' }); return; }
    if (fmt === 'srec') { onPick({ fmt: 'srec' }); return; }
    if (fmt === 'binary') {
      view = 'binary-config';
    }
  }

  function saveBinary() {
    if (!fillValid) {
      fillError = 'Enter a valid hex byte (00–FF).';
      return;
    }
    onPick({ fmt: 'binary', fillByte: parseInt(fillHex, 16) });
  }

  function handleFillInput(e) {
    const pos = e.target.selectionStart;
    fillHex = e.target.value.toUpperCase();
    fillError = '';
    setTimeout(() => e.target.setSelectionRange(pos, pos), 0);
  }

  function handleBackdrop(e) {
    if (e.target === e.currentTarget) onCancel();
  }

  function handleKey(e) {
    if (e.key === 'Escape') {
      if (view === 'binary-config') { view = 'format-select'; }
      else { onCancel(); }
    }
    if (e.key === 'Enter' && view === 'binary-config') saveBinary();
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
      {#if view === 'format-select'}
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

          <button class="fmt-btn" onclick={() => pick('binary')}>
            <span class="fmt-icon">
              <!-- Chip / binary grid icon -->
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"
                   stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <rect x="7" y="7" width="10" height="10" rx="1"/>
                <line x1="9"  y1="4" x2="9"  y2="7"/>
                <line x1="12" y1="4" x2="12" y2="7"/>
                <line x1="15" y1="4" x2="15" y2="7"/>
                <line x1="9"  y1="17" x2="9"  y2="20"/>
                <line x1="12" y1="17" x2="12" y2="20"/>
                <line x1="15" y1="17" x2="15" y2="20"/>
                <line x1="4"  y1="9" x2="7"  y2="9"/>
                <line x1="4"  y1="12" x2="7" y2="12"/>
                <line x1="4"  y1="15" x2="7" y2="15"/>
                <line x1="17" y1="9" x2="20" y2="9"/>
                <line x1="17" y1="12" x2="20" y2="12"/>
                <line x1="17" y1="15" x2="20" y2="15"/>
              </svg>
            </span>
            <span class="fmt-body">
              <span class="fmt-name">Binary</span>
              <span class="fmt-ext">.bin</span>
            </span>
          </button>
        </div>

        <div class="actions">
          <button class="btn-cancel" onclick={onCancel}>Cancel</button>
        </div>

      {:else}
        <!-- Binary config sub-panel -->
        <h2 class="title">Binary Export Options</h2>
        <p class="subtitle">Configure the raw binary output</p>

        <label class="field-label" for="fill-byte-input">Fill byte (gaps)</label>
        <div class="fill-row">
          <input
            id="fill-byte-input"
            class="fill-input"
            class:invalid={!!fillError}
            value={fillHex}
            oninput={handleFillInput}
            maxlength="2"
            spellcheck="false"
            autocomplete="off"
            placeholder="FF"
          />
          <span class="fill-decimal">= {fillDecimal}</span>
        </div>

        {#if fillError}
          <p class="error-msg">{fillError}</p>
        {/if}

        <div class="actions">
          <button class="btn-back" onclick={() => (view = 'format-select')}>Back</button>
          <button class="btn-ok" disabled={!fillValid} onclick={saveBinary}>Save…</button>
        </div>
      {/if}
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
    color: var(--c-text);
    margin: 0;
  }

  .subtitle {
    font-size: 12px;
    color: var(--c-muted);
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
    background: var(--c-hover);
    border: 1px solid var(--c-border2);
    border-radius: 7px;
    cursor: pointer;
    color: inherit;
    text-align: left;
    transition: background 0.1s, border-color 0.1s;
  }

  .fmt-btn:hover {
    background: var(--c-accent-b);
    border-color: var(--c-accent-h);
  }

  .fmt-btn:active {
    background: #0a5080;
  }

  .fmt-icon {
    flex-shrink: 0;
    color: var(--c-accent-t);
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
    color: var(--c-text);
  }

  .fmt-ext {
    font-size: 11px;
    color: var(--c-muted);
    font-family: 'SF Mono', 'Cascadia Code', 'Fira Code', monospace;
  }

  .fmt-btn:hover .fmt-name,
  .fmt-btn:hover .fmt-ext {
    color: #fff;
  }

  /* Binary sub-panel */
  .field-label {
    font-size: 11px;
    color: var(--c-muted);
    letter-spacing: 0.02em;
    margin-top: 8px;
  }

  .fill-row {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 4px;
  }

  .fill-input {
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Courier New', monospace;
    font-size: 14px;
    padding: 7px 10px;
    background: var(--c-bg);
    border: 1px solid var(--c-dim);
    border-radius: 5px;
    color: var(--c-accent-t);
    outline: none;
    width: 60px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    transition: border-color 0.15s;
  }

  .fill-input:focus { border-color: var(--c-accent); }
  .fill-input.invalid { border-color: var(--c-err); }

  .fill-decimal {
    font-size: 11px;
    color: var(--c-muted);
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', monospace;
    white-space: nowrap;
  }

  .error-msg {
    font-size: 11px;
    color: var(--c-err);
    min-height: 14px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 12px;
  }

  .btn-cancel, .btn-back, .btn-ok {
    padding: 5px 16px;
    font-size: 13px;
    border-radius: 5px;
    cursor: pointer;
    font-family: inherit;
    transition: background 0.1s, color 0.1s;
  }

  .btn-cancel, .btn-back {
    background: transparent;
    border: 1px solid var(--c-dim);
    color: var(--c-muted);
  }

  .btn-cancel:hover, .btn-back:hover {
    background: var(--c-hover);
    color: var(--c-text);
  }

  .btn-ok {
    background: var(--c-accent-b);
    border: 1px solid transparent;
    color: #fff;
  }

  .btn-ok:hover:not(:disabled) { background: var(--c-accent-h); }

  .btn-ok:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>
