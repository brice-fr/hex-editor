<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->
<script>
  let { children } = $props();
</script>

{@render children()}

<style>
  /* ── Colour tokens (dark mode defaults) ──────────────────────────── */
  :global(:root) {
    --c-bg:        #1e1e1e;
    --c-surface:   #252526;
    --c-raised:    #2d2d2d;
    --c-hover:     #3c3c3c;
    --c-border:    #3a3a3a;
    --c-border2:   #454545;
    --c-text:      #e0e0e0;
    --c-text2:     #ccc;
    --c-muted:     #888;
    --c-dim:       #555;
    --c-accent:    #007acc;
    --c-accent-t:  #9cdcfe;
    --c-accent-b:  #0e639c;
    --c-accent-h:  #1177bb;
    --c-accent-bg: rgba(0,122,204,0.15);
    --c-err:       #f47171;
    --c-err-bg:    rgba(244,71,71,0.15);
    --c-gap-bg:    #1a1a1a;
    --c-gap-text:  #555;
    --c-addr:      #569cd6;
    --c-null:      #3a3a3a;
    --c-null-text: #3a3a3a;
    --c-ec0:       transparent;
    --c-ec1:       rgba(255,255,255,0.03);
    --c-header-bg: #1e1e1e;
    --c-sel:       rgba(58,110,165,0.60);
    --c-sel-text:  #ffffff;
    --c-scrollbar-thumb: #000000;
    --c-scrollbar-track: #2d2d2d;
    /* diff viewer colours */
    --c-diff-changed:        #ffcb3d;
    --c-diff-changed-bg:     rgba(255,203,61,0.12);
    --c-diff-ref-only:       #6bc5f8;
    --c-diff-ref-only-bg:    rgba(107,197,248,0.12);
    --c-diff-cmp-only:       #6cde8d;
    --c-diff-cmp-only-bg:    rgba(108,222,141,0.12);
  }

  @media (prefers-color-scheme: light) {
    :global(:root:not([data-theme="dark"])) {
      --c-bg:        #ffffff;
      --c-surface:   #f8f8f8;
      --c-raised:    #f0f0f0;
      --c-hover:     #e8e8e8;
      --c-border:    #e0e0e0;
      --c-border2:   #d4d4d4;
      --c-text:      #1e1e1e;
      --c-text2:     #444;
      --c-muted:     #666;
      --c-dim:       #999;
      --c-accent:    #0070c1;
      --c-accent-t:  #0070c1;
      --c-accent-b:  #0e639c;
      --c-accent-h:  #0080d4;
      --c-accent-bg: rgba(0,112,193,0.10);
      --c-err:       #d32f2f;
      --c-err-bg:    rgba(211,47,47,0.10);
      --c-gap-bg:    #f0f0f0;
      --c-gap-text:  #aaa;
      --c-addr:      #0451a5;
      --c-null:      #e8e8e8;
      --c-null-text: #e8e8e8;
      --c-ec0:       transparent;
      --c-ec1:       rgba(0,0,0,0.03);
      --c-header-bg: #f3f3f3;
      --c-sel:       rgba(0,95,204,0.22);
      --c-sel-text:  #1a1a1a;
      --c-scrollbar-thumb: #777777;
      --c-scrollbar-track: #d4d4d4;
      /* diff viewer colours */
      --c-diff-changed:        #9a6200;
      --c-diff-changed-bg:     rgba(154,98,0,0.10);
      --c-diff-ref-only:       #0451a5;
      --c-diff-ref-only-bg:    rgba(4,81,165,0.10);
      --c-diff-cmp-only:       #057a43;
      --c-diff-cmp-only-bg:    rgba(5,122,67,0.10);
    }
  }

  :global(:root[data-theme="light"]) {
    --c-bg:        #ffffff;
    --c-surface:   #f8f8f8;
    --c-raised:    #f0f0f0;
    --c-hover:     #e8e8e8;
    --c-border:    #e0e0e0;
    --c-border2:   #d4d4d4;
    --c-text:      #1e1e1e;
    --c-text2:     #444;
    --c-muted:     #666;
    --c-dim:       #999;
    --c-accent:    #0070c1;
    --c-accent-t:  #0070c1;
    --c-accent-b:  #0e639c;
    --c-accent-h:  #0080d4;
    --c-accent-bg: rgba(0,112,193,0.10);
    --c-err:       #d32f2f;
    --c-err-bg:    rgba(211,47,47,0.10);
    --c-gap-bg:    #f0f0f0;
    --c-gap-text:  #aaa;
    --c-addr:      #0451a5;
    --c-null:      #e8e8e8;
    --c-null-text: #e8e8e8;
    --c-ec0:       transparent;
    --c-ec1:       rgba(0,0,0,0.03);
    --c-header-bg: #f3f3f3;
    --c-sel:       rgba(0,95,204,0.22);
    --c-sel-text:  #1a1a1a;
    --c-scrollbar-thumb: #777777;
    --c-scrollbar-track: #d4d4d4;
    /* diff viewer colours */
    --c-diff-changed:    #9a6200;
    --c-diff-changed-bg: rgba(154,98,0,0.10);
    --c-diff-ref-only:   #0451a5;
    --c-diff-cmp-only:   #057a43;
  }

  /* ── Global resets ──────────────────────────────────────────────── */
  :global(*, *::before, *::after) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    background: var(--c-bg);
    color: var(--c-text);
    overflow: hidden;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI',
                 'Helvetica Neue', Arial, sans-serif;
    font-feature-settings: 'cv02', 'cv03', 'cv04', 'tnum';
    -webkit-font-smoothing: antialiased;
  }
</style>
