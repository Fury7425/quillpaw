<script lang="ts">
  import { get } from "svelte/store";
  import {
    aiEnabled,
    aiProposals,
    aiStatus,
    aiModelInfo,
    addProposal,
  } from "$lib/stores/ai";
  import { activePath, noteBody } from "$lib/stores/editor";
  import { vaultPath } from "$lib/stores/vault";
  import { pushToast } from "$lib/stores/ui";
  import { tauriInvoke } from "$lib/utils/tauri_bridge";
  import ProposalCard from "$lib/components/ProposalCard.svelte";
  import type { AiProposal } from "$lib/types";

  let question = "";
  let busy = false;

  const ensureNote = (): string | null => {
    const path = get(activePath);
    if (!path) {
      pushToast("Open a note to use AI actions.");
      return null;
    }
    return path;
  };

  const runAction = async <T,>(fn: () => Promise<T>) => {
    busy = true;
    try {
      return await fn();
    } catch (err) {
      pushToast(err instanceof Error ? err.message : String(err));
    } finally {
      busy = false;
    }
  };

  const summarize = async () => {
    const path = ensureNote();
    if (!path) return;
    await runAction(async () => {
      const proposal = await tauriInvoke<AiProposal>("summarize_note", {
        noteContent: get(noteBody),
        targetPath: path,
      });
      addProposal(proposal);
    });
  };

  const ask = async () => {
    const vault = get(vaultPath);
    if (!vault) {
      pushToast("Open a vault to ask questions.");
      return;
    }
    await runAction(async () => {
      const proposal = await tauriInvoke<AiProposal>("ask_question", {
        vaultPath: vault,
        question,
        targetPath: get(activePath),
      });
      addProposal(proposal);
      question = "";
    });
  };

  const reminders = async () => {
    const path = ensureNote();
    if (!path) return;
    await runAction(async () => {
      const proposals = await tauriInvoke<AiProposal[]>("detect_reminders", {
        noteContent: get(noteBody),
        targetPath: path,
      });
      proposals.forEach(addProposal);
    });
  };

  const suggestTags = async () => {
    const vault = get(vaultPath);
    const path = ensureNote();
    if (!vault || !path) return;
    await runAction(async () => {
      const proposal = await tauriInvoke<AiProposal>("suggest_tags", {
        vaultPath: vault,
        noteContent: get(noteBody),
        targetPath: path,
      });
      addProposal(proposal);
    });
  };
</script>

<div class="panel">
  <h3>AI Panel</h3>
  <p class="status">{$aiStatus}</p>

  <div class="section">
    <h4>Model Status</h4>
    {#if $aiModelInfo?.loaded}
      <p class="muted">Loaded: {$aiModelInfo.model_path}</p>
      <p class="muted">Device: {$aiModelInfo.device_mode}</p>
    {:else}
      <p class="muted">No model loaded.</p>
    {/if}
  </div>

  <div class="section">
    <h4>Actions</h4>
    <button disabled={!$aiEnabled || busy} on:click={summarize}
      >Summarize</button
    >
    <label class="field">
      <span>Ask a question</span>
      <input
        placeholder="What did I decide about the launch?"
        bind:value={question}
        disabled={!$aiEnabled || busy}
      />
    </label>
    <button disabled={!$aiEnabled || busy || !question.trim()} on:click={ask}
      >Ask Q&amp;A</button
    >
    <button disabled={!$aiEnabled || busy} on:click={reminders}
      >Reminders Scan</button
    >
    <button disabled={!$aiEnabled || busy} on:click={suggestTags}
      >Tag Suggestions</button
    >
  </div>

  <div class="section">
    <h4>Proposals</h4>
    {#if $aiProposals.length === 0}
      <p class="muted">No proposals yet.</p>
    {:else}
      {#each $aiProposals as proposal (proposal.id)}
        <ProposalCard {proposal} />
      {/each}
    {/if}
  </div>
</div>

<style>
  .panel {
    padding: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }
  h3 {
    margin: 0;
  }
  .status {
    color: var(--text-secondary);
    margin: 0;
  }
  .section h4 {
    margin: 0 0 var(--space-2);
    color: var(--text-secondary);
  }
  .muted {
    color: var(--text-muted);
  }
  button {
    width: 100%;
    margin-bottom: var(--space-2);
  }
  .field {
    display: grid;
    gap: var(--space-1);
    margin-bottom: var(--space-2);
    color: var(--text-secondary);
  }
  input {
    width: 100%;
  }
</style>
