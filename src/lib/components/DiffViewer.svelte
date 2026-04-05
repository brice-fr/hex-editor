<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  /**
   * DiffViewer — side-by-side hex diff of two firmware files.
   *
   * Layout per data row:
   *   [ref hex bytes]   ADDRESS   [cmp hex bytes]
   *
   * Identical aligned sections are collapsed into a single gap marker row.
   *   • Double-click a gap marker  → expand it (show individual identical rows)
   *   • Double-click a same row    → collapse it back to a gap marker
   *
   * Cell status colours:
   *   same      → default text
   *   changed   → --c-diff-changed  (yellow)
   *   ref-only  → --c-diff-ref-only (blue)  — cmp side shows "··"
   *   cmp-only  → --c-diff-cmp-only (green) — ref side shows "··"
   *   empty     → dim "··" on both sides
   */

  let {
    refPath     = '',
    cmpPath     = '',
    refRecords  = [],
    cmpRecords  = [],
    bytesPerRow = 16,
  } = $props();

  const ITEM_HEIGHT = 22;  // px — data rows
  const GAP_HEIGHT  = 30;  // px — gap-marker rows
  const BUFFER      = 8;   // extra rows rendered above/below viewport

  // ── Step 1: build raw rows (no collapsing) ──────────────────────────────────

  function buildRawRows(refRecs, cmpRecs, bpr) {
    const refMap = new Map();
    const cmpMap = new Map();

    const isData = (t) => t === 'Data' || t === 'S1' || t === 'S2' || t === 'S3';
    for (const rec of refRecs) {
      if (!isData(rec.record_type)) continue;
      rec.data.forEach((b, i) => refMap.set(rec.address + i, b));
    }
    for (const rec of cmpRecs) {
      if (!isData(rec.record_type)) continue;
      rec.data.forEach((b, i) => cmpMap.set(rec.address + i, b));
    }

    if (refMap.size === 0 && cmpMap.size === 0) return [];

    let minAddr = Infinity, maxAddr = -Infinity;
    for (const a of refMap.keys()) { if (a < minAddr) minAddr = a; if (a > maxAddr) maxAddr = a; }
    for (const a of cmpMap.keys()) { if (a < minAddr) minAddr = a; if (a > maxAddr) maxAddr = a; }
    const rowStart = Math.floor(minAddr / bpr) * bpr;
    const rowEnd   = Math.ceil((maxAddr + 1) / bpr) * bpr;

    const rows = [];
    let emptyRunStart = null;
    for (let addr = rowStart; addr < rowEnd; addr += bpr) {
      let hasData = false;
      for (let i = 0; i < bpr; i++) {
        if (refMap.has(addr + i) || cmpMap.has(addr + i)) { hasData = true; break; }
      }
      if (!hasData) {
        if (emptyRunStart === null) emptyRunStart = addr;
        continue;
      }
      if (emptyRunStart !== null) {
        rows.push({ type: 'addr-gap', addr: emptyRunStart, toAddr: addr });
        emptyRunStart = null;
      }

      const cells = [];
      let allSame = true;
      for (let i = 0; i < bpr; i++) {
        const a  = addr + i;
        const rb = refMap.get(a);
        const cb = cmpMap.get(a);
        let status;
        if      (rb === undefined && cb === undefined)              status = 'empty';
        else if (rb !== undefined && cb !== undefined && rb === cb) status = 'same';
        else if (rb !== undefined && cb !== undefined)              status = 'changed';
        else if (rb !== undefined)                                  status = 'ref-only';
        else                                                        status = 'cmp-only';
        if (status !== 'same' && status !== 'empty') allSame = false;
        cells.push({ rb, cb, status });
      }
      const nonEmptyCount = cells.filter(c => c.status !== 'empty').length;
      rows.push({ type: 'row', addr, cells, allSame, nonEmptyCount });
    }
    return rows;
  }

  // ── Step 2: build display items — collapse same runs or expand them ─────────

  function buildDisplayItems(rawRows, expanded, bpr) {
    const result = [];
    let gapStart = null, gapBytes = 0, gapRows = [], gapLastAddr = 0;

    for (const row of rawRows) {
      if (row.type === 'addr-gap') {
        if (gapStart !== null) {
          if (expanded.includes(gapStart)) {
            for (const r of gapRows) result.push({ ...r, gapAddr: gapStart });
          } else {
            result.push({ type: 'gap', addr: gapStart, lastAddr: gapLastAddr + bpr - 1, count: gapBytes });
          }
          gapStart = null; gapBytes = 0; gapRows = []; gapLastAddr = 0;
        }
        result.push(row);
        continue;
      }
      if (row.allSame) {
        if (gapStart === null) gapStart = row.addr;
        gapBytes += row.nonEmptyCount;
        gapLastAddr = row.addr;
        gapRows.push(row);
      } else {
        if (gapStart !== null) {
          if (expanded.includes(gapStart)) {
            for (const r of gapRows) result.push({ ...r, gapAddr: gapStart });
          } else {
            result.push({ type: 'gap', addr: gapStart, lastAddr: gapLastAddr + bpr - 1, count: gapBytes });
          }
          gapStart = null; gapBytes = 0; gapRows = []; gapLastAddr = 0;
        }
        result.push(row);
      }
    }
    if (gapStart !== null) {
      if (expanded.includes(gapStart)) {
        for (const r of gapRows) result.push({ ...r, gapAddr: gapStart });
      } else {
        result.push({ type: 'gap', addr: gapStart, lastAddr: gapLastAddr + bpr - 1, count: gapBytes });
      }
    }
    return result;
  }

  // ── Expand/collapse state ───────────────────────────────────────────────────

  let expandedGaps = $state(/** @type {number[]} */ ([]));

  function expandGap(addr) {
    if (!expandedGaps.includes(addr)) expandedGaps.push(addr);
  }
  function collapseGap(addr) {
    const idx = expandedGaps.indexOf(addr);
    if (idx !== -1) expandedGaps.splice(idx, 1);
  }

  // ── Derived state ───────────────────────────────────────────────────────────

  const rawRows     = $derived(buildRawRows(refRecords, cmpRecords, bytesPerRow));
  const changedRows = $derived(rawRows.filter(r => r.type === 'row' && !r.allSame).length);

  // Per-gap row counts — used for stats and expand/collapse all
  const gapInfo = $derived.by(() => {
    const gaps = [];
    let gapStart = null, gapCount = 0;
    for (const row of rawRows) {
      if (row.type === 'addr-gap') {
        if (gapStart !== null) { gaps.push({ addr: gapStart, count: gapCount }); gapStart = null; gapCount = 0; }
        continue;
      }
      if (row.allSame) {
        if (gapStart === null) gapStart = row.addr;
        gapCount++;
      } else {
        if (gapStart !== null) { gaps.push({ addr: gapStart, count: gapCount }); gapStart = null; gapCount = 0; }
      }
    }
    if (gapStart !== null) gaps.push({ addr: gapStart, count: gapCount });
    return gaps;
  });

  const collapsedRows = $derived(
    gapInfo.filter(g => !expandedGaps.includes(g.addr)).reduce((s, g) => s + g.count, 0)
  );
  const shownSameRows = $derived(
    gapInfo.filter(g =>  expandedGaps.includes(g.addr)).reduce((s, g) => s + g.count, 0)
  );

  function expandAll() {
    for (const g of gapInfo) {
      if (!expandedGaps.includes(g.addr)) expandedGaps.push(g.addr);
    }
  }
  function collapseAll() {
    expandedGaps.length = 0;
  }

  const items = $derived.by(() => buildDisplayItems(rawRows, expandedGaps, bytesPerRow));

  // Per-item heights → prefix-sum array for virtual scroll
  const offsets = $derived.by(() => {
    const arr = new Float64Array(items.length + 1);
    for (let i = 0; i < items.length; i++) {
      arr[i + 1] = arr[i] + (items[i].type === 'gap' || items[i].type === 'addr-gap' ? GAP_HEIGHT : ITEM_HEIGHT);
    }
    return arr;
  });
  const totalHeight = $derived(offsets[items.length]);

  function firstVisible(top) {
    let lo = 0, hi = items.length;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (offsets[mid + 1] <= top) lo = mid + 1; else hi = mid;
    }
    return lo;
  }

  let scrollTop  = $state(0);
  let containerH = $state(600);

  const COL_HEADER_H = 22;
  const visStart = $derived(Math.max(0, firstVisible(scrollTop) - BUFFER));
  const visEnd   = $derived(Math.min(items.length, firstVisible(scrollTop + containerH - COL_HEADER_H) + BUFFER));
  const visItems = $derived(items.slice(visStart, visEnd));
  const padTop   = $derived(offsets[visStart]);
  const padBot   = $derived(totalHeight - offsets[visEnd]);

  // ── Helpers ─────────────────────────────────────────────────────────────────

  function hex8(n) { return n.toString(16).padStart(8, '0').toUpperCase(); }
  function hex2(n) { return n.toString(16).padStart(2, '0').toUpperCase(); }
  function lastName(path) { return path ? path.split('/').at(-1) : '—'; }

  function itemKey(item) {
    if (item.type === 'gap')      return `gap-${item.addr}`;
    if (item.type === 'addr-gap') return `addr-gap-${item.addr}`;
    if (item.gapAddr !== undefined) return `same-${item.addr}`;
    return `row-${item.addr}`;
  }
