<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  /** @type {{ record_type: string; address: number; data: number[] }[]} */
  let { records = [], topAddress = 0, onJump = (_addr) => {}, onClose = () => {} } = $props();

  function isDataRecord(rec) {
    return (rec.record_type === 'Data' || rec.record_type === 'S1'
         || rec.record_type === 'S2'  || rec.record_type === 'S3')
        && rec.data.length > 0;
  }

  /** Merge contiguous records into address-sorted segments */
  const segments = $derived((() => {
    const sorted = records.filter(isDataRecord).sort((a, b) => a.address - b.address);
    const segs = [];
    for (const rec of sorted) {
      if (segs.length === 0) {
        segs.push({ address: rec.address, size: rec.data.length });
      } else {
        const last = segs[segs.length - 1];
        if (rec.address === last.address + last.size) {
          last.size += rec.data.length;   // contiguous — extend
        } else {
          segs.push({ address: rec.address, size: rec.data.length });
        }
      }
    }
    return segs;
  })());

  /**
   * Segment that contains the current top-visible address — auto-tracks scroll.
   * Falls back to -1 when address is in a gap.
   */
  const activeIdx = $derived(
    segments.findIndex(s => topAddress >= s.address && topAddress < s.address + s.size)
  );

  function handleClick(idx) {
    onJump(segments[idx].address);
  }

  function hex32(n) { return '0x' + n.toString(16).padStart(8, '0').toUpperCase(); }

  function fmtSize(n) {
    if (n >= 1024 * 1024) return (n / (1024 * 1024)).toFixed(2) + ' MiB';
    if (n >= 1024)        return (n / 1024).toFixed(2) + ' KiB';
    return n.toLocaleString() + ' B';
  }
</script>

<div class="segment-panel">
  <div class="panel-header">
    <span class="header-title">
      Segments
      {#if segments.length > 0}
        <span class="badge">{segments.length}</span>
      {/if}
    </span>
    <button class="close-btn" onclick={onClose} aria-label="Close panel" title="Close">×</button>
  </div>

  {#if segments.length === 0}
    <p class="empty">No data loaded</p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th class="th-num">#</th>
            <th>Start</th>
            <th>End</th>
            <th class="th-size">Size</th>
          </tr>
        </thead>
        <tbody>
          {#each segments as seg, i}
            <!-- svelte-ignore a11y_interactive_supports_focus -->
            <tr
              class:active={i === activeIdx}
              onclick={() => handleClick(i)}
              role="button"
              tabindex="0"
              onkeydown={(e) => e.key === 'Enter' && handleClick(i)}
            >
              <td class="td-num">{i + 1}</td>
              <td class="td-addr">{hex32(seg.address)}</td>
              <td class="td-addr">{hex32(seg.address + seg.size - 1)}</td>
              <td class="td-size">{fmtSize(seg.size)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .segment-panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Courier New', monospace;
    font-size: 12px;
    background: #1e1e1e;
    color: #d4d4d4;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 6px 5px 10px;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: #888;
    background: #252526;
    border-bottom: 1px solid #3c3c3c;
    flex-shrink: 0;
    user-select: none;
  }

  .header-title {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .badge {
    background: #3c3c3c;
    color: #aaa;
    border-radius: 10px;
    padding: 0 6px;
    font-size: 10px;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: #555;
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    padding: 2px 5px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .close-btn:hover {
    color: #ccc;
    background: rgba(255, 255, 255, 0.08);
  }

  .empty {
    color: #555;
    text-align: center;
    margin-top: 2rem;
    font-family: 'Inter', sans-serif;
    font-size: 12px;
  }

  .table-wrap {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  thead th {
    position: sticky;
    top: 0;
    background: #252526;
    color: #666;
    font-weight: 400;
    padding: 3px 8px;
    text-align: left;
    border-bottom: 1px solid #3c3c3c;
    font-size: 11px;
    user-select: none;
  }

  .th-num  { width: 24px; text-align: center; }
  .th-size { text-align: right; padding-right: 10px; }

  tbody tr {
    cursor: pointer;
  }

  tbody tr:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  tbody tr.active {
    background: rgba(0, 122, 204, 0.16);
  }

  tbody tr.active td { color: #4fc3f7; }
  tbody tr.active .td-addr { color: #4fc3f7; }

  td {
    padding: 3px 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    white-space: nowrap;
  }

  .td-num  { color: #555; text-align: center; width: 24px; }
  .td-addr { color: #569cd6; font-size: 11.5px; }
  .td-size { color: #9cdcfe; text-align: right; padding-right: 10px; }

  /* Light mode */
  @media (prefers-color-scheme: light) {
    .segment-panel            { background: #fff; color: #1e1e1e; }
    .panel-header             { background: #f5f5f5; border-bottom-color: #d0d0d0; color: #888; }
    .close-btn                { color: #bbb; }
    .close-btn:hover          { color: #666; background: rgba(0,0,0,0.06); }
    .badge                    { background: #e0e0e0; color: #888; }
    thead th                  { background: #f5f5f5; border-bottom-color: #d0d0d0; }
    tbody tr:hover            { background: rgba(0, 0, 0, 0.03); }
    tbody tr.active           { background: rgba(0, 122, 204, 0.08); }
    tbody tr.active td        { color: #0070c1; }
    tbody tr.active .td-addr  { color: #0070c1; }
    td                        { border-bottom-color: rgba(0, 0, 0, 0.05); }
    .td-num                   { color: #bbb; }
    .td-addr                  { color: #0070c1; }
    .td-size                  { color: #267f99; }
  }
</style>
