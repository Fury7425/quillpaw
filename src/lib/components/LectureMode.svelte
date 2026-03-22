<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { get } from "svelte/store";

  import { sttActive, sttDevice, sttModelPath } from "$lib/stores/stt";
  import { vaultPath } from "$lib/stores/vault";
  import { requestInsert } from "$lib/stores/editor";
  import { pushToast } from "$lib/stores/ui";
  import { tauriInvoke } from "$lib/utils/tauri_bridge";

  let transcript = "";
  let unlisten: (() => void) | null = null;

  const start = async () => {
    if (!get(vaultPath)) {
      pushToast("Open a vault to start lecture mode.");
      return;
    }
    if (!get(sttModelPath)) {
      pushToast("Set an STT model path in Settings.");
      return;
    }
    try {
      await tauriInvoke("start_lecture_mode", { vaultPath: get(vaultPath) });
      sttActive.set(true);
    } catch (err) {
      pushToast(err instanceof Error ? err.message : String(err));
    }
  };

  const stop = async () => {
    try {
      await tauriInvoke("stop_lecture_mode");
    } catch (err) {
      pushToast(err instanceof Error ? err.message : String(err));
    } finally {
      sttActive.set(false);
    }
  };

  onMount(() => {
    const setup = async () => {
      try {
        unlisten = await listen<{ text: string; is_final: boolean }>(
          "stt-text-chunk",
          (event) => {
            const text = event.payload.text.trim();
            if (!text) return;
            transcript = `${transcript} ${text}`.trim();
            requestInsert(`${text} `);
          },
        );
        if (!get(sttActive)) {
          await start();
        }
      } catch (err) {
        pushToast(err instanceof Error ? err.message : String(err));
      }
    };

    void setup();
  });

  onDestroy(() => {
    unlisten?.();
    if (get(sttActive)) {
      stop();
    }
  });
</script>

<div class="lecture">
  <div>
    <strong>Lecture Mode</strong>
    <div class="meta">
      Device: {$sttDevice || "Default"} - Model: {$sttModelPath || "Unset"}
    </div>
    {#if transcript}
      <div class="transcript">{transcript}</div>
    {/if}
  </div>
  {#if $sttActive}
    <button on:click={stop}>Stop</button>
  {:else}
    <button on:click={start}>Start</button>
  {/if}
</div>

<style>
  .lecture {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-4);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    gap: var(--space-3);
  }
  .meta {
    font-size: 12px;
    color: var(--text-secondary);
  }
  .transcript {
    margin-top: var(--space-1);
    font-size: 12px;
    color: var(--text-muted);
  }
</style>
