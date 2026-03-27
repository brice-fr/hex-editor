<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  /** @type {{ record_type: string; address: number; data: number[] }[]} */
  let { records = [], bytesPerRow = 16, fontSize = 13, onScrolled = () => {}, onTopAddress = (_addr) => {}, onByteClick = (_addr) => {}, onSelectionChange = (_sel) => {}, gotoTarget = null } = $props();

  const ROW_HEIGHT = $derived(fontSize + 7);
  const OVERSCAN   = 8;

  function buildRows(records, bytesPerRow) {
    // 1. Filter to data records with data.length > 0
    const dataRecs = records.filter(rec => {
      const isData = rec.record_type === 'Data'
                  || rec.record_type === 'S1'
                  || rec.record_type === 'S2'
                  || rec.record_type === 'S3';
      return isData && rec.data.length > 0;
    });

    // 2. Sort by address ascending
    dataRecs.sort((a, b) => a.address - b.address);

    // 3. Merge contiguous records (gap == 0) into segments
    /** @type {{ address: number; data: number[] }[]} */
    const segments = [];
    for (const rec of dataRecs) {
      if (segments.length === 0) {
        segments.push({ address: rec.address, data: [...rec.data] });
      } else {
        const last = segments[segments.length - 1];
        const gap = rec.address - (last.address + last.data.length);
        if (gap === 0) {
          // Contiguous — merge
          last.data.push(...rec.data);
        } else {
          segments.push({ address: rec.address, data: [...rec.data] });
        }
      }
    }

    // 4. Build rows from segments
    const rows = [];
    let counter = 0;

    for (let si = 0; si < segments.length; si++) {
      const seg = segments[si];

      // 4a. Insert gap row if not the first segment
      if (si > 0) {
        const prevSeg = segments[si - 1];
        const gap = seg.address - (prevSeg.address + prevSeg.data.length);
        rows.push({
          type: 'gap',
          id: `gap_${counter++}`,
          gapBytes: gap,
          fromAddr: prevSeg.address + prevSeg.data.length,
          toAddr: seg.address,
        });
      }

      // 4b. Align start to bytesPerRow boundary
      const alignedStart = Math.floor(seg.address / bytesPerRow) * bytesPerRow;
      const leadingBlanks = seg.address - alignedStart;
      const padded = [...Array(leadingBlanks).fill(null), ...seg.data];

      // 4c. Emit data rows
      for (let i = 0; i < padded.length; i += bytesPerRow) {
        rows.push({
          type: 'data',
          id: `data_${counter++}`,
          address: alignedStart + i,
          bytes: padded.slice(i, i + bytesPerRow),
        });
      }
    }

    return rows;
  }

  const rows = $derived(buildRows(records, bytesPerRow));

  const segmentCount = $derived(rows.filter(r => r.type === 'gap').length + (rows.length > 0 ? 1 : 0));
  const dataRowCount = $derived(rows.filter(r => r.type === 'data').length);
  const totalDataBytes = $derived(
    rows.filter(r => r.type === 'data')
        .reduce((sum, r) => sum + r.bytes.filter(b => b !== null).length, 0)
  );

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
    const topIdx = Math.floor(scrollTop / ROW_HEIGHT);
    const topRow = rows.slice(topIdx).find(r => r.type === 'data');
    onTopAddress(topRow?.address ?? 0);
  });

  // Navigate to a target address when gotoTarget changes
  $effect(() => {
    if (!gotoTarget || !scrollEl) return;
    const rowIdx = rows.findIndex(
      r => r.type === 'data' && r.address <= gotoTarget.addr && gotoTarget.addr < r.address + bytesPerRow
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
  function toAscii(b) { return b !== null && b >= 0x20 && b < 0x7f ? String.fromCharCode(b) : '.'; }
  function isPrint(b) { return b !== null && b >= 0x20 && b < 0x7f; }

  const COLS = $derived(Array.from({ length: bytesPerRow }, (_, i) => i));

  // ── Selection ────────────────────────────────────────────────────────────
  let selAnchor  = $state(/** @type {number|null} */ (null));
  let selFocus   = $state(/** @type {number|null} */ (null));
  let isDragging = false;

  const selMin   = $derived(selAnchor !== null && selFocus !== null ? Math.min(selAnchor, selFocus) : null);
  const selMax   = $derived(selAnchor !== null && selFocus !== null ? Math.max(selAnchor, selFocus) : null);
  const selCount = $derived(selMin !== null ? selMax - selMin + 1 : 0);

  // Notify parent whenever selection changes (focus = last byte cursor touched)
  $effect(() => {
    onSelectionChange(selMin !== null ? { start: selMin, end: selMax, count: selCount, focus: selFocus } : null);
  });

  // Clear selection when file changes
  $effect(() => { void records; selAnchor = null; selFocus = null; });

  function onScrollPointerDown(e) {
    if (e.button !== 0) return;
    const byteEl = document.elementFromPoint(e.clientX, e.clientY)?.closest('[data-addr]');
    if (!byteEl) { selAnchor = null; selFocus = null; return; }
    const addr = parseInt(/** @type {HTMLElement} */ (byteEl).dataset.addr ?? '');
    if (isNaN(addr)) return;
    // Toggle off: re-clicking the same single-byte selection clears it
    if (!e.shiftKey && selAnchor === addr && selFocus === addr) {
      selAnchor = null; selFocus = null; return;
    }
    if (e.shiftKey && selAnchor !== null) {
      selFocus = addr;
    } else {
      selAnchor = addr;
      selFocus  = addr;
    }
    isDragging = true;
    e.currentTarget.setPointerCapture(e.pointerId);
  }

  function onScrollPointerMove(e) {
    if (!isDragging) return;
    const byteEl = document.elementFromPoint(e.clientX, e.clientY)?.closest('[data-addr]');
    if (byteEl) {
      const addr = parseInt(/** @type {HTMLElement} */ (byteEl).dataset.addr ?? '');
      if (!isNaN(addr)) selFocus = addr;
    }
  }

  function onScrollPointerUp() { isDragging = false; }

  /** Clear selection — called from parent via Escape */
  export function clearSelection() { selAnchor = null; selFocus = null; }

  // ── Minimap ──────────────────────────────────────────────────────────────
  const MINIMAP_W     = 16;
  let minimapCanvas     = $state(/** @type {HTMLCanvasElement|null} */ (null));
  let minimapH          = $state(0);
  let minimapMode       = 'none'; // 'none' | 'drag' | 'page'
  let minimapDragOffset = 0;      // clientY offset within the band at drag start
  let minimapScrollTimer = null;
  let minimapScrollIntv  = null;

  function drawMinimap() {
    const canvas = minimapCanvas;
    if (!canvas || minimapH < 2) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    // Resize the canvas here (not via Svelte binding) so the size update and
    // the draw happen in the same synchronous call — avoids a blank frame
    // where Svelte's DOM update clears the canvas before $effect redraws it.
    if (canvas.height !== minimapH) canvas.height = minimapH;
    const W = MINIMAP_W, H = minimapH, total = rows.length;
    if (total === 0) { ctx.clearRect(0, 0, W, H); return; }

    // Per-pixel row fill
    const img = ctx.createImageData(W, H);
    const px  = img.data;
    for (let py = 0; py < H; py++) {
      const ri     = Math.min(total - 1, Math.floor(py / H * total));
      const isData = rows[ri].type === 'data';
      const r = isData ? 74  : 36;
      const g = isData ? 144 : 36;
      const b = isData ? 226 : 40;
      for (let x = 0; x < W; x++) {
        const i = (py * W + x) * 4;
        px[i] = r; px[i+1] = g; px[i+2] = b; px[i+3] = 255;
      }
    }
    ctx.putImageData(img, 0, 0);

    // Viewport band
    const visRows = Math.ceil(clientHeight / ROW_HEIGHT);
    const topRow  = Math.floor(scrollTop / ROW_HEIGHT);
    const bandTop = Math.round(topRow / total * H);
    const bandH   = Math.max(3, Math.round(visRows / total * H));
    ctx.fillStyle   = 'rgba(255,255,255,0.18)';
    ctx.fillRect(0, bandTop, W, bandH);
    ctx.strokeStyle = 'rgba(255,255,255,0.55)';
    ctx.lineWidth   = 1;
    ctx.strokeRect(0.5, bandTop + 0.5, W - 1, bandH - 1);
  }

  // Redraw whenever any dependency changes
  $effect(() => { void rows; void scrollTop; void clientHeight; void minimapH; void minimapCanvas; void ROW_HEIGHT; drawMinimap(); });

  /** Returns the current viewport band geometry in minimap pixels. */
  function minimapBand() {
    const total = rows.length;
    if (!total || !minimapH) return { top: 0, h: minimapH };
    const visRows = Math.ceil(clientHeight / ROW_HEIGHT);
    const topRow  = Math.floor(scrollTop / ROW_HEIGHT);
    const top     = Math.round(topRow / total * minimapH);
    const h       = Math.max(3, Math.round(visRows / total * minimapH));
    return { top, h };
  }

  function onMinimapDown(e) {
    e.currentTarget.setPointerCapture(e.pointerId);
    const rect = e.currentTarget.getBoundingClientRect();
    const y = e.clientY - rect.top;
    const { top, h } = minimapBand();
    if (y >= top && y <= top + h) {
      // Drag the band
      minimapMode = 'drag';
      minimapDragOffset = y - top;
    } else {
      // Page scroll above or below the band
      minimapMode = 'page';
      const dir = y < top ? -1 : 1;
      if (scrollEl) scrollEl.scrollTop += dir * clientHeight;
      minimapScrollTimer = setTimeout(() => {
        minimapScrollIntv = setInterval(() => {
          if (scrollEl) scrollEl.scrollTop += dir * clientHeight;
        }, 80);
      }, 350);
    }
  }

  function onMinimapMove(e) {
    if (minimapMode !== 'drag') return;
    const total = rows.length;
    if (!total || !minimapH) return;
    const { h } = minimapBand();
    const rect = e.currentTarget.getBoundingClientRect();
    const y = e.clientY - rect.top;
    const newTop = Math.max(0, Math.min(minimapH - h, y - minimapDragOffset));
    const range  = minimapH - h;
    const frac   = range > 0 ? newTop / range : 0;
    if (scrollEl) scrollEl.scrollTop = frac * (scrollEl.scrollHeight - scrollEl.clientHeight);
  }

  function onMinimapUp() {
    clearTimeout(minimapScrollTimer);
    clearInterval(minimapScrollIntv);
    minimapScrollTimer = null;
    minimapScrollIntv  = null;
    minimapMode = 'none';
  }

  // ── Scroll buttons ────────────────────────────────────────────────────────
  let scrollBtnInterval = null;

  function startScrollBtn(dir) {
    if (scrollEl) scrollEl.scrollTop += dir * ROW_HEIGHT * 3;
    scrollBtnInterval = setInterval(() => {
      if (scrollEl) scrollEl.scrollTop += dir * ROW_HEIGHT * 3;
    }, 80);
  }

  function stopScrollBtn() {
    if (scrollBtnInterval) { clearInterval(scrollBtnInterval); scrollBtnInterval = null; }
  }

  $effect(() => () => stopScrollBtn());
</script>

<div class="hex-viewer" style="font-size:{fontSize}px">
  {#if rows.length === 0}
    <p class="empty-state">No data to display. Open an Intel HEX or S-record file.</p>
  {:else}

    <!-- ── Sticky header ── -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="hex-header" onpointerdown={() => { selAnchor = null; selFocus = null; }}>
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

    <!-- ── Scroll area + minimap ── -->
    <div class="scroll-and-minimap">

      <!-- ── Virtual-scroll container ── -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        class="hex-scroll"
        role="grid"
        aria-rowcount={rows.length}
        onscroll={onScroll}
        onpointerdown={onScrollPointerDown}
        onpointermove={onScrollPointerMove}
        onpointerup={onScrollPointerUp}
        bind:clientHeight
        bind:this={scrollEl}
      >
        <div class="scroll-space" style="height:{totalHeight}px;">
          <div class="visible-rows" style="top:{offsetTop}px;">
            {#each visibleRows as row (row.id)}
              {#if row.type === 'gap'}
                <div class="gap-row" style="height:{ROW_HEIGHT}px;">
                  <div class="gap-line"></div>
                  <span class="gap-label">gap: 0x{row.gapBytes.toString(16).toUpperCase()} bytes ({row.gapBytes}) · {hex32(row.fromAddr)} – {hex32(row.toAddr - 1)}</span>
                  <div class="gap-line"></div>
                </div>
              {:else}
                {@const pad = bytesPerRow - row.bytes.length}
                <div class="hex-row" role="row" style="height:{ROW_HEIGHT}px;">

                  <span class="col-addr">{hex32(row.address)}</span>
                  <span class="v-sep"></span>

                  <span class="col-hex-area">
                    {#each row.bytes as byte, i}
                      {#if i === 8}<span class="mid-gap"></span>{/if}
                      {#if byte === null}
                        <span class="hb ec-{i % 2} blank">__</span>
                      {:else}
                        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
                        <span
                          class="hb ec-{i % 2} clickable"
                          class:sel={selMin !== null && selMin <= row.address + i && row.address + i <= selMax}
                          data-addr={row.address + i}
                          onclick={(e) => { e.stopPropagation(); onByteClick(row.address + i); }}
                        >{hex8(byte)}</span>
                      {/if}
                    {/each}
                    {#each { length: pad } as _, i}
                      {@const col = row.bytes.length + i}
                      {#if col === 8}<span class="mid-gap"></span>{/if}
                      <span class="hb ec-{col % 2} pad"></span>
                    {/each}
                  </span>

                  <span class="v-sep"></span>
                  <span class="col-ascii-area">
                    {#each row.bytes as byte, i}
                      {#if byte === null}
                        <span class="ac blank">·</span>
                      {:else}
                        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
                        <span
                          class="ac clickable"
                          class:sel={selMin !== null && selMin <= row.address + i && row.address + i <= selMax}
                          class:np={!isPrint(byte)}
                          data-addr={row.address + i}
                          onclick={(e) => { e.stopPropagation(); onByteClick(row.address + i); }}
                        >{toAscii(byte)}</span>
                      {/if}
                    {/each}
                  </span>

                </div>
              {/if}
            {/each}
          </div>
        </div>
      </div>

      <!-- ── Minimap ── -->
      <div class="minimap-wrap">
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <button
          class="scroll-btn"
          onpointerdown={() => startScrollBtn(-1)}
          onpointerup={stopScrollBtn}
          onpointerleave={stopScrollBtn}
          onpointercancel={stopScrollBtn}
        >▲</button>

        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div
          class="minimap-canvas-area"
          bind:clientHeight={minimapH}
          onpointerdown={onMinimapDown}
          onpointermove={onMinimapMove}
          onpointerup={onMinimapUp}
          onpointercancel={onMinimapUp}
        >
          <canvas bind:this={minimapCanvas} width={MINIMAP_W}></canvas>
        </div>

        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <button
          class="scroll-btn"
          onpointerdown={() => startScrollBtn(1)}
          onpointerup={stopScrollBtn}
          onpointerleave={stopScrollBtn}
          onpointercancel={stopScrollBtn}
        >▼</button>
      </div>

    </div><!-- /scroll-and-minimap -->

    <div class="row-count">
      {dataRowCount.toLocaleString()} rows &nbsp;·&nbsp; {totalDataBytes.toLocaleString()} bytes
      {#if segmentCount > 1}&nbsp;·&nbsp; {segmentCount} segments{/if}
    </div>

  {/if}
</div>

<style>
  .hex-viewer {
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Courier New', Courier, monospace;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--c-bg);
  }

  /* ── Header ── */
  .hex-header {
    display: flex;
    align-items: stretch;
    height: 22px;
    background: var(--c-surface);
    border-bottom: 2px solid var(--c-hover);
    flex-shrink: 0;
    font-weight: 400;
    letter-spacing: 0;
    color: var(--c-dim);
  }

  /* ── Scroll + minimap wrapper ── */
  .scroll-and-minimap {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-height: 0;
  }

  /* ── Scroll container ── */
  .hex-scroll {
    flex: 1;
    overflow-y: scroll;
    overflow-x: auto;
    position: relative;
    scrollbar-width: none; /* Firefox */
    user-select: none;
  }
  .hex-scroll::-webkit-scrollbar { display: none; } /* Chrome/Safari */

  /* ── Minimap ── */
  .minimap-wrap {
    width: 16px;
    flex-shrink: 0;
    background: var(--c-surface);
    border-left: 1px solid var(--c-border);
    display: flex;
    flex-direction: column;
  }

  .minimap-canvas-area {
    flex: 1;
    overflow: hidden;
    cursor: ns-resize;
    user-select: none;
  }

  .minimap-canvas-area canvas {
    display: block;
  }

  .scroll-btn {
    width: 100%;
    height: 16px;
    flex-shrink: 0;
    background: var(--c-surface);
    border: none;
    border-color: var(--c-border);
    color: var(--c-dim);
    font-size: 7px;
    line-height: 1;
    padding: 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    user-select: none;
  }

  .scroll-btn:first-child {
    border-bottom: 1px solid var(--c-border);
  }

  .scroll-btn:last-child {
    border-top: 1px solid var(--c-border);
  }

  .scroll-btn:hover {
    background: var(--c-hover);
    color: var(--c-text);
  }

  .scroll-btn:active {
    background: var(--c-accent-b);
    color: #fff;
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
  }

  .hex-row:nth-child(odd) {
    background: var(--c-ec1);
  }

  /* On hover: flatten per-cell backgrounds, apply uniform highlight */
  .hex-row:hover {
    background: var(--c-hover);
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
    color: var(--c-addr);
  }

  .v-sep {
    width: 2px;
    flex-shrink: 0;
    background: var(--c-hover);
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
    color: var(--c-text);
  }

  /* Alternating column shading */
  .hb.ec-0 {
    background: var(--c-ec1);
  }

  .hb.pad {
    color: transparent;
  }

  .hb.clickable,
  .ac.clickable {
    cursor: pointer;
  }

  .hb.clickable:hover {
    background: var(--c-hover) !important;
    border-radius: 2px;
  }

  .ac.clickable:hover {
    background: var(--c-hover);
    border-radius: 2px;
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
    color: var(--c-accent-t);
  }

  /* Non-printable dot */
  .ac.np {
    color: var(--c-border2);
  }

  /* ── Selection ── */
  .hb.sel,
  .ac.sel {
    background: var(--c-sel) !important;
    color: var(--c-sel-text) !important;
    border-radius: 2px;
  }

  /* Gap separator row */
  .gap-row {
    display: flex;
    align-items: center;
    padding: 0 12px;
    gap: 8px;
    flex-shrink: 0;
    background: var(--c-gap-bg);
  }
  .gap-line {
    flex: 1;
    border-top: 1px dashed var(--c-border);
  }
  .gap-label {
    color: var(--c-gap-text);
    font-size: 11px;
    white-space: nowrap;
    flex-shrink: 0;
    user-select: none;
  }
  /* Blank/gap byte cells (leading nulls at segment start) */
  .hb.blank {
    color: var(--c-null-text);
  }
  .ac.blank {
    color: var(--c-null-text);
  }

  /* ── Row-count footer ── */
  .row-count {
    padding: 3px 12px;
    font-size: 11px;
    color: var(--c-dim);
    border-top: 1px solid var(--c-border);
    flex-shrink: 0;
    user-select: none;
  }

  /* ── Empty state ── */
  .empty-state {
    color: var(--c-dim);
    text-align: center;
    margin-top: 4rem;
  }
</style>
