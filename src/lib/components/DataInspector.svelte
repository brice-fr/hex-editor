<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  /** @type {{ record_type: string; address: number; data: number[] }[]} */
  let { records = [], address = 0, pinned = false, onClose = () => {}, onUnpin = () => {} } = $props();

  function isDataRecord(rec) {
    return (rec.record_type === 'Data' || rec.record_type === 'S1'
         || rec.record_type === 'S2'  || rec.record_type === 'S3')
        && rec.data.length > 0;
  }

  /** Read up to `count` bytes from `addr`; entries are null when unmapped */
  function getBytesAt(recs, addr, count) {
    const out = new Array(count).fill(null);
    for (const rec of recs) {
      if (!isDataRecord(rec)) continue;
      const rs = rec.address, re = rec.address + rec.data.length;
      if (re <= addr || rs >= addr + count) continue;
      for (let i = 0; i < count; i++) {
        const abs = addr + i;
        if (abs >= rs && abs < re) out[i] = rec.data[abs - rs];
      }
    }
    return out;
  }

  /** Build a DataView from the first `n` entries; returns null if any are null */
  function makeView(b, n) {
    const s = b.slice(0, n);
    if (s.some(x => x === null)) return null;
    const buf = new ArrayBuffer(n);
    new Uint8Array(buf).set(s);
    return new DataView(buf);
  }

  /** Format a float — strip trailing noise but keep precision */
  function fmtF(v) {
    if (!isFinite(v)) return v.toString();
    return parseFloat(v.toPrecision(10)).toString();
  }

  /** Decode bytes (number|null)[] as UTF-8 up to the first null byte or unmapped slot */
  function decodeUtf8String(b) {
    const stop = b.findIndex(x => x === null || x === 0);
    const slice = (stop >= 0 ? b.slice(0, stop) : b).filter(x => x !== null);
    if (slice.length === 0) return '(empty)';
    return new TextDecoder('utf-8', { fatal: false }).decode(new Uint8Array(slice));
  }

  /** Decode bytes as UTF-16 LE, stopping at the first 0x00 0x00 pair or unmapped slot */
  function decodeUtf16String(b) {
    const buf = [];
    for (let i = 0; i + 1 < b.length; i += 2) {
      const lo = b[i], hi = b[i + 1];
      if (lo === null || hi === null) break;
      if (lo === 0 && hi === 0) break;
      buf.push(lo, hi);
    }
    if (buf.length === 0) return '(empty)';
    return new TextDecoder('utf-16le', { fatal: false }).decode(new Uint8Array(buf));
  }

  const bytes = $derived(getBytesAt(records, address, 8));

  const hexBytes = $derived(
    bytes.map(b => b !== null ? b.toString(16).padStart(2, '0').toUpperCase() : '--').join(' ')
  );

  const rows = $derived((() => {
    const b = bytes;
    const v1 = makeView(b, 1);
    const v2 = makeView(b, 2);
    const v4 = makeView(b, 4);
    const v8 = makeView(b, 8);
    const na = '—';

    const hasNullByte = b.some(x => x === 0);
    const hasTwoContiguousZeros = b.some((x, i) => x === 0 && i + 1 < b.length && b[i + 1] === 0);

    const result = [];
    if (hasNullByte) {
      result.push({ group: 'String' });
      result.push({ label: 'UTF-8',    value: decodeUtf8String(b),  str: true });
      if (hasTwoContiguousZeros) {
        result.push({ label: 'UTF-16 LE', value: decodeUtf16String(b), str: true });
      }
    }
    result.push(
      { group: '8-bit' },
      { label: 'Uint8',     value: v1 ? v1.getUint8(0).toString()              : na },
      { label: 'Int8',      value: v1 ? v1.getInt8(0).toString()               : na },
      { group: '16-bit' },
      { label: 'Uint16 LE', value: v2 ? v2.getUint16(0, true).toString()       : na },
      { label: 'Uint16 BE', value: v2 ? v2.getUint16(0, false).toString()      : na },
      { label: 'Int16 LE',  value: v2 ? v2.getInt16(0, true).toString()        : na },
      { label: 'Int16 BE',  value: v2 ? v2.getInt16(0, false).toString()       : na },
      { group: '32-bit' },
      { label: 'Uint32 LE', value: v4 ? v4.getUint32(0, true).toString()       : na },
      { label: 'Uint32 BE', value: v4 ? v4.getUint32(0, false).toString()      : na },
      { label: 'Int32 LE',  value: v4 ? v4.getInt32(0, true).toString()        : na },
      { label: 'Int32 BE',  value: v4 ? v4.getInt32(0, false).toString()       : na },
      { label: 'Float32 LE',value: v4 ? fmtF(v4.getFloat32(0, true))           : na },
      { label: 'Float32 BE',value: v4 ? fmtF(v4.getFloat32(0, false))          : na },
      { group: '64-bit' },
      { label: 'Uint64 LE', value: v8 ? v8.getBigUint64(0, true).toString()    : na },
      { label: 'Uint64 BE', value: v8 ? v8.getBigUint64(0, false).toString()   : na },
      { label: 'Int64 LE',  value: v8 ? v8.getBigInt64(0, true).toString()     : na },
      { label: 'Int64 BE',  value: v8 ? v8.getBigInt64(0, false).toString()    : na },
      { label: 'Float64 LE',value: v8 ? fmtF(v8.getFloat64(0, true))           : na },
      { label: 'Float64 BE',value: v8 ? fmtF(v8.getFloat64(0, false))          : na },
    );
    return result;
  })());

  function hex32(n) { return '0x' + n.toString(16).padStart(8, '0').toUpperCase(); }
