<script lang="ts">
  import { openNote } from '$lib/stores/editor';
  import { closeSearch, openSearch, pushToast } from '$lib/stores/ui';
  import { vaultPath } from '$lib/stores/vault';
  import { tauriInvoke } from '$lib/utils/tauri_bridge';
  import type { SearchResult } from '$lib/types';

  export let open = false;
  export let mode: 'keyword' | 'semantic' | 'smart' = 'keyword';

  let query = '';
  let results: SearchResult[] = [];
  let timer: ReturnType<typeof setTimeout> | null = null;
  let loading = false;

  const runSearch = async () => {
    if (!query.trim()) {
      results = [];
      return;
    }
    const vault = $vaultPath;
    if (!vault) return;
    loading = true;
    try {
      if (mode === 'semantic') {
        results = await tauriInvoke<SearchResult[]>('search_semantic', { vaultPath: vault, query });
      } else if (mode === 'smart') {
        const keyword = await tauriInvoke<SearchResult[]>('search_notes', { vaultPath: vault, query });
        const semantic = await tauriInvoke<SearchResult[]>('search_semantic', { vaultPath: vault, query });
        results = mergeResults(keyword, semantic);
      } else {
        results = await tauriInvoke<SearchResult[]>('search_notes', { vaultPath: vault, query });
      }
    } catch (err) {
      pushToast(err instanceof Error ? err.message : String(err));
    } finally {
      loading = false;
    }
  };

  const onInput = () => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(runSearch, 150);
  };

  const changeMode = (next: 'keyword' | 'semantic' | 'smart') => {
    openSearch(next);
    setTimeout(runSearch, 0);
  };

  const openResult = (result: SearchResult) => {
    if (result.path) openNote(result.path);
    closeSearch();
  };

  const mergeResults = (keyword: SearchResult[], semantic: SearchResult[]) => {
    const merged = new Map<string, SearchResult>();
    const add = (result: SearchResult) => {
      const key = result.path || result.title;
      const existing = merged.get(key);
      if (!existing || result.score > existing.score) {
        merged.set(key, result);
      }
    };
    semantic.forEach(add);
    keyword.forEach(add);
    return Array.from(merged.values()).sort((a, b) => b.score - a.score);
  };
</script>

{#if open}
  <div class="backdrop" on:click={closeSearch}>
    <div class="modal" on:click|stopPropagation>
      <div class="tabs">
        <button class={`tab ${mode === 'keyword' ? 'active' : ''}`} on:click={() => changeMode('keyword')}>
          Keyword
        </button>
        <button class={`tab ${mode === 'semantic' ? 'active' : ''}`} on:click={() => changeMode('semantic')}>
          Semantic
        </button>
        <button class={`tab ${mode === 'smart' ? 'active' : ''}`} on:click={() => changeMode('smart')}>
          Smart
        </button>
      </div>
      <input placeholder="Search notes" bind:value={query} on:input={onInput} />
      {#if loading}
        <div class="loading">Searching...</div>
      {:else}
        <div class="results">
          {#each results as result (result.path + result.title)}
            <button on:click={() => openResult(result)}>
              <div class="title">{result.title}</div>
              <div class="snippet">{@html result.snippet}</div>
              <div class="meta">{result.result_type} - {result.score.toFixed(2)}</div>
            </button>
          {/each}
          {#if results.length === 0}
            <div class="empty">No results.</div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: var(--overlay);
    display: grid;
    place-items: center;
    z-index: 40;
  }
  .modal {
    width: 600px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-strong);
    padding: var(--space-3);
  }
  .tabs {
    display: flex;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }
  .tab {
    color: var(--text-muted);
    background: transparent;
    border: 1px solid transparent;
    padding: 0;
  }
  .tab.active {
    color: var(--accent);
    border-color: var(--accent);
    padding: 2px 8px;
    border-radius: var(--radius-sm);
  }
  input {
    width: 100%;
    margin-bottom: var(--space-2);
  }
  .results {
    display: grid;
    gap: var(--space-2);
    max-height: 360px;
    overflow: auto;
  }
  .results button {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    text-align: left;
    padding: var(--space-2);
  }
  .title {
    font-weight: 600;
  }
  .snippet {
    color: var(--text-secondary);
    font-size: 12px;
  }
  .meta {
    font-size: 11px;
    color: var(--text-muted);
  }
  .empty {
    color: var(--text-muted);
    padding: var(--space-2);
  }
</style>
