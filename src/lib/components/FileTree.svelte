<script lang="ts">
  import { onDestroy } from "svelte";
  import TreeNode from "$lib/components/TreeNode.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import type { FileNode } from "$lib/types";
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
    toVaultRelative,
    renameRequest,
  } from "$lib/stores/vault";
  import { openNote } from "$lib/stores/editor";
  import { pushToast } from "$lib/stores/ui";

  let expanded = new Set<string>();
  let contextOpen = false;
  let contextX = 0;
  let contextY = 0;
  let contextNode: FileNode | null = null;
  let deleteDialogOpen = false;
  let deleteMessage = "";
  let editingPath: string | null = null;

  const formatError = (err: unknown) =>
    err instanceof Error ? err.message : String(err);

  const unsubscribeRename = renameRequest.subscribe((path) => {
    if (path) {
      editingPath = path;
      renameRequest.set(null);
    }
  });

  onDestroy(() => {
    unsubscribeRename();
  });

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

  const handleWindowClick = (event: MouseEvent) => {
    const target = event.target;
    if (target instanceof Element && target.closest(".context")) return;
    closeContext();
  };

  const parentFolder = (path: string) => {
    const parts = path.split(/[/\\]/);
    parts.pop();
    return parts.join("/");
  };

  const fileName = (path: string) => path.split(/[/\\]/).pop() ?? path;

  const handleCreateNote = async () => {
    const title = window.prompt("Note title");
    if (!title) return closeContext();
    const vault = $vaultPath;
    if (!vault) return closeContext();
    try {
      const target = contextNode?.path ?? vault;
      const base = contextNode?.is_folder ? target : parentFolder(target);
      const folder = toVaultRelative(base, vault);
      const path = await createNote(folder, title);
      if (path) openNote(path);
    } catch (err) {
      pushToast(formatError(err) || "Could not create that note.");
    } finally {
      closeContext();
    }
  };

  const handleCreateFolder = async () => {
    const name = window.prompt("Folder name");
    if (!name) return closeContext();
    const vault = $vaultPath;
    if (!vault) return closeContext();
    try {
      const target = contextNode?.path ?? vault;
      const base = contextNode?.is_folder ? target : parentFolder(target);
      const relative = toVaultRelative(base, vault);
      const folderPath = relative ? `${relative}/${name}` : name;
      await createFolder(folderPath);
    } catch (err) {
      pushToast(formatError(err) || "Could not create that folder.");
    } finally {
      closeContext();
    }
  };

  const handleRename = () => {
    if (!contextNode) return closeContext();
    editingPath = contextNode.path;
    closeContext();
  };

  const handleRenameCommit = async (newName: string) => {
    if (!editingPath || !newName || newName === fileName(editingPath)) {
      editingPath = null;
      return;
    }

    try {
      await renameItem(editingPath, newName);
    } catch (err) {
      pushToast(formatError(err) || "Could not rename that item.");
    }

    editingPath = null;
  };

  const handleDelete = () => {
    if (!contextNode) return closeContext();
    deleteMessage = `Delete ${contextNode.name}?`;
    deleteDialogOpen = true;
    closeContext();
  };

  const onConfirmDelete = async () => {
    try {
      if (contextNode) {
        await deleteItem(contextNode.path);
      }
    } catch (err) {
      pushToast(formatError(err) || "Could not delete that item.");
    } finally {
      deleteDialogOpen = false;
    }
  };

  const onCancelDelete = () => {
    deleteDialogOpen = false;
  };
</script>

<svelte:window on:click={handleWindowClick} />

<section class="panel">
  <div class="header">
    <div>
      <div class="eyebrow">Library</div>
      <h3>Vault</h3>
    </div>
    {#if $focusedPath}
      <span class="focus"
        >{($focusedIsFolder ? "Folder" : "File") + " selected"}</span
      >
    {/if}
  </div>
  <div class="tree">
    {#each $fileTree as node (node.path)}
      <TreeNode
        {node}
        {expanded}
        {toggleFolder}
        onContext={onContextMenu}
        {editingPath}
        onRenameCommit={handleRenameCommit}
      />
    {/each}
  </div>
</section>

<ConfirmDialog
  open={deleteDialogOpen}
  message={deleteMessage}
  onConfirm={onConfirmDelete}
  onCancel={onCancelDelete}
/>

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
    height: 100%;
    padding: var(--space-4);
    background: linear-gradient(
      180deg,
      rgba(18, 27, 43, 0.64),
      rgba(11, 17, 29, 0.4)
    );
  }
  .header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--border-subtle);
  }
  .eyebrow {
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    margin-bottom: 4px;
  }
  h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 18px;
    letter-spacing: -0.03em;
  }
  .focus {
    padding: 6px 10px;
    border-radius: 999px;
    border: 1px solid var(--border-subtle);
    background: rgba(14, 22, 36, 0.78);
    color: var(--text-secondary);
    font-size: 12px;
    white-space: nowrap;
  }
  .tree {
    display: grid;
    gap: 4px;
  }
  .context {
    position: fixed;
    min-width: 168px;
    background: rgba(16, 24, 39, 0.96);
    border: 1px solid var(--border);
    border-radius: 18px;
    padding: var(--space-2);
    display: grid;
    gap: var(--space-1);
    z-index: 50;
    box-shadow: var(--shadow-soft);
  }
  .context button {
    background: rgba(14, 22, 36, 0.78);
    border: 1px solid var(--border-subtle);
    padding: 10px 12px;
    text-align: left;
    color: var(--text-primary);
  }
  .context .danger {
    border-color: rgba(255, 139, 145, 0.26);
    color: var(--danger);
  }
</style>