</script>

<div class="inspector-panel">
  <div class="panel-header">
    <span class="header-title">Data Inspector</span>
    {#if pinned}
      <button class="addr-chip pinned" onclick={onUnpin} title="Pinned — click to follow scroll again">
        {hex32(address)}
      </button>
    {:else}
      <span class="addr-chip">{hex32(address)}</span>
    {/if}
    <button class="close-btn" onclick={onClose} aria-label="Close panel" title="Close">×</button>
  </div>

  <div class="bytes-strip">
    <span class="bytes-label">Bytes</span>
    <span class="bytes-val">{hexBytes}</span>
  </div>

  <div class="rows-wrap">
    {#each rows as row}
      {#if row.group}
        <div class="group-hdr">{row.group}</div>
      {:else}
        <div class="insp-row">
          <span class="insp-label">{row.label}</span>
          <span class="insp-value" class:na={row.value === '—'} class:str={row.str}>{row.value}</span>
        </div>
      {/if}
    {/each}
  </div>
</div>

<style>
  .inspector-panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Courier New', monospace;
    font-size: 12px;
    background: var(--c-bg);
    color: var(--c-text);
    overflow: hidden;
  }

  /* ── Header ── */
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
    color: var(--c-muted);
    background: var(--c-surface);
    border-bottom: 1px solid var(--c-hover);
    flex-shrink: 0;
    user-select: none;
  }

  .header-title {
    flex: 1;
  }

  .addr-chip {
    font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', monospace;
    font-size: 10.5px;
    color: var(--c-addr);
    font-weight: 400;
    letter-spacing: 0;
    text-transform: none;
    background: none;
    border: none;
    padding: 0;
    cursor: default;
  }

  /* When pinned: chip becomes a clickable button to unpin */
  .addr-chip.pinned {
    cursor: pointer;
    color: #ce9178;
    border-bottom: 1px dashed #ce9178;
  }

  .addr-chip.pinned:hover {
    color: #f4b89a;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--c-dim);
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
    color: var(--c-text2);
    background: var(--c-hover);
  }

  /* ── Raw byte strip ── */
  .bytes-strip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    border-bottom: 1px solid var(--c-border);
    flex-shrink: 0;
  }

  .bytes-label {
    color: var(--c-dim);
    font-family: 'Inter', sans-serif;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    flex-shrink: 0;
  }

  .bytes-val {
    color: var(--c-accent-t);
    font-size: 11px;
    letter-spacing: 0.04em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* ── Scrollable interpretation area ── */
  .rows-wrap {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 2px 0;
  }

  .group-hdr {
    padding: 5px 10px 2px;
    font-family: 'Inter', -apple-system, sans-serif;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--c-border2);
    user-select: none;
  }

  .insp-row {
    display: flex;
    align-items: baseline;
    padding: 2px 10px;
    gap: 6px;
  }

  .insp-row:hover {
    background: var(--c-ec1);
  }

  .insp-label {
    flex-shrink: 0;
    width: 80px;
    color: var(--c-dim);
    font-size: 11px;
    text-align: right;
  }

  .insp-value {
    flex: 1;
    color: var(--c-text);
    font-size: 11.5px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .insp-value.na {
    color: var(--c-null-text);
  }

  .insp-value.str {
    color: #ce9178; /* string literal colour — warm orange, same as VS Code */
    font-style: italic;
  }
</style>
