<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { load, type Store } from "@tauri-apps/plugin-store";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { listen } from "@tauri-apps/api/event";
  import { get } from "svelte/store";

  import { uiState, toggleSettings, pushToast } from "$lib/stores/ui";
  import { vaultPath } from "$lib/stores/vault";
  import {
    aiDeviceMode,
    aiEnabled,
    aiModelPath,
    aiStatus,
    setModelStatus,
  } from "$lib/stores/ai";
  import { sttDevice, sttModelPath } from "$lib/stores/stt";
  import { tauriInvoke } from "$lib/utils/tauri_bridge";
  import type { AiModelStatus, ModelDownloadProgress } from "$lib/types";

  export let open = false;

  let configStore: Store | null = null;
  let devices: string[] = [];
  let npuAvailable = false;
  let downloadMap: Record<string, ModelDownloadProgress> = {};
  let unsubscribeDownload: (() => void) | null = null;
  let lastVaultPath = "";

  const loadConfig = async () => {
    const vault = get(vaultPath);
    if (!vault) return;
    const store = await load(`${vault}/.quillpaw/config.json`);
    configStore = store;
    const storedAiPath = await store.get<string>("aiModelPath");
    const storedDeviceMode = await store.get<string>("aiDeviceMode");
    const storedSttPath = await store.get<string>("sttModelPath");
    const storedDevice = await store.get<string>("sttDevice");
    const storedTheme = await store.get<string>("theme");

    if (
      storedTheme === "Dark Warm" ||
      storedTheme === "Dark Cool" ||
      storedTheme === "Light Parchment"
    ) {
      uiState.update((state) => ({
        ...state,
        theme: storedTheme as "Dark Warm" | "Dark Cool" | "Light Parchment",
      }));
      document.documentElement.setAttribute("data-theme", storedTheme);
    } else {
      document.documentElement.setAttribute("data-theme", "Dark Warm");
    }

    if (storedAiPath) aiModelPath.set(storedAiPath);
    if (
      storedDeviceMode === "cpu" ||
      storedDeviceMode === "npu" ||
      storedDeviceMode === "auto"
    ) {
      aiDeviceMode.set(storedDeviceMode);
    }
    if (storedSttPath) {
      sttModelPath.set(storedSttPath);
      await tauriInvoke("set_stt_model_path", { modelPath: storedSttPath });
    }
    if (storedDevice) {
      sttDevice.set(storedDevice);
      await tauriInvoke("set_audio_device", { deviceName: storedDevice });
    }
    await refreshStatus();
  };

  const refreshStatus = async () => {
    try {
      const status = await tauriInvoke<AiModelStatus>("get_ai_status");
      setModelStatus(status);
      npuAvailable = await tauriInvoke<boolean>("detect_npu");
    } catch (err) {
      pushToast(err instanceof Error ? err.message : String(err));
    }
  };

  const refreshDevices = async () => {
    try {
      devices = await tauriInvoke<string[]>("list_audio_devices");
    } catch (err) {
      pushToast(err instanceof Error ? err.message : String(err));
    }
  };

  const pickModelPath = async () => {
    const selected = await openDialog({
      filters: [{ name: "GGUF", extensions: ["gguf"] }],
    });
    if (typeof selected === "string") {
      aiModelPath.set(selected);
      await configStore?.set("aiModelPath", selected);
      await configStore?.save();
    }
  };

  const pickSttModel = async () => {
    const selected = await openDialog({
      filters: [{ name: "Model", extensions: ["bin", "gguf"] }],
    });
    if (typeof selected === "string") {
      sttModelPath.set(selected);
      await tauriInvoke("set_stt_model_path", { modelPath: selected });
      await configStore?.set("sttModelPath", selected);
      await configStore?.save();
    }
  };

  const applyDeviceMode = async (mode: "auto" | "cpu" | "npu") => {
    aiDeviceMode.set(mode);
    await configStore?.set("aiDeviceMode", mode);
    await configStore?.save();
  };

  const applyAudioDevice = async (device: string) => {
    sttDevice.set(device);
    await tauriInvoke("set_audio_device", { deviceName: device });
    await configStore?.set("sttDevice", device);
    await configStore?.save();
  };

  const applyTheme = async (event: Event) => {
    const target = event.target as HTMLSelectElement;
    const theme = target.value as "Dark Warm" | "Dark Cool" | "Light Parchment";
    uiState.update((state) => ({ ...state, theme }));
    document.documentElement.setAttribute("data-theme", theme);
    await configStore?.set("theme", theme);
    await configStore?.save();
  };

  const toggleAi = async () => {
    try {
      if (get(aiEnabled)) {
        const status = await tauriInvoke<AiModelStatus>("unload_ai_model");
        setModelStatus(status);
      } else {
        const path = get(aiModelPath);
        if (!path) {
          pushToast("Select a model path first.");
          return;
        }
        const status = await tauriInvoke<AiModelStatus>("load_ai_model", {
          modelPath: path,
          deviceMode: get(aiDeviceMode),
        });
        setModelStatus(status);
      }
    } catch (err) {
      pushToast(err instanceof Error ? err.message : String(err));
    }
  };

  const downloadModel = async (modelId: string) => {
    const vault = get(vaultPath);
    if (!vault) {
      pushToast("Open a vault first.");
      return;
    }
    try {
      await tauriInvoke("download_ai_model", {
        vaultPath: vault,
        modelId,
      });
    } catch (err) {
      pushToast(err instanceof Error ? err.message : String(err));
    }
  };

  const downloadProgress = (id: string) => {
    const progress = downloadMap[id];
    if (!progress) return 0;
    if (!progress.total) return 0;
    return Math.min(100, (progress.downloaded / progress.total) * 100);
  };

  onMount(async () => {
    await loadConfig();
    await refreshDevices();
    unsubscribeDownload = await listen<ModelDownloadProgress>(
      "model-download-progress",
      (event) => {
        downloadMap = {
          ...downloadMap,
          [event.payload.model_id]: event.payload,
        };
        if (event.payload.done && event.payload.path) {
          aiModelPath.set(event.payload.path);
          configStore?.set("aiModelPath", event.payload.path);
          configStore?.save();
        }
      },
    );
  });

  onDestroy(() => {
    unsubscribeDownload?.();
  });

  $: if ($vaultPath && $vaultPath !== lastVaultPath) {
    lastVaultPath = $vaultPath;
    loadConfig();
  }
