<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  /** @type {{ record_type: string; address: number; data: number[] }[]} */
  let { records = [], bytesPerRow = 16, onScrolled = () => {}, onTopAddress = (_addr) => {}, gotoTarget = null } = $props();

  const ROW_HEIGHT = 20;
  const OVERSCAN   = 8;

  function buildRows(records, bytesPerRow) {
    const rows = [];
    for (const rec of records) {
      const isData = rec.record_type === 'Data'
                  || rec.record_type === 'S1'
                  || rec.record_type === 'S2'
                  || rec.record_type === 'S3';
      if (!isData || rec.data.length === 0) continue;
      for (let i = 0; i < rec.data.length; i += bytesPerRow) {
        rows.push({ address: rec.address + i, bytes: rec.data.slice(i, i + bytesPerRow) });
      }
    }
    return rows;
  }

  const rows = $derived(buildRows(records, bytesPerRow));

  let scrollTop    = $state(0);
  let clientHeight = $state(600);

  const totalHeight = $derived(rows.length * ROW_HEIGHT);
  const startIdx    = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN));
  const endIdx      = $derived(Math.min(rows.length, Math.ceil((scrollTop + clientHeight) / ROW_HEIGHT) + OVERSCAN));
  const visibleRows = $derived(rows.slice(startIdx, endIdx));
  const offsetTop   = $derived(startIdx * ROW_HEIGHT);

  let scrollEl         = $state(null);
  let suppressScrolled = false;

  // Throttle state — plain vars, not reactive (no re-render needed)
  const SCROLL_THROTTLE_MS = 50;
  let rawScrollTop = 0;   // latest position from every scroll event
  let throttleId   = null;

  function commitScroll() {
    if (scrollTop !== rawScrollTop) scrollTop = rawScrollTop;
    throttleId = null;
  }

  function onScroll(e) {
    rawScrollTop = e.currentTarget.scrollTop;
    if (!suppressScrolled) onScrolled(); // user-initiated only

    if (throttleId === null) {
      // Leading edge: apply immediately, schedule a trailing catch-up
      scrollTop  = rawScrollTop;
      throttleId = setTimeout(commitScroll, SCROLL_THROTTLE_MS);
    }
    // Intermediate events: rawScrollTop advances; commitScroll will flush it
  }

  // Clean up any pending timer when the component is destroyed
  $effect(() => () => { if (throttleId !== null) clearTimeout(throttleId); });

  // Keep parent's top-address in sync (fires on mount, file load, scroll, and navigation)
  $effect(() => {
    onTopAddress(rows[Math.floor(scrollTop / ROW_HEIGHT)]?.address ?? 0);
  });

  // Navigate to a target address when gotoTarget changes
  $effect(() => {
    if (!gotoTarget || !scrollEl) return;
    const rowIdx = rows.findIndex(
      r => r.address <= gotoTarget.addr && gotoTarget.addr < r.address + bytesPerRow
    );
    if (rowIdx >= 0) {
      const newTop = rowIdx * ROW_HEIGHT;
      // Cancel any pending throttle so it can't overwrite the navigation jump
      if (throttleId !== null) { clearTimeout(throttleId); throttleId = null; }
      rawScrollTop     = newTop;
      suppressScrolled = true;
      scrollEl.scrollTop = newTop;
      scrollTop          = newTop;
      // Re-enable after the scroll event has fired
      requestAnimationFrame(() => { suppressScrolled = false; });
    }
  });

  function hex8(n)  { return n.toString(16).padStart(2, '0').toUpperCase(); }
  function hex32(n) { return n.toString(16).padStart(8, '0').toUpperCase(); }
  function toAscii(b) { return b >= 0x20 && b < 0x7f ? String.fromCharCode(b) : '.'; }
  function isPrint(b) { return b >= 0x20 && b < 0x7f; }

  const COLS = Array.from({ length: 16 }, (_, i) => i);
</script>

