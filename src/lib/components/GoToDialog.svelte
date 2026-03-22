<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  /**
   * GoToDialog — prompts the user for a hex address and navigates to it.
   *
   * Props
   *   open      – controls visibility
   *   prefill   – address (number) to pre-fill; refreshed each time open→true
   *   minAddr   – lowest valid address in the loaded file
   *   maxAddr   – highest valid address in the loaded file
   *   onGoto(addr: number) – called with the parsed address when OK is pressed
   *   onCancel()           – called when dismissed without navigating
   */
  let {
    open     = false,
    prefill  = 0,
    minAddr  = 0,
    maxAddr  = 0,
    onGoto   = (_addr) => {},
    onCancel = () => {},
  } = $props();

  let inputVal = $state('');
  let error    = $state('');
  let inputEl  = $state(null);

  // Reset input & error each time the dialog opens
  $effect(() => {
    if (open) {
      inputVal = '0x' + prefill.toString(16).padStart(8, '0').toUpperCase();
      error    = '';
      // Focus & select all after the DOM renders
      setTimeout(() => inputEl?.select(), 30);
    }
  });

  function parse(raw) {
    const cleaned = raw.trim().replace(/^0x/i, '');
    if (!/^[0-9a-fA-F]+$/.test(cleaned)) return NaN;
    return parseInt(cleaned, 16);
  }

  function commit() {
    const addr = parse(inputVal);
    if (isNaN(addr)) {
      error = 'Not a valid hex address.';
      return;
    }
    if (addr < minAddr || addr > maxAddr) {
      error = `Out of range — valid: 0x${minAddr.toString(16).toUpperCase().padStart(8,'0')} – 0x${maxAddr.toString(16).toUpperCase().padStart(8,'0')}`;
      return;
    }
    onGoto(addr);
  }

  function handleKey(e) {
    if (e.key === 'Enter')  commit();
    if (e.key === 'Escape') onCancel();
  }

  function handleBackdrop(e) {
    if (e.target === e.currentTarget) onCancel();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Go to address"
    tabindex="-1"
    onclick={handleBackdrop}
    onkeydown={handleKey}
  >
    <div class="card">
      <h2 class="title">Go to Address</h2>

      <label class="field-label" for="addr-input">Hex address</label>
      <input
        id="addr-input"
        class="addr-input"
        class:invalid={!!error}
        bind:value={inputVal}
        bind:this={inputEl}
        onkeydown={handleKey}
        spellcheck="false"
        autocomplete="off"
        placeholder="0x00000000"
      />

      {#if error}
        <p class="error-msg">{error}</p>
      {/if}

      <div class="actions">
        <button class="btn-cancel" onclick={onCancel}>Cancel</button>
        <button class="btn-ok"     onclick={commit}>OK</button>
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
    width: 300px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .title {
    font-size: 14px;
    font-weight: 600;
    color: var(--c-text);
    margin-bottom: 4px;
  }

  .field-label {
    font-size: 11px;
    color: var(--c-muted);
    letter-spacing: 0.02em;
  }

  .addr-input {
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Courier New', monospace;
    font-size: 14px;
    padding: 7px 10px;
    background: var(--c-bg);
    border: 1px solid var(--c-dim);
    border-radius: 5px;
    color: var(--c-accent-t);
    outline: none;
    width: 100%;
    letter-spacing: 0.05em;
    transition: border-color 0.15s;
  }

  .addr-input:focus {
    border-color: var(--c-accent);
  }

  .addr-input.invalid {
    border-color: var(--c-err);
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
    border: 1px solid var(--c-dim);
    color: var(--c-muted);
  }

  .btn-cancel:hover { background: var(--c-hover); color: var(--c-text); }

  .btn-ok {
    background: var(--c-accent-b);
    border: 1px solid transparent;
    color: #fff;
  }

  .btn-ok:hover { background: var(--c-accent-h); }
</style>
