<script lang="ts">
  import { onMount } from "svelte";

  export let open = false;
  export let message = "";
  export let onConfirm: () => void;
  export let onCancel: () => void;

  const handleKeydown = (event: KeyboardEvent) => {
    if (!open) return;
    if (event.key === "Escape") {
      onCancel();
    } else if (event.key === "Enter") {
      onConfirm();
    }
  };

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

{#if open}
  <div class="overlay" role="alertdialog" aria-modal="true">
    <div class="dialog">
      <p>{message}</p>
      <div class="actions">
        <button class="ghost" on:click={onCancel}>Cancel</button>
        <button class="danger" on:click={onConfirm}>Confirm</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: var(--overlay);
    display: grid;
    place-items: center;
    z-index: 38;
  }
  .dialog {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    box-shadow: var(--shadow-strong);
    width: 320px;
    text-align: center;
  }
  p {
    margin: 0 0 var(--space-6);
    color: var(--text-primary);
    font-size: 16px;
  }
  .actions {
    display: flex;
    gap: var(--space-3);
    justify-content: center;
  }
  button {
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: 14px;
    transition: all var(--transition);
  }
  .danger {
    background: var(--danger);
    border: 1px solid var(--danger);
    color: white;
  }
  .danger:hover {
    filter: brightness(1.1);
  }
  .ghost {
    background: transparent;
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
  }
  .ghost:hover {
    background: var(--bg-surface);
    border-color: var(--border);
    color: var(--text-primary);
  }
</style>
