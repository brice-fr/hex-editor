<script>
  /**
   * HexViewer — displays parsed hex file data in a classic hex-dump layout.
   *
   * Props:
   *   records  - array of HexRecord objects from the Rust backend
   *   bytesPerRow - number of bytes to display per row (default 16)
   *
   * TODO: replace the naive full-render with a virtual-scrolling list
   *       (e.g. svelte-virtual-list) once the data model is stable.
   */

  /** @type {{ record_type: string; address: number; data: number[] }[]} */
  let { records = [], bytesPerRow = 16 } = $props();

  /**
   * Flatten all data records into a single address-keyed array of rows
   * suitable for display.
   */
  function buildRows(records, bytesPerRow) {
    const rows = [];
    for (const rec of records) {
      if (rec.record_type !== 'Data' || rec.data.length === 0) continue;
      for (let i = 0; i < rec.data.length; i += bytesPerRow) {
        const chunk = rec.data.slice(i, i + bytesPerRow);
        rows.push({
          address: rec.address + i,
          bytes: chunk,
          ascii: chunk
            .map((b) => (b >= 0x20 && b < 0x7f ? String.fromCharCode(b) : '.'))
            .join(''),
        });
      }
    }
    return rows;
  }

  const rows = $derived(buildRows(records, bytesPerRow));

  function hex8(n) {
    return n.toString(16).padStart(2, '0').toUpperCase();
  }
  function hex32(n) {
    return n.toString(16).padStart(8, '0').toUpperCase();
  }
</script>

<div class="hex-viewer">
  {#if rows.length === 0}
    <p class="empty-state">No data to display. Open an Intel HEX or S-record file.</p>
  {:else}
    <!-- Virtual scrolling stub: renders all rows for now -->
    <div class="hex-table-wrap">
      <table class="hex-table">
        <thead>
          <tr>
            <th class="col-addr">Address</th>
            <th class="col-hex">Hex</th>
            <th class="col-ascii">ASCII</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row}
            <tr>
              <td class="col-addr">{hex32(row.address)}</td>
              <td class="col-hex">
                {#each row.bytes as byte, i}
                  <span class="byte">{hex8(byte)}</span>{i === 7 ? ' ' : ''}
                {/each}
              </td>
              <td class="col-ascii">{row.ascii}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .hex-viewer {
    font-family: 'Courier New', Courier, monospace;
    font-size: 13px;
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .empty-state {
    color: #888;
    text-align: center;
    margin-top: 4rem;
  }

  .hex-table-wrap {
    overflow-y: auto;
    flex: 1;
  }

  .hex-table {
    border-collapse: collapse;
    width: 100%;
  }

  .hex-table th,
  .hex-table td {
    padding: 2px 8px;
    white-space: pre;
    text-align: left;
  }

  .hex-table thead th {
    position: sticky;
    top: 0;
    background: #1e1e1e;
    color: #aaa;
    border-bottom: 1px solid #444;
  }

  .col-addr {
    color: #569cd6;
    min-width: 90px;
  }

  .col-hex {
    color: #d4d4d4;
  }

  .col-ascii {
    color: #9cdcfe;
  }

  .byte {
    display: inline-block;
    width: 2ch;
    margin-right: 4px;
  }

  @media (prefers-color-scheme: light) {
    .hex-table thead th {
      background: #f0f0f0;
      color: #555;
      border-bottom: 1px solid #ccc;
    }
    .col-addr { color: #0070c1; }
    .col-hex  { color: #1e1e1e; }
    .col-ascii { color: #267f99; }
  }
</style>
