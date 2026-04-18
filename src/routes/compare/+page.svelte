<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: 2026 Brice LECOLE -->

<script>
  import { onMount } from 'svelte';
  import { parseFile } from '$lib/api.js';
  import DiffViewer from '$lib/components/DiffViewer.svelte';

  let refPath  = $state('');
  let cmpPath  = $state('');
  let refRecords = $state([]);
  let cmpRecords = $state([]);
  let loading  = $state(true);
  let error    = $state('');

  onMount(async () => {
    const params = new URLSearchParams(window.location.search);
    refPath = params.get('ref') ?? '';
    cmpPath = params.get('cmp') ?? '';

    if (!refPath || !cmpPath) {
      error = 'Missing file paths in URL parameters.';
      loading = false;
      return;
    }

    try {
      [refRecords, cmpRecords] = await Promise.all([parseFile(refPath), parseFile(cmpPath)]);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });
</script>

{#if loading}
  <div class="state-screen">
    <svg class="spin" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"
         stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
    </svg>
    <span>Loading files…</span>
  </div>
{:else if error}
  <div class="state-screen err">{error}</div>
{:else}
  <DiffViewer {refPath} {cmpPath} {refRecords} {cmpRecords} />
{/if}

<style>
  .state-screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    gap: 14px;
    color: var(--c-muted);
    font-size: 13px;
  }
  .state-screen.err { color: var(--c-err); }
  .spin {
    width: 28px;
    height: 28px;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
