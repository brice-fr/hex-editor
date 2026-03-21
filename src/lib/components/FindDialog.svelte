<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  /**
   * FindDialog — floating (non-modal) search panel.
   *
   * Props
   *   open        – controls visibility
   *   records     – the loaded data records (same array passed to HexViewer)
   *   topAddress  – currently topmost visible address (updated live by parent)
   *   onNavigate  – called with a byte address to scroll HexViewer there
   *   onClose     – called when the panel is dismissed
   */
  let {
    open       = false,
    records    = [],
    topAddress = 0,
    onNavigate = (_addr) => {},
    onClose    = () => {},
  } = $props();

  // ── State ──────────────────────────────────────────────────────────────────
  let tab           = $state('text');   // 'text' | 'hex'
  let textQuery     = $state('');
  let hexQuery      = $state('');
  let direction     = $state('forward');
  let wrapAround    = $state(true);
  let caseSensitive = $state(false);

  let results     = $state([]);
  let showResults = $state(false);
  let statusMsg   = $state('');
  let errorMsg    = $state('');
  let hasMatch       = $state(false);  // true after first successful Find
  let lastMatchAddr  = $state(null);   // address of the last successful match

  let inputEl = $state(null);
  let panelEl = $state(null);

  // Panel position — null means "use CSS default (top-right)"
  let panelX = $state(null);
  let panelY = $state(null);

  // Drag state (plain variables, no reactivity needed)
  let dragging    = false;
  let dragOffsetX = 0;
  let dragOffsetY = 0;

  function onHeaderPointerDown(e) {
    if (e.button !== 0 || !panelEl) return;
    const rect = panelEl.getBoundingClientRect();
    // Latch absolute position the first time we drag
    panelX = rect.left;
    panelY = rect.top;
    dragOffsetX = e.clientX - rect.left;
    dragOffsetY = e.clientY - rect.top;
    dragging = true;
    e.currentTarget.setPointerCapture(e.pointerId);
    e.preventDefault();
  }

  function onHeaderPointerMove(e) {
    if (!dragging) return;
    panelX = e.clientX - dragOffsetX;
    panelY = e.clientY - dragOffsetY;
  }

  function onHeaderPointerUp(e) {
    dragging = false;
    e.currentTarget.releasePointerCapture(e.pointerId);
  }

  // Compute the inline style for the panel
  const panelStyle = $derived(
    panelX !== null
      ? `left:${panelX}px; top:${panelY}px; right:auto;`
      : ''
  );

  // Focus input and reset messages every time the panel opens
  $effect(() => {
    if (open) {
      statusMsg   = '';
      errorMsg    = '';
      showResults = false;
      results     = [];
      hasMatch      = false;
      lastMatchAddr = null;
      // Reset position so it starts top-right each time it's reopened
      panelX = null;
      panelY = null;
      setTimeout(() => inputEl?.focus(), 40);
    }
  });

  // Clear results whenever the query or tab changes
  $effect(() => {
    // create explicit dependencies
    const _tab = tab, _tq = textQuery, _hq = hexQuery;
    showResults   = false;
    results       = [];
    statusMsg     = '';
    errorMsg      = '';
    hasMatch      = false;
    lastMatchAddr = null;
  });

  // ── Pattern parsing ────────────────────────────────────────────────────────
  function parsePattern() {
    errorMsg = '';
    if (tab === 'hex') {
      const raw = hexQuery.replace(/\s+/g, '');
      if (!raw) { errorMsg = 'Enter a hex pattern  (e.g. DE AD BE EF)'; return null; }
      if (raw.length % 2 !== 0) { errorMsg = 'Hex pattern must have an even number of digits'; return null; }
      const bytes = [];
      for (let i = 0; i < raw.length; i += 2) {
        const n = parseInt(raw.slice(i, i + 2), 16);
        if (isNaN(n)) { errorMsg = 'Invalid hex character'; return null; }
        bytes.push(n);
      }
      return new Uint8Array(bytes);
    } else {
      if (!textQuery) { errorMsg = 'Enter a search text'; return null; }
      const str = caseSensitive ? textQuery : textQuery.toLowerCase();
      return new TextEncoder().encode(str);
    }
  }

  // ── Search engine ─────────────────────────────────────────────────────────
  function computeAllMatches(pat) {
    const matches = [];
    const textInsensitive = tab === 'text' && !caseSensitive;
    for (const rec of records) {
      const isData = rec.record_type === 'Data'
                  || rec.record_type === 'S1'
                  || rec.record_type === 'S2'
                  || rec.record_type === 'S3';
      if (!isData || !rec.data?.length || rec.data.length < pat.length) continue;
      const raw = new Uint8Array(rec.data);
      const src = textInsensitive
        ? raw.map(b => b >= 65 && b <= 90 ? b + 32 : b)
        : raw;
      outer: for (let i = 0; i <= src.length - pat.length; i++) {
        for (let j = 0; j < pat.length; j++) {
          if (src[i + j] !== pat[j]) continue outer;
        }
        matches.push(rec.address + i);
      }
    }
    return matches.sort((a, b) => a - b);
  }

  // ── Actions ───────────────────────────────────────────────────────────────
  function handleSearch() {
    const pat = parsePattern();
    if (!pat) return;
    statusMsg   = '';
    showResults = false;

    const all = computeAllMatches(pat);
    if (all.length === 0) { statusMsg = 'Pattern not found'; return; }

    // After a previous match, advance strictly beyond it.
    // On the first search, use topAddress as the starting reference.
    const pivot = lastMatchAddr !== null ? lastMatchAddr : topAddress;

    let target;
    if (direction === 'forward') {
      target = all.find(a => a > pivot);
      if (target === undefined) {
        if (wrapAround) { target = all[0]; statusMsg = 'Wrapped to beginning · '; }
        else            { statusMsg = 'No more matches forward'; return; }
      }
    } else {
      const prev = all.filter(a => a < pivot);
      target = prev.length > 0 ? prev[prev.length - 1] : undefined;
      if (target === undefined) {
        if (wrapAround) { target = all[all.length - 1]; statusMsg = 'Wrapped to end · '; }
        else            { statusMsg = 'No more matches backward'; return; }
      }
    }
    onNavigate(target);
    statusMsg     += `Match at ${fmtAddr(target)}`;
    hasMatch       = true;
    lastMatchAddr  = target;
  }

  function handleFindAll() {
    const pat = parsePattern();
    if (!pat) return;
    statusMsg   = '';
    showResults = true;
    results     = computeAllMatches(pat);
    statusMsg   = results.length === 0
      ? 'No matches found'
      : `${results.length} match${results.length !== 1 ? 'es' : ''} found`;
  }

  function handleResultClick(addr) {
    onNavigate(addr);
    statusMsg = `Navigated to ${fmtAddr(addr)}`;
  }

  // ── Helpers ───────────────────────────────────────────────────────────────
  function fmtAddr(a) {
    return '0x' + a.toString(16).toUpperCase().padStart(8, '0');
  }

  function handleKey(e) {
    if (e.key === 'Escape') { e.stopPropagation(); onClose(); }
    if (e.key === 'Enter')  { e.preventDefault();  handleSearch(); }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="find-panel" bind:this={panelEl} style={panelStyle}
       onkeydown={handleKey} role="dialog" aria-label="Find" aria-modal="false" tabindex="-1">

    <!-- Header — drag handle -->
    <div class="panel-header"
         onpointerdown={onHeaderPointerDown}
         onpointermove={onHeaderPointerMove}
         onpointerup={onHeaderPointerUp}>
      <span class="panel-title">Find</span>
      <button class="close-btn" onclick={onClose} aria-label="Close find panel">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"
             stroke-linecap="round" xmlns="http://www.w3.org/2000/svg">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6"  y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <!-- Tab bar -->
    <div class="tab-bar" role="tablist">
      <button role="tab" class="tab" class:active={tab === 'text'}
              aria-selected={tab === 'text'} onclick={() => tab = 'text'}>Text</button>
      <button role="tab" class="tab" class:active={tab === 'hex'}
              aria-selected={tab === 'hex'}  onclick={() => tab = 'hex'}>Hex</button>
    </div>

    <!-- Search input -->
    <div class="input-row">
      <label class="input-label" for="find-input">Search for</label>
      {#if tab === 'text'}
        <input
          id="find-input"
          bind:this={inputEl}
          bind:value={textQuery}
          class="find-input"
          placeholder="Text to find…"
          spellcheck="false"
          autocomplete="off"
        />
      {:else}
        <input
          id="find-input"
          bind:this={inputEl}
          bind:value={hexQuery}
          class="find-input mono"
          placeholder="DE AD BE EF …"
          spellcheck="false"
          autocomplete="off"
        />
      {/if}
    </div>

    <!-- Options -->
    <div class="options">
      <fieldset class="direction-group">
        <legend class="opt-legend">Direction</legend>
        <label class="radio-label">
          <input type="radio" bind:group={direction} value="forward"/> Forward
        </label>
        <label class="radio-label">
          <input type="radio" bind:group={direction} value="backward"/> Backward
        </label>
      </fieldset>

      <div class="checkboxes">
        <label class="check-label">
          <input type="checkbox" bind:checked={wrapAround}/> Wrap Around
        </label>
        <label class="check-label" class:dimmed={tab === 'hex'}>
          <input type="checkbox" bind:checked={caseSensitive} disabled={tab === 'hex'}/> Case Sensitive
        </label>
      </div>
    </div>

    <!-- Status / Error -->
    {#if errorMsg}
      <div class="msg error">{errorMsg}</div>
    {:else if statusMsg}
      <div class="msg info">{statusMsg}</div>
    {/if}

    <!-- Find All results -->
    {#if showResults && results.length > 0}
      <div class="results" role="listbox" aria-label="Match list">
        {#each results.slice(0, 500) as addr}
          <button class="result-row" role="option" aria-selected="false" onclick={() => handleResultClick(addr)}>
            {fmtAddr(addr)}
          </button>
        {/each}
        {#if results.length > 500}
          <div class="results-more">… and {(results.length - 500).toLocaleString()} more</div>
        {/if}
      </div>
    {/if}

    <!-- Buttons -->
    <div class="btn-row">
      <button class="btn btn-ghost" onclick={onClose}>Close</button>
      <div class="btn-right">
        <button class="btn btn-secondary" onclick={handleFindAll}>Find All</button>
        <button class="btn btn-primary"   onclick={handleSearch}>{hasMatch ? 'Find Next' : 'Find'}</button>
      </div>
    </div>

  </div>
{/if}

<style>
  .find-panel {
    position: fixed;
    top: 56px;       /* below the 36px toolbar + some gap */
    right: 16px;
    width: 340px;
    background: #252526;
    border: 1px solid #454545;
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.55);
    z-index: 60;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    font-size: 12.5px;
  }

  /* ── Header ── */
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px 8px;
    border-bottom: 1px solid #3a3a3a;
    cursor: move;
    user-select: none;
  }

  .panel-title {
    font-size: 13px;
    font-weight: 600;
    color: #e0e0e0;
    letter-spacing: 0.01em;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: #888;
    padding: 0;
    transition: background 0.1s, color 0.1s;
  }

  .close-btn:hover { background: #3c3c3c; color: #e0e0e0; }
  .close-btn svg   { width: 14px; height: 14px; }

  /* ── Tabs ── */
  .tab-bar {
    display: flex;
    border-bottom: 1px solid #3a3a3a;
  }

  .tab {
    flex: 1;
    padding: 6px 0;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    color: #888;
    transition: color 0.1s, border-color 0.1s;
    font-family: inherit;
  }

  .tab:hover  { color: #ccc; }
  .tab.active { color: #9cdcfe; border-bottom-color: #9cdcfe; }

  /* ── Input ── */
  .input-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px 12px 6px;
  }

  .input-label {
    font-size: 11px;
    color: #888;
    user-select: none;
  }

  .find-input {
    width: 100%;
    padding: 5px 8px;
    background: #1e1e1e;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 12.5px;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    outline: none;
    transition: border-color 0.1s;
  }

  .find-input.mono {
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Courier New', monospace;
    letter-spacing: 0.05em;
  }

  .find-input:focus { border-color: #007acc; }

  /* ── Options ── */
  .options {
    display: flex;
    gap: 12px;
    padding: 8px 12px;
    flex-wrap: wrap;
  }

  .direction-group {
    border: 1px solid #3a3a3a;
    border-radius: 5px;
    padding: 5px 10px 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .opt-legend {
    font-size: 10.5px;
    color: #666;
    padding: 0 4px;
    user-select: none;
  }

  .radio-label {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    color: #ccc;
    cursor: pointer;
    user-select: none;
  }

  .radio-label input { accent-color: #007acc; cursor: pointer; }

  .checkboxes {
    display: flex;
    flex-direction: column;
    gap: 6px;
    justify-content: center;
  }

  .check-label {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    color: #ccc;
    cursor: pointer;
    user-select: none;
  }

  .check-label input  { accent-color: #007acc; cursor: pointer; }
  .check-label.dimmed { opacity: 0.38; cursor: not-allowed; }

  /* ── Status messages ── */
  .msg {
    margin: 0 12px 6px;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11.5px;
  }

  .msg.error { background: rgba(244, 71, 71, 0.15); color: #f47171; }
  .msg.info  { background: rgba(0, 122, 204, 0.15); color: #9cdcfe; }

  /* ── Results list ── */
  .results {
    max-height: 160px;
    overflow-y: auto;
    border-top: 1px solid #3a3a3a;
    border-bottom: 1px solid #3a3a3a;
    margin: 0 0 2px;
  }

  .result-row {
    display: block;
    width: 100%;
    padding: 3px 14px;
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
    color: #9cdcfe;
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Courier New', monospace;
    font-size: 12px;
    transition: background 0.08s;
  }

  .result-row:hover { background: #0e639c; color: #fff; }

  .results-more {
    padding: 3px 14px 5px;
    font-size: 11px;
    color: #666;
    font-style: italic;
  }

  /* ── Footer buttons ── */
  .btn-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px 10px;
    gap: 8px;
  }

  .btn-right {
    display: flex;
    gap: 6px;
  }

  .btn {
    padding: 4px 14px;
    font-size: 12px;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-family: inherit;
    transition: background 0.1s;
  }

  .btn-ghost     { background: transparent; border: 1px solid #555; color: #aaa; }
  .btn-secondary { background: #3a3a3a;     color: #d4d4d4; }
  .btn-primary   { background: #0e639c;     color: #fff; }

  .btn-ghost:hover     { background: #3a3a3a; color: #e0e0e0; }
  .btn-secondary:hover { background: #4a4a4a; }
  .btn-primary:hover   { background: #1177bb; }

  /* ── Light mode ── */
  @media (prefers-color-scheme: light) {
    .find-panel     { background: #f8f8f8; border-color: #d4d4d4; box-shadow: 0 8px 32px rgba(0,0,0,0.18); }
    .panel-header   { border-bottom-color: #e0e0e0; }
    .panel-title    { color: #1e1e1e; }
    .close-btn      { color: #666; }
    .close-btn:hover { background: #e8e8e8; color: #1e1e1e; }
    .tab-bar        { border-bottom-color: #e0e0e0; }
    .tab            { color: #666; }
    .tab.active     { color: #0070c1; border-bottom-color: #0070c1; }
    .find-input     { background: #fff; border-color: #ccc; color: #1e1e1e; }
    .direction-group { border-color: #ddd; }
    .opt-legend     { color: #999; }
    .radio-label, .check-label { color: #333; }
    .msg.info       { background: rgba(0,112,193,0.1); color: #0070c1; }
    .results        { border-color: #e0e0e0; }
    .result-row     { color: #0070c1; }
    .btn-ghost      { border-color: #ccc; color: #555; }
    .btn-ghost:hover { background: #eee; }
    .btn-secondary  { background: #e8e8e8; color: #333; }
    .btn-secondary:hover { background: #ddd; }
  }
</style>
