<script lang="ts">
  import { X } from "phosphor-svelte";
  import { openTabs, activePath, openNote, closeTab } from "$lib/stores/editor";
</script>

<div class="tabbar">
  {#each $openTabs as tab (tab.path)}
    <div
      class={`tab ${$activePath === tab.path ? "active" : ""}`}
      role="button"
      tabindex="0"
      aria-pressed={$activePath === tab.path}
      on:click={() => openNote(tab.path)}
      on:keydown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          openNote(tab.path);
        }
      }}
    >
      <span class="title">{tab.title}</span>
      {#if tab.dirty}
        <span class="dot"></span>
      {/if}
      <button
        type="button"
        class="close"
        aria-label={`Close ${tab.title}`}
        on:click|stopPropagation={() => closeTab(tab.path)}
      >
        <X size={12} />
      </button>
    </div>
  {/each}
</div>

<style>
  .tabbar {
    display: flex;
    gap: var(--space-1);
    padding: var(--space-2);
    border-bottom: 1px solid var(--border);
    background: var(--bg-panel);
    overflow-x: auto;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 4px 10px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  .tab.active {
    border-color: var(--accent);
  }
  .title {
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
  }
  .close {
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    border-radius: var(--radius-sm);
  }
</style>
