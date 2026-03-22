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
    padding: var(--space-3) var(--space-4) var(--space-2);
    border-bottom: 1px solid var(--border-subtle);
    background: rgba(11, 17, 29, 0.52);
    overflow-x: auto;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 8px 12px;
    background: rgba(16, 24, 39, 0.74);
    border: 1px solid var(--border-subtle);
    border-radius: 14px;
    cursor: pointer;
    transition:
      border-color var(--transition),
      background var(--transition),
      transform var(--transition);
  }
  .tab:hover {
    transform: translateY(-1px);
    border-color: var(--border);
  }
  .tab.active {
    border-color: rgba(88, 193, 255, 0.22);
    background: linear-gradient(
      135deg,
      rgba(88, 193, 255, 0.14),
      rgba(126, 240, 197, 0.08)
    );
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
    background: var(--accent-bright);
  }
  .close {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 10px;
    background: transparent;
    border-color: transparent;
  }
</style>