<div class="hex-viewer">
  {#if rows.length === 0}
    <p class="empty-state">No data to display. Open an Intel HEX or S-record file.</p>
  {:else}

    <!-- ── Sticky header ── -->
    <div class="hex-header">
      <span class="col-addr">Address</span>
      <span class="v-sep"></span>
      <span class="col-hex-area">
        {#each COLS as i}
          {#if i === 8}<span class="mid-gap"></span>{/if}
          <span class="hb ec-{i % 2}">{i.toString(16).padStart(2, '0').toUpperCase()}</span>
        {/each}
      </span>
      <span class="v-sep"></span>
      <span class="col-ascii-area">
        {#each COLS as i}
          <span class="ac">{i.toString(16).toUpperCase()}</span>
        {/each}
      </span>
    </div>

    <!-- ── Virtual-scroll container ── -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="hex-scroll"
      role="grid"
      aria-rowcount={rows.length}
      onscroll={onScroll}
      bind:clientHeight
      bind:this={scrollEl}
    >
      <div class="scroll-space" style="height:{totalHeight}px;">
        <div class="visible-rows" style="top:{offsetTop}px;">
          {#each visibleRows as row (row.address)}
            {@const pad = bytesPerRow - row.bytes.length}
            <div class="hex-row" role="row">

              <span class="col-addr">{hex32(row.address)}</span>
              <span class="v-sep"></span>

              <span class="col-hex-area">
                {#each row.bytes as byte, i}
                  {#if i === 8}<span class="mid-gap"></span>{/if}
                  <span class="hb ec-{i % 2}">{hex8(byte)}</span>
                {/each}
                {#each { length: pad } as _, i}
                  {@const col = row.bytes.length + i}
                  {#if col === 8}<span class="mid-gap"></span>{/if}
                  <span class="hb ec-{col % 2} pad"></span>
                {/each}
              </span>

              <span class="v-sep"></span>
              <span class="col-ascii-area">
                {#each row.bytes as byte}
                  <span class="ac" class:np={!isPrint(byte)}>{toAscii(byte)}</span>
                {/each}
              </span>

            </div>
          {/each}
        </div>
      </div>
    </div>

    <div class="row-count">
      {rows.length.toLocaleString()} rows &nbsp;·&nbsp; {(rows.length * bytesPerRow).toLocaleString()} bytes
    </div>

  {/if}
</div>

<style>
  .hex-viewer {
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Courier New', Courier, monospace;
    font-size: 13px;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: #1e1e1e;
  }

  /* ── Header ── */
  .hex-header {
    display: flex;
    align-items: stretch;
    height: 22px;
    background: #252526;
    border-bottom: 2px solid #3c3c3c;
    flex-shrink: 0;
    user-select: none;
    font-size: 13px;
    font-weight: 400;
    letter-spacing: 0;
    color: #666;
  }

  /* ── Scroll container ── */
  .hex-scroll {
    flex: 1;
    overflow-y: auto;
    overflow-x: auto;
    position: relative;
  }

  .scroll-space {
    position: relative;
    width: 100%;
  }

  .visible-rows {
    position: absolute;
    left: 0;
    right: 0;
  }

  /* ── Row ── */
  .hex-row {
    display: flex;
    align-items: stretch;
    height: 20px;         /* must match ROW_HEIGHT */
  }

  .hex-row:nth-child(odd) {
    background: rgba(255, 255, 255, 0.015);
  }

  /* On hover: flatten per-cell backgrounds, apply uniform highlight */
  .hex-row:hover {
    background: rgba(255, 255, 255, 0.055);
  }

  .hex-row:hover .hb.ec-0,
  .hex-row:hover .hb.ec-1 {
    background: transparent;
  }

  /* ── Address column ── */
  .col-addr {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    width: calc(8ch + 22px);
    padding: 0 10px 0 12px;
    color: #569cd6;
  }

  .v-sep {
    width: 2px;
    flex-shrink: 0;
    background: #3c3c3c;
    align-self: stretch;
  }

  /* ── Hex bytes area ── */
  .col-hex-area {
    display: flex;
    align-items: stretch;
    padding: 0 8px;
    flex-shrink: 0;
  }

  /* Individual hex byte cell — 3ch = "XX " */
  .hb {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 3ch;
    flex-shrink: 0;
    color: #d4d4d4;
  }

  /* Alternating column shading */
  .hb.ec-0 {
    background: rgba(255, 255, 255, 0.05);
  }

  .hb.pad {
    color: transparent;
  }

  /* Mid-row gap between byte 7 and byte 8 */
  .mid-gap {
    width: 8px;
    flex-shrink: 0;
  }

  /* ── ASCII area ── */
  .col-ascii-area {
    display: flex;
    align-items: stretch;
    padding: 0 8px;
  }

  /* Individual ASCII char cell */
  .ac {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1ch;
    padding: 0 2px;
    flex-shrink: 0;
    color: #9cdcfe;
  }

  /* Non-printable dot */
  .ac.np {
    color: #444;
  }

  /* ── Row-count footer ── */
  .row-count {
    padding: 3px 12px;
    font-size: 11px;
    color: #555;
    border-top: 1px solid #2a2a2a;
    flex-shrink: 0;
    user-select: none;
  }

  /* ── Empty state ── */
  .empty-state {
    color: #555;
    text-align: center;
    margin-top: 4rem;
  }

  /* ── Light mode ── */
  @media (prefers-color-scheme: light) {
    .hex-viewer             { background: #fff; }
    .hex-header             { background: #f5f5f5; border-bottom-color: #d0d0d0; }
    .hex-row:nth-child(odd) { background: rgba(0, 0, 0, 0.01); }
    .hex-row:hover          { background: rgba(0, 120, 212, 0.07); }
    .col-addr               { color: #0070c1; }
    .v-sep                  { background: #d0d0d0; }
    .hb                     { color: #1e1e1e; }
    .hb.ec-0                { background: rgba(0, 0, 0, 0.04); }
    .ac                     { color: #267f99; }
    .ac.np                  { color: #c8c8c8; }
    .row-count              { color: #aaa; border-top-color: #eee; }
  }
</style>
