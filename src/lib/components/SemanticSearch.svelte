<script lang="ts">
  import { appState, openFile } from '$lib/store.svelte';
  import { semanticSearch, fileRead } from '$lib/api';
  import type { SemanticResult } from '$lib/types';
  import { Command } from 'bits-ui';
  import { Dialog } from '$lib/components/ui';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();
  let open = $state(true);
  let query = $state('');
  let results = $state<SemanticResult[]>([]);
  let isSearching = $state(false);
  let searchError = $state<string | null>(null);
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if (!open) {
      onClose();
    }
  });

  async function doSearch() {
    if (!appState.system || query.trim().length < 2) {
      results = [];
      searchError = null;
      isSearching = false;
      return;
    }
    isSearching = true;
    searchError = null;
    try {
      results = await semanticSearch(appState.system.path, query.trim());
    } catch (e) {
      searchError = String(e);
      results = [];
    } finally {
      isSearching = false;
    }
  }

  $effect(() => {
    const q = query;
    if (searchTimeout) clearTimeout(searchTimeout);
    if (q.trim().length < 2) {
      results = [];
      searchError = null;
      isSearching = false;
      return;
    }
    searchTimeout = setTimeout(() => {
      doSearch();
    }, 300);
  });

  async function handleSelect(result: SemanticResult) {
    const content = await fileRead(result.path);
    openFile(result.path, content);
    open = false;
  }

  function truncate(text: string, max: number) {
    if (text.length <= max) return text;
    return text.slice(0, max).trimEnd() + '…';
  }

  function formatScore(score: number) {
    return score.toFixed(3);
  }
</script>

<Dialog bind:open position="top" contentClass="search-dialog" showClose={false}>
  <Command.Root shouldFilter={false} class="cmd-root">
    <Command.Input
      class="cmd-input"
      bind:value={query}
      placeholder="Semantic search..."
      autofocus
    />
    <Command.List class="cmd-list">
      {#if isSearching}
        <Command.Empty class="cmd-empty">Searching…</Command.Empty>
      {:else if searchError}
        <Command.Empty class="cmd-empty error">{searchError}</Command.Empty>
      {:else if query.trim().length < 2}
        <Command.Empty class="cmd-empty">Type at least 2 characters to search</Command.Empty>
      {:else if results.length === 0}
        <Command.Empty class="cmd-empty">No results</Command.Empty>
      {:else}
        {#each results as result}
          <Command.Item
            class="cmd-item"
            value={result.path}
            onSelect={() => handleSelect(result)}
          >
            <div class="result-top">
              <span class="result-name">{result.name}</span>
              <span class="result-score">{formatScore(result.score)}</span>
            </div>
            <div class="result-snippet">{truncate(result.content, 140)}</div>
          </Command.Item>
        {/each}
      {/if}
    </Command.List>
  </Command.Root>
</Dialog>

<style>
  :global(.search-dialog) {
    width: 600px;
  }
  .result-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .result-name {
    color: var(--text-bright);
    font-size: 13px;
  }
  .result-score {
    color: var(--muted-2);
    font-size: 11px;
    font-family: var(--font);
  }
  .result-snippet {
    color: var(--muted-3);
    font-size: 11px;
    line-height: 1.4;
  }
  :global(.cmd-empty.error) {
    color: var(--error);
  }
</style>
