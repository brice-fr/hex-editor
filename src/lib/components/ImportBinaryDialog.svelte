<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  /**
   * ImportBinaryDialog — asks the user for a base address when opening a .bin file.
   *
   * Props
   *   open      – controls visibility
   *   onImport(addr: number) – called with the parsed base address when Import is clicked
   *   onCancel()             – called when dismissed without importing
   */
  let {
    open     = false,
    onImport = (_addr) => {},
    onCancel = () => {},
  } = $props();

  let inputVal  = $state('00000000');
  let error     = $state('');
  let inputEl   = $state(null);

  // Decimal preview
  const decimalPreview = $derived((() => {
    const cleaned = inputVal.trim().toUpperCase();
    if (!/^[0-9A-F]{1,8}$/.test(cleaned)) return '—';
    return parseInt(cleaned, 16).toLocaleString();
  })());

  const isValid = $derived((() => {
    const cleaned = inputVal.trim().toUpperCase();
    return /^[0-9A-F]{1,8}$/.test(cleaned);
  })());

  // Reset each time the dialog opens
  $effect(() => {
    if (open) {
      inputVal = '00000000';
      error    = '';
      setTimeout(() => inputEl?.select(), 30);
    }
  });

  function commit() {
    const cleaned = inputVal.trim().toUpperCase();
    if (!/^[0-9A-F]{1,8}$/.test(cleaned)) {
      error = 'Enter a valid 8-digit hex address (0–FFFFFFFF).';
      return;
    }
    error = '';
    onImport(parseInt(cleaned, 16));
  }

  function handleKey(e) {
    if (e.key === 'Enter' && isValid) commit();
    if (e.key === 'Escape') onCancel();
  }

  function handleBackdrop(e) {
    if (e.target === e.currentTarget) onCancel();
  }

  function handleInput(e) {
    // Uppercase automatically
    const pos = e.target.selectionStart;
    inputVal = e.target.value.toUpperCase();
    // restore cursor after reactive update
    setTimeout(() => e.target.setSelectionRange(pos, pos), 0);
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Import binary — set base address"
    tabindex="-1"
    onclick={handleBackdrop}
    onkeydown={handleKey}
  >
    <div class="card">
      <h2 class="title">Import Binary</h2>
      <p class="subtitle">Set the base address for this binary file</p>

      <label class="field-label" for="base-addr-input">Base address (hex)</label>
      <div class="input-row">
        <input
          id="base-addr-input"
          class="addr-input"
          class:invalid={!!error}
          bind:this={inputEl}
          value={inputVal}
          oninput={handleInput}
          onkeydown={handleKey}
          maxlength="8"
          spellcheck="false"
          autocomplete="off"
          placeholder="00000000"
        />
        <span class="decimal-preview">= {decimalPreview}</span>
      </div>

      {#if error}
        <p class="error-msg">{error}</p>
      {/if}

      <div class="actions">
        <button class="btn-cancel" onclick={onCancel}>Cancel</button>
        <button class="btn-ok" disabled={!isValid} onclick={commit}>Import</button>
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
    padding: 22px 24px 18px;
    width: 320px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .title {
    font-size: 14px;
    font-weight: 600;
    color: #e8e8e8;
    margin-bottom: 2px;
  }

  .subtitle {
    font-size: 12px;
    color: #888;
    margin: 0 0 10px;
  }

  .field-label {
    font-size: 11px;
    color: #888;
    letter-spacing: 0.02em;
  }

  .input-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .addr-input {
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Courier New', monospace;
    font-size: 14px;
    padding: 7px 10px;
    background: #1e1e1e;
    border: 1px solid #555;
    border-radius: 5px;
    color: #9cdcfe;
    outline: none;
    flex: 1;
    letter-spacing: 0.05em;
    transition: border-color 0.15s;
  }

  .addr-input:focus {
    border-color: #007acc;
  }

  .addr-input.invalid {
    border-color: #f44747;
  }

  .decimal-preview {
    font-size: 11px;
    color: #888;
    white-space: nowrap;
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', monospace;
  }

  .error-msg {
    font-size: 11px;
    color: #f44747;
    min-height: 14px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 6px;
  }

  .btn-cancel, .btn-ok {
    padding: 5px 18px;
    font-size: 13px;
    border-radius: 5px;
    cursor: pointer;
    font-family: inherit;
    transition: background 0.1s;
  }

  .btn-cancel {
    background: transparent;
    border: 1px solid #555;
    color: #aaa;
  }

  .btn-cancel:hover { background: #3a3a3a; color: #e0e0e0; }

  .btn-ok {
    background: #0e639c;
    border: 1px solid transparent;
    color: #fff;
  }

  .btn-ok:hover:not(:disabled) { background: #1177bb; }

  .btn-ok:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  @media (prefers-color-scheme: light) {
    .card        { background: #f8f8f8; border-color: #ddd; box-shadow: 0 20px 60px rgba(0,0,0,0.2); }
    .title       { color: #1e1e1e; }
    .subtitle    { color: #666; }
    .addr-input  { background: #fff; border-color: #ccc; color: #0070c1; }
    .btn-cancel  { border-color: #ccc; color: #555; }
    .btn-cancel:hover { background: #eee; color: #1e1e1e; }
  }
</style>