</script>

<div class="diff-shell">

  <!-- File names header -->
  <div class="diff-header">
    <div class="side-label ref-label" title={refPath}>{lastName(refPath)}</div>
    <div class="addr-col-label"></div>
    <div class="side-label cmp-label" title={cmpPath}>{lastName(cmpPath)}</div>
  </div>

  <!-- Virtual scroll container — col-header is the first child so it shares
       the same content width as data rows (scrollbar-aware), then sticks to top. -->
  <div
    class="diff-scroll"
    bind:clientHeight={containerH}
    onscroll={(e) => { scrollTop = e.currentTarget.scrollTop; }}
  >

    <!-- Column index header (sticky) -->
    <div class="col-header">
      <div class="hex-side ref-side">
        {#each { length: bytesPerRow } as _, i}
          {#if i === bytesPerRow / 2}<span class="half-gap"></span>{/if}
          <span class="byte-cell ec-{i % 2} col-idx">{(i).toString(16).padStart(2, '0').toUpperCase()}</span>
        {/each}
      </div>
      <span class="v-sep"></span>
      <div class="addr-col col-addr-label">Address</div>
      <span class="v-sep"></span>
      <div class="hex-side cmp-side">
        {#each { length: bytesPerRow } as _, i}
          {#if i === bytesPerRow / 2}<span class="half-gap"></span>{/if}
          <span class="byte-cell ec-{i % 2} col-idx">{(i).toString(16).padStart(2, '0').toUpperCase()}</span>
        {/each}
      </div>
    </div>

    <div class="scroll-space" style="height:{totalHeight}px; position:relative;">
      <div style="position:absolute; top:{padTop}px; left:0; right:0;">

        {#each visItems as item (itemKey(item))}

          {#if item.type === 'addr-gap'}
            <div class="addr-gap-row" style="height:{GAP_HEIGHT}px">
              <span class="gap-line"></span>
              <span class="gap-text addr-gap-text">
                gap: 0x{(item.toAddr - item.addr).toString(16).toUpperCase()} bytes ({(item.toAddr - item.addr).toLocaleString()}) · {hex8(item.addr)} – {hex8(item.toAddr - 1)}
              </span>
              <span class="gap-line"></span>
            </div>

          {:else if item.type === 'gap'}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="gap-row"
              style="height:{GAP_HEIGHT}px"
              title="Double-click to expand"
              ondblclick={() => expandGap(item.addr)}
            >
              <span class="gap-line"></span>
              <span class="gap-text">
                {hex8(item.addr)} – {hex8(item.lastAddr)}
                &nbsp;·&nbsp;
                {item.count.toLocaleString()} identical byte{item.count === 1 ? '' : 's'}
              </span>
              <span class="gap-line"></span>
            </div>

          {:else if item.gapAddr !== undefined}
            <!-- Expanded same row — double-click collapses back -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="data-row same-row"
              class:row-alt={Math.floor(item.addr / bytesPerRow) % 2 === 1}
              style="height:{ITEM_HEIGHT}px"
              title="Double-click to collapse"
              ondblclick={() => collapseGap(item.gapAddr)}
            >
              <div class="hex-side ref-side">
                {#each item.cells as cell, ci}
                  {#if ci === bytesPerRow / 2}<span class="half-gap"></span>{/if}
                  <span class="byte-cell {cell.status} ec-{ci % 2}">{cell.rb !== undefined ? hex2(cell.rb) : '··'}</span>
                {/each}
              </div>
              <span class="v-sep"></span>
              <div class="addr-col">{hex8(item.addr)}</div>
              <span class="v-sep"></span>
              <div class="hex-side cmp-side">
                {#each item.cells as cell, ci}
                  {#if ci === bytesPerRow / 2}<span class="half-gap"></span>{/if}
                  <span class="byte-cell {cell.status} ec-{ci % 2}">{cell.cb !== undefined ? hex2(cell.cb) : '··'}</span>
                {/each}
              </div>
            </div>

          {:else}
            <!-- Diff row -->
            <div
              class="data-row"
              class:row-alt={Math.floor(item.addr / bytesPerRow) % 2 === 1}
              style="height:{ITEM_HEIGHT}px"
            >
              <div class="hex-side ref-side">
                {#each item.cells as cell, ci}
                  {#if ci === bytesPerRow / 2}<span class="half-gap"></span>{/if}
                  <span class="byte-cell {cell.status} ec-{ci % 2}">{cell.rb !== undefined ? hex2(cell.rb) : '··'}</span>
                {/each}
              </div>
              <span class="v-sep"></span>
              <div class="addr-col">{hex8(item.addr)}</div>
              <span class="v-sep"></span>
              <div class="hex-side cmp-side">
                {#each item.cells as cell, ci}
                  {#if ci === bytesPerRow / 2}<span class="half-gap"></span>{/if}
                  <span class="byte-cell {cell.status} ec-{ci % 2}">{cell.cb !== undefined ? hex2(cell.cb) : '··'}</span>
                {/each}
              </div>
            </div>
          {/if}

        {/each}

      </div>
    </div>
  </div>

  <!-- Stats bar -->
  <div class="stats-bar">
    {#if rawRows.length === 0 || changedRows === 0}
      <span>Files are identical — no differences found.</span>
    {:else}
      <span class="stat changed">{changedRows} difference row{changedRows === 1 ? '' : 's'}</span>
      {#if collapsedRows > 0}
        <span class="stat-sep">·</span>
        <span class="stat same">{collapsedRows.toLocaleString()} identical row{collapsedRows === 1 ? '' : 's'} collapsed</span>
      {/if}
      {#if shownSameRows > 0}
        <span class="stat-sep">·</span>
        <span class="stat same">{shownSameRows.toLocaleString()} identical row{shownSameRows === 1 ? '' : 's'} shown</span>
      {/if}
    {/if}
    <span class="stat-actions">
      {#if collapsedRows > 0}
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a href="#" class="stat-link" onclick={(e) => { e.preventDefault(); expandAll(); }}>expand all</a>
      {/if}
      {#if collapsedRows > 0 && shownSameRows > 0}
        <span class="stat-sep">–</span>
      {/if}
      {#if shownSameRows > 0}
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a href="#" class="stat-link" onclick={(e) => { e.preventDefault(); collapseAll(); }}>collapse all</a>
      {/if}
    </span>
  </div>

  <!-- Legend -->
  <div class="legend">
    <span class="leg-item"><span class="leg-swatch ref-only"></span>Reference file</span>
    <span class="leg-item"><span class="leg-swatch cmp-only"></span>Compare file</span>
  </div>

</div>

<style>
  /* ── Shell ─────────────────────────────────────────────────── */
  .diff-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--c-bg);
    color: var(--c-text);
    overflow: hidden;
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Consolas', monospace;
    font-size: 12px;
  }

  /* ── Header ────────────────────────────────────────────────── */
  .diff-header {
    display: flex;
    align-items: center;
    background: var(--c-header-bg);
    border-bottom: 1px solid var(--c-border);
    padding: 0 12px;
    height: 36px;
    flex-shrink: 0;
  }

  .side-label {
    flex: 1;
    font-size: 11.5px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    padding: 0 4px;
  }

  .ref-label { color: var(--c-diff-ref-only); text-align: right; }
  .cmp-label { color: var(--c-diff-cmp-only); text-align: left;  }

  /* spacer that holds the width of the address column in the file-names header */
  .addr-col-label {
    flex-shrink: 0;
    width: calc(90px + 4px); /* addr-col + 2 × v-sep */
  }

  /* ── Column index header ───────────────────────────────────── */
  .col-header {
    position: sticky;
    top: 0;
    z-index: 1;
    display: flex;
    align-items: stretch;
    justify-content: center;
    padding: 0 12px;
    background: var(--c-surface);
    border-bottom: 2px solid var(--c-hover);
    height: 22px;
    color: var(--c-dim);
  }

  .col-idx   { color: var(--c-dim); }
  .col-addr-label { color: var(--c-addr); }

  /* ── Stats bar ─────────────────────────────────────────────── */
  .stats-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 14px;
    background: var(--c-surface);
    border-top: 1px solid var(--c-border);
    font-size: 11px;
    flex-shrink: 0;
    color: var(--c-muted);
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  }

  .stat.changed { color: var(--c-diff-changed); }
  .stat.same    { color: var(--c-muted); }
  .stat-sep     { color: var(--c-dim); }

  .stat-actions { margin-left: auto; display: flex; align-items: center; gap: 6px; }

  .stat-link {
    color: var(--c-accent-t);
    text-decoration: none;
    cursor: pointer;
  }
  .stat-link:hover { text-decoration: underline; }

  /* ── Scroll area ───────────────────────────────────────────── */
  .diff-scroll {
    flex: 1;
    overflow-y: scroll;
    overflow-x: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--c-scrollbar-thumb) var(--c-scrollbar-track);
  }

  .diff-scroll::-webkit-scrollbar       { width: 8px; height: 8px; }
  .diff-scroll::-webkit-scrollbar-track { background: var(--c-scrollbar-track); }
  .diff-scroll::-webkit-scrollbar-thumb { background: var(--c-scrollbar-thumb); border-radius: 4px; }

  /* ── Address gap row (no data in either file) ─────────────── */
  .addr-gap-row {
    display: flex;
    align-items: center;
    padding: 0 14px;
    gap: 10px;
    background: var(--c-gap-bg);
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    user-select: none;
  }

  .addr-gap-text {
    color: var(--c-gap-text) !important;
  }

  /* ── Gap row ───────────────────────────────────────────────── */
  .gap-row {
    display: flex;
    align-items: center;
    padding: 0 14px;
    gap: 10px;
    cursor: pointer;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    user-select: none;
  }

  .gap-row:hover .gap-text { color: var(--c-text2); }
  .gap-row:hover .gap-line { background: var(--c-border2); }

  .gap-line {
    flex: 1;
    height: 1px;
    background: var(--c-border);
    transition: background 0.1s;
  }

  .gap-text {
    flex-shrink: 0;
    font-size: 11px;
    color: var(--c-dim);
    letter-spacing: 0.03em;
    white-space: nowrap;
    transition: color 0.1s;
  }

  /* ── Vertical separators (between hex sides and address column) */
  .v-sep {
    width: 2px;
    flex-shrink: 0;
    background: var(--c-hover);
    align-self: stretch;
  }

  /* ── Data row ──────────────────────────────────────────────── */
  .data-row {
    display: flex;
    align-items: stretch;
    justify-content: center;
    padding: 0 12px;
  }

  .data-row.row-alt { background: var(--c-ec1); }

  .data-row:hover { background: var(--c-hover); }

  /* Flatten per-cell column tint on hover */
  .data-row:hover .byte-cell.ec-0 { background: transparent; }

  /* Expanded same rows — pointer cursor */
  .same-row { cursor: pointer; }

  /* ── Hex sides ─────────────────────────────────────────────── */
  /* flex-shrink:0 gives each side a fixed intrinsic width (content-driven),
     exactly like HexViewer's col-hex-area.  Combined with justify-content:center
     on the row, every byte column lands at the same x-position in both the
     sticky col-header and every data row, regardless of scroll-container width. */
  .hex-side {
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  .ref-side { padding-right: 8px; }
  .cmp-side { padding-left:  8px; }

  .half-gap {
    width: 8px;
    flex-shrink: 0;
  }

  /* ── Byte cells ────────────────────────────────────────────── */
  .byte-cell {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 3ch;
    flex-shrink: 0;
    border-radius: 2px;
    color: var(--c-text);
  }

  /* Alternating column tint — only visible on same/empty cells (diff rules override) */
  .byte-cell.ec-0 { background: var(--c-ec1); }

  /* ref side: always blue for any differing position */
  .ref-side .byte-cell.changed,
  .ref-side .byte-cell.ref-only,
  .ref-side .byte-cell.cmp-only { color: var(--c-diff-ref-only); background: var(--c-diff-ref-only-bg); }

  /* cmp side: always green for any differing position */
  .cmp-side .byte-cell.changed,
  .cmp-side .byte-cell.cmp-only,
  .cmp-side .byte-cell.ref-only { color: var(--c-diff-cmp-only); background: var(--c-diff-cmp-only-bg); }

  .byte-cell.empty { color: var(--c-dim); }

  /* ── Address column ────────────────────────────────────────── */
  .addr-col {
    flex-shrink: 0;
    width: 90px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--c-addr);
    font-size: 11px;
    letter-spacing: 0.04em;
  }

  /* ── Legend ────────────────────────────────────────────────── */
  .legend {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 5px 14px;
    border-top: 1px solid var(--c-border);
    background: var(--c-surface);
    font-size: 10.5px;
    color: var(--c-muted);
    flex-shrink: 0;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  }

  .leg-item {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .leg-swatch {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 2px;
  }

  .leg-swatch.ref-only { background: var(--c-diff-ref-only); }
  .leg-swatch.cmp-only { background: var(--c-diff-cmp-only); }
</style>
