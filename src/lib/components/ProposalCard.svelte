<script lang="ts">
  import { get } from 'svelte/store';
  import type { AiProposal } from '$lib/types';
  import { tauriInvoke } from '$lib/utils/tauri_bridge';
  import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
  import { removeProposal, updateProposal } from '$lib/stores/ai';
  import { activePath, openNote } from '$lib/stores/editor';
  import { pushToast } from '$lib/stores/ui';

  export let proposal: AiProposal;

  const apply = async () => {
    if (!proposal.target_path) {
      pushToast('This proposal has no target note.');
      return;
    }
    try {
      await tauriInvoke('apply_ai_proposal', {
        proposalId: proposal.id,
        targetPath: proposal.target_path
      });
      if (proposal.proposal_type === 'reminder') {
        const granted = await isPermissionGranted();
        if (!granted) {
          await requestPermission();
        }
        sendNotification({ title: 'Quillpaw Reminder', body: proposal.content });
      }
      removeProposal(proposal.id);
      if (get(activePath) === proposal.target_path) {
        await openNote(proposal.target_path);
      }
    } catch (err) {
      pushToast(err instanceof Error ? err.message : String(err));
    }
  };

  const edit = () => {
    const updated = window.prompt('Edit proposal text', proposal.content);
    if (updated !== null) {
      updateProposal(proposal.id, updated);
    }
  };

  const dismiss = () => {
    removeProposal(proposal.id);
  };
</script>

<div class="card">
  <div class="header">
    <span class="type">{proposal.proposal_type}</span>
    <span class="title">{proposal.title}</span>
  </div>
  <p>{proposal.content}</p>
  <div class="actions">
    <button class="primary" on:click={apply} disabled={!proposal.target_path}>Accept</button>
    <button on:click={edit}>Edit</button>
    <button class="ghost" on:click={dismiss}>Dismiss</button>
  </div>
</div>

<style>
  .card {
    border-left: 3px solid var(--accent);
    background: var(--accent-glow);
    border-radius: var(--radius-md);
    padding: var(--space-3);
    margin-bottom: var(--space-2);
  }
  .header {
    display: flex;
    gap: var(--space-2);
    align-items: baseline;
    margin-bottom: var(--space-2);
  }
  .type {
    text-transform: uppercase;
    font-size: 10px;
    color: var(--text-muted);
  }
  .title {
    font-weight: 600;
  }
  .actions {
    display: flex;
    gap: var(--space-2);
  }
  .primary {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--bg-base);
  }
  .ghost {
    background: transparent;
  }
</style>
