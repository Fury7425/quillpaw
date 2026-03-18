<script lang="ts">
  import { FolderSimple, FileText } from "phosphor-svelte";
  import type { FileNode } from "$lib/types";
  import { setFocused, focusedPath } from "$lib/stores/vault";
  import { openNote } from "$lib/stores/editor";

  export let node: FileNode;
  export let expanded: Set<string>;
  export let toggleFolder: (path: string) => void;
  export let onContext: (event: MouseEvent, node: FileNode) => void;
  export let editingPath: string | null = null;
  export let onRenameCommit: (newName: string) => void;

  $: editing = node.path === editingPath;
  let inputVal = node.name;

  const focus = (el: HTMLInputElement) => el.focus();

  const handleRenameKey = (e: KeyboardEvent) => {
    if (e.key === "Enter") {
      onRenameCommit(inputVal.trim());
    } else if (e.key === "Escape") {
      editing = false;
    }
  };

  const activate = () => {
    setFocused(node.path, node.is_folder);
    if (!node.is_folder) openNote(node.path);
    if (node.is_folder) toggleFolder(node.path);
  };

  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      activate();
    }
  };
</script>

<div class="node">
  <div
    class={`row ${node.path === $focusedPath ? "active" : ""}`}
    role="button"
    tabindex="0"
    on:click={activate}
    on:keydown={handleKeydown}
    on:contextmenu={(event) => onContext(event, node)}
  >
    {#if node.is_folder}
      <FolderSimple size={16} class="folder-icon" />
    {:else}
      <FileText size={16} class="file-icon" />
    {/if}
    {#if editing}
      <input
        type="text"
        bind:value={inputVal}
        use:focus
        on:blur={() => onRenameCommit(inputVal.trim())}
        on:keydown={handleRenameKey}
        on:click|stopPropagation
      />
    {:else}
      <span>{node.name}</span>
    {/if}
  </div>
  {#if node.is_folder && expanded.has(node.path) && node.children}
    <div class="children">
      {#each node.children as child (child.path)}
        <svelte:self
          node={child}
          {expanded}
          {toggleFolder}
          {onContext}
          {onRenameCommit}
          {editingPath}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2);
    border-left: 3px solid transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  input {
    background: var(--bg-surface);
    border: 1px solid var(--accent);
    color: var(--text-primary);
    font-family: inherit;
    font-size: inherit;
    padding: 0 4px;
    width: 100%;
    border-radius: var(--radius-xs);
  }
  .row.active {
    border-left-color: var(--accent);
    background: var(--accent-subtle);
  }
  .children {
    margin-left: var(--space-3);
  }
  .folder-icon {
    color: var(--accent2);
  }
  .file-icon {
    color: var(--text-secondary);
  }
</style>
