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
      inputVal = node.name;
      onRenameCommit(node.name);
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
    padding: 10px 12px;
    border: 1px solid transparent;
    border-radius: 14px;
    cursor: pointer;
    transition:
      border-color var(--transition),
      background var(--transition),
      transform var(--transition);
  }
  .row:hover {
    background: var(--bg-surface);
    border-color: var(--border-subtle);
    transform: translateX(2px);
  }
  input {
    background: var(--bg-elevated);
    border: 1px solid var(--accent);
    color: var(--text-primary);
    font-family: inherit;
    font-size: inherit;
    padding: 6px 8px;
    width: 100%;
    border-radius: var(--radius-xs);
  }
  .row.active {
    border-color: var(--accent-glow);
    background: linear-gradient(135deg, var(--accent-subtle), var(--accent2-subtle));
  }
  .children {
    margin-left: var(--space-4);
    padding-left: var(--space-2);
    border-left: 1px solid var(--border-subtle);
  }
  .folder-icon {
    color: var(--accent2);
  }
  .file-icon {
    color: var(--text-secondary);
  }
</style>
