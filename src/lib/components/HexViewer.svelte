<script>
  /** @type {{ record_type: string; address: number; data: number[] }[]} */
  let { records = [], bytesPerRow = 16 } = $props();

  const ROW_HEIGHT = 20; // px — must match CSS .hex-row height
  const OVERSCAN   = 8;  // extra rows rendered above and below the visible area

  // ── Flatten all Data records into display rows ──────────────────────────────
  function buildRows(records, bytesPerRow) {
    const rows = [];
    for (const rec of records) {
      if (rec.record_type !== 'Data' || rec.data.length === 0) continue;
      for (let i = 0; i < rec.data.length; i += bytesPerRow) {
        const bytes = rec.data.slice(i, i + bytesPerRow);
        rows.push({
          address: rec.address + i,
          bytes,
          ascii: bytes
            .map((b) => (b >= 0x20 && b < 0x7f ? String.fromCharCode(b) : '.'))
            .join(''),
        });
      }
    }
    return rows;
  }

  const rows = $derived(buildRows(records, bytesPerRow));

  // ── Virtual-scroll state ────────────────────────────────────────────────────
  let scrollTop    = $state(0);
  let clientHeight = $state(600); // updated by bind:clientHeight after mount

  const totalHeight = $derived(rows.length * ROW_HEIGHT);

  const startIdx = $derived(
    Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN)
  );
  const endIdx = $derived(
    Math.min(rows.length, Math.ceil((scrollTop + clientHeight) / ROW_HEIGHT) + OVERSCAN)
  );
  const visibleRows = $derived(rows.slice(startIdx, endIdx));
  const offsetTop   = $derived(startIdx * ROW_HEIGHT);

  function onScroll(e) {
    scrollTop = e.currentTarget.scrollTop;
  }

  // ── Formatters ──────────────────────────────────────────────────────────────
  function hex8(n)  { return n.toString(16).padStart(2, '0').toUpperCase(); }
  function hex32(n) { return n.toString(16).padStart(8, '0').toUpperCase(); }

  function formatHex(bytes, bytesPerRow) {
    const parts = [];
    for (let i = 0; i < bytes.length; i++) {
      if (i === bytesPerRow / 2) parts.push(' '); // mid-row gap
      parts.push(hex8(bytes[i]));
      if (i < bytes.length - 1) parts.push(' ');
    }
    // Pad short last row so the ASCII column stays aligned
    const missing = bytesPerRow - bytes.length;
    for (let i = 0; i < missing; i++) {
      if (bytes.length + i === bytesPerRow / 2) parts.push(' ');
      parts.push('   ');
    }
    return parts.join('');
  }
</script>

<div class="hex-viewer">
  {#if rows.length === 0}
    <p class="empty-state">No data to display. Open an Intel HEX or S-record file.</p>
  {:else}
    <!-- ── Sticky column header ── -->
    <div class="hex-header">
      <span class="col-addr">Address </span>
      <span class="col-hex">{"00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F"}</span>
      <span class="col-sep"></span>
      <span class="col-ascii">0123456789ABCDEF</span>
    </div>

    <!-- ── Virtual-scroll container ── -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="hex-scroll"
      role="grid"
      aria-rowcount={rows.length}
      onscroll={onScroll}
      bind:clientHeight
    >
      <!-- Full-height spacer — gives the scrollbar the correct thumb size -->
      <div class="scroll-space" style="height:{totalHeight}px;">
        <!-- Visible rows pinned to their absolute position -->
        <div class="visible-rows" style="top:{offsetTop}px;">
          {#each visibleRows as row (row.address)}
            <div class="hex-row" role="row">
              <span class="col-addr">{hex32(row.address)}</span>
              <span class="col-hex">{formatHex(row.bytes, bytesPerRow)}</span>
              <span class="col-sep">│</span>
              <span class="col-ascii">{row.ascii}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <div class="statusbar">
      {rows.length.toLocaleString()} rows &nbsp;·&nbsp;
      {(rows.length * bytesPerRow).toLocaleString()} bytes
    </div>
  {/if}
</div>

<style>
  .hex-viewer {
    font-family: 'Courier New', Courier, monospace;
    font-size: 13px;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* ── Header ── */
  .hex-header {
    display: flex;
    gap: 12px;
    padding: 3px 12px;
    background: #252526;
    border-bottom: 1px solid #444;
    color: #888;
    flex-shrink: 0;
    user-select: none;
  }

  /* ── Scroll container ── */
  .hex-scroll {
    flex: 1;
    overflow-y: auto;
    overflow-x: auto;
    position: relative;
  }

  /* Full-height invisible div that makes the scrollbar correct */
  .scroll-space {
    position: relative;
    width: 100%;
  }

  /* Rendered rows float at the right offset inside scroll-space */
  .visible-rows {
    position: absolute;
    left: 0;
    right: 0;
  }

  /* ── Row ── */
  .hex-row {
    display: flex;
    gap: 12px;
    padding: 0 12px;
    height: 20px;          /* must match ROW_HEIGHT constant */
    line-height: 20px;
    align-items: center;
  }

  .hex-row:hover {
    background: #2a2d2e;
  }

  /* ── Columns ── */
  .col-addr {
    color: #569cd6;
    flex-shrink: 0;
    width: 8ch;
  }

  .col-hex {
    color: #d4d4d4;
    flex-shrink: 0;
    white-space: pre;
    /* 16 bytes × 3 chars + 1 extra space for mid-gap */
    width: calc(16 * 3ch + 1ch);
  }

  .col-sep {
    color: #555;
    flex-shrink: 0;
  }

  .col-ascii {
    color: #9cdcfe;
    white-space: pre;
    letter-spacing: 0.5px;
  }

  /* ── Status bar ── */
  .statusbar {
    padding: 3px 12px;
    font-size: 11px;
    color: #666;
    border-top: 1px solid #333;
    flex-shrink: 0;
    user-select: none;
  }

  /* ── Empty state ── */
  .empty-state {
    color: #666;
    text-align: center;
    margin-top: 4rem;
  }

  /* ── Light mode ── */
  @media (prefers-color-scheme: light) {
    .hex-header   { background: #f0f0f0; border-bottom-color: #ccc; color: #888; }
    .hex-row:hover { background: #f0f4f8; }
    .col-addr     { color: #0070c1; }
    .col-hex      { color: #1e1e1e; }
    .col-sep      { color: #bbb; }
    .col-ascii    { color: #267f99; }
    .statusbar    { border-top-color: #ddd; color: #999; }
  }
</style>
