<script lang="ts">
  import TreeNode from '$lib/components/TreeNode.svelte';
  import type { FileNode } from '$lib/types';
  import {
    fileTree,
    vaultPath,
    createNote,
    createFolder,
    renameItem,
    deleteItem,
    setFocused,
    focusedPath,
    focusedIsFolder,
    toVaultRelative
  } from '$lib/stores/vault';
  import { openNote } from '$lib/stores/editor';

  let expanded = new Set<string>();
  let contextOpen = false;
  let contextX = 0;
  let contextY = 0;
  let contextNode: FileNode | null = null;

  const toggleFolder = (path: string) => {
    const next = new Set(expanded);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    expanded = next;
  };

  const onContextMenu = (event: MouseEvent, node: FileNode) => {
    event.preventDefault();
    contextOpen = true;
    contextX = event.clientX;
    contextY = event.clientY;
    contextNode = node;
    setFocused(node.path, node.is_folder);
  };

  const closeContext = () => {
    contextOpen = false;
    contextNode = null;
  };

  const handlePanelKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Escape' || event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      closeContext();
    }
  };

  const parentFolder = (path: string) => {
    const parts = path.split(/[/\\]/);
    parts.pop();
    return parts.join('/');
  };

  const handleCreateNote = async () => {
    const title = window.prompt('Note title');
    if (!title) return closeContext();
    const vault = $vaultPath;
    if (!vault) return closeContext();
    const target = contextNode?.path ?? vault;
    const base = contextNode?.is_folder ? target : parentFolder(target);
    const folder = toVaultRelative(base, vault);
    const path = await createNote(folder, title);
    if (path) openNote(path);
    closeContext();
  };

  const handleCreateFolder = async () => {
    const name = window.prompt('Folder name');
    if (!name) return closeContext();
    const vault = $vaultPath;
    if (!vault) return closeContext();
    const target = contextNode?.path ?? vault;
    const base = contextNode?.is_folder ? target : parentFolder(target);
    const relative = toVaultRelative(base, vault);
    const folderPath = relative ? `${relative}/${name}` : name;
    await createFolder(folderPath);
    closeContext();
  };

  const handleRename = async () => {
    if (!contextNode) return closeContext();
    const name = window.prompt('New name', contextNode.name);
    if (!name) return closeContext();
    await renameItem(contextNode.path, name);
    closeContext();
  };

  const handleDelete = async () => {
    if (!contextNode) return closeContext();
    const confirmed = window.confirm(`Delete ${contextNode.name}?`);
    if (!confirmed) return closeContext();
    await deleteItem(contextNode.path);
    closeContext();
  };
</script>

<div
  class="panel"
  role="button"
  tabindex="0"
  on:click={closeContext}
  on:keydown={handlePanelKeydown}
>
  <div class="header">
    <h3>Vault</h3>
    {#if $focusedPath}
      <span class="focus">{($focusedIsFolder ? 'Folder' : 'File') + ' selected'}</span>
    {/if}
  </div>
  <div class="tree">
    {#each $fileTree as node (node.path)}
      <TreeNode {node} {expanded} {toggleFolder} onContext={onContextMenu} />
    {/each}
  </div>
</div>

{#if contextOpen}
  <div class="context" style={`top:${contextY}px; left:${contextX}px;`}>
    <button on:click={handleCreateNote}>New Note</button>
    <button on:click={handleCreateFolder}>New Folder</button>
    <button on:click={handleRename}>Rename</button>
    <button class="danger" on:click={handleDelete}>Delete</button>
  </div>
{/if}

<style>
  .panel {
    padding: var(--space-3);
  }
  .header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: var(--space-2);
  }
  h3 {
    margin: 0;
    color: var(--text-secondary);
  }
  .focus {
    font-size: 11px;
    color: var(--text-muted);
  }
  .context {
    position: fixed;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--space-2);
    display: grid;
    gap: var(--space-1);
    z-index: 50;
  }
  .context button {
    background: transparent;
    border: 1px solid var(--border-subtle);
    padding: 6px 8px;
    text-align: left;
  }
  .context .danger {
    border-color: var(--danger);
    color: var(--danger);
  }
</style>