</script>

{#if open}
  <div class="drawer">
    <div class="header">
      <h3>Settings</h3>
      <button on:click={toggleSettings}>Close</button>
    </div>

    <section>
      <h4>General</h4>
      <label for="vault-path">Vault Path</label>
      <input
        id="vault-path"
        value={$vaultPath || ""}
        placeholder="Select a vault"
        disabled
      />
      <label for="theme-select">Theme</label>
      <select id="theme-select" value={$uiState.theme} on:change={applyTheme}>
        <option value="Dark Warm">Dark Warm</option>
        <option value="Dark Cool">Dark Cool</option>
        <option value="Light Parchment">Light Parchment</option>
      </select>
    </section>

    <section>
      <h4>Editor</h4>
      <label for="autosave-delay">Auto-save delay (ms)</label>
      <input id="autosave-delay" type="number" value="800" />
      <label for="line-numbers">Line numbers</label>
      <input id="line-numbers" type="checkbox" checked />
      <label for="focus-mode">Focus mode</label>
      <input id="focus-mode" type="checkbox" />
    </section>

    <section>
      <h4>AI</h4>
      <p class="muted">{$aiStatus}</p>
      <label for="ai-model-path">Model Path</label>
      <input
        id="ai-model-path"
        value={$aiModelPath}
        placeholder="No model selected"
        disabled
      />
      <div class="row">
        <button on:click={pickModelPath}>Set Model Path</button>
        <button class="primary" on:click={toggleAi}
          >{$aiEnabled ? "Unload AI" : "Enable AI"}</button
        >
      </div>
      <label for="ai-device-mode">Device Mode</label>
      <select
        id="ai-device-mode"
        value={$aiDeviceMode}
        on:change={(event) =>
          applyDeviceMode(
            (event.currentTarget as HTMLSelectElement).value as
              | "auto"
              | "cpu"
              | "npu",
          )}
      >
        <option value="auto">Auto</option>
        <option value="cpu">CPU</option>
        <option value="npu" disabled={!npuAvailable}>Low (NPU)</option>
      </select>
      <div class="downloads">
        <h5>Model Downloads</h5>
        <button on:click={() => downloadModel("phi-3-mini")}
          >Download Phi-3 Mini</button
        >
        <div class="progress">
          <div
            class="bar"
            style={`width:${downloadProgress("phi-3-mini")}%`}
          ></div>
        </div>
        <button on:click={() => downloadModel("qwen2.5-1.5b")}
          >Download Qwen2.5 1.5B</button
        >
        <div class="progress">
          <div
            class="bar"
            style={`width:${downloadProgress("qwen2.5-1.5b")}%`}
          ></div>
        </div>
      </div>
    </section>

    <section>
      <h4>Speech</h4>
      <label for="stt-device">Input Device</label>
      <select
        id="stt-device"
        value={$sttDevice}
        on:change={(event) =>
          applyAudioDevice((event.currentTarget as HTMLSelectElement).value)}
      >
        <option value="">Default</option>
        {#each devices as device}
          <option value={device}>{device}</option>
        {/each}
      </select>
      <label for="stt-model-path">STT Model</label>
      <input
        id="stt-model-path"
        value={$sttModelPath}
        placeholder="No model selected"
        disabled
      />
      <button on:click={pickSttModel}>Set STT Model</button>
    </section>

    <section>
      <h4>Privacy</h4>
      <label>
        <input type="checkbox" />
        Telemetry (default OFF)
      </label>
      <p class="muted">Quillpaw stores all data locally.</p>
    </section>
  </div>
{/if}

<style>
  .drawer {
    position: fixed;
    top: 36px;
    right: 0;
    bottom: 28px;
    width: 360px;
    background: var(--bg-elevated);
    border-left: 1px solid var(--border);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    z-index: 35;
    overflow: auto;
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  section {
    display: grid;
    gap: var(--space-2);
  }
  h4 {
    margin: 0;
    color: var(--text-secondary);
  }
  h5 {
    margin: var(--space-2) 0 0;
    font-size: 12px;
    color: var(--text-muted);
  }
  .muted {
    color: var(--text-muted);
  }
  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-2);
  }
  .primary {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--bg-base);
  }
  .downloads {
    display: grid;
    gap: var(--space-2);
  }
  .progress {
    height: 6px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .bar {
    height: 100%;
    background: var(--accent);
  }
</style>
