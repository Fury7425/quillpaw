<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { openVault, createNote, createFolder, focusedPath, focusedIsFolder, vaultPath, toVaultRelative } from '$lib/stores/vault';
  import { openNote } from '$lib/stores/editor';
  import { closeCommand, toggleLeft, toggleRight, toggleSettings } from '$lib/stores/ui';

  export let open = false;

  let query = '';
  const dispatch = createEventDispatcher();

  const commands = [
    { id: 'open-vault', label: 'Open Vault', action: () => openVault() },
    { id: 'new-note', label: 'New Note', action: async () => {
      const title = window.prompt('Note title');
      if (!title) return;
      const vault = $vaultPath;
      if (!vault) return;
      const focused = $focusedIsFolder ? $focusedPath ?? vault : $focusedPath ?? vault;
      const base = $focusedIsFolder
        ? focused
        : focused
            .split(/[/\\]/)
            .slice(0, -1)
            .join('/') || vault;
      const folder = toVaultRelative(base ?? vault, vault);
      const path = await createNote(folder, title);
      if (path) openNote(path);
    }},
    { id: 'new-folder', label: 'New Folder', action: async () => {
      const name = window.prompt('Folder name');
      if (!name) return;
      const vault = $vaultPath;
      if (!vault) return;
      const focused = $focusedIsFolder ? $focusedPath ?? vault : $focusedPath ?? vault;
      const base = $focusedIsFolder
        ? focused
        : focused
            .split(/[/\\]/)
            .slice(0, -1)
            .join('/') || vault;
      const relative = toVaultRelative(base ?? vault, vault);
      const folderPath = relative ? `${relative}/${name}` : name;
      await createFolder(folderPath);
    }},
    { id: 'toggle-left', label: 'Toggle Left Panel', action: () => toggleLeft() },
    { id: 'toggle-right', label: 'Toggle Right Panel', action: () => toggleRight() },
    { id: 'settings', label: 'Settings', action: () => toggleSettings() }
  ];

  $: filtered = commands.filter((cmd) => cmd.label.toLowerCase().includes(query.toLowerCase()));

  const run = async (action: () => void | Promise<void>) => {
    await action();
    closeCommand();
    query = '';
    dispatch('close');
  };
</script>

{#if open}
  <div class="backdrop" on:click={closeCommand}>
    <div class="modal" on:click|stopPropagation>
      <input placeholder="Type a command" bind:value={query} />
      <div class="list">
        {#each filtered as cmd}
          <button on:click={() => run(cmd.action)}>{cmd.label}</button>
        {/each}
        {#if filtered.length === 0}
          <div class="empty">No matches.</div>
        {/if}
      </div>
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
    width: 480px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-strong);
    padding: var(--space-3);
  }
  input {
    width: 100%;
    margin-bottom: var(--space-2);
  }
  .list {
    display: grid;
    gap: var(--space-1);
  }
  .list button {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    padding: var(--space-2);
    text-align: left;
  }
  .empty {
    color: var(--text-muted);
    padding: var(--space-2);
  }
</style>
