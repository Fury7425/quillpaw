<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { listen } from "@tauri-apps/api/event";
  import { get } from "svelte/store";

  import { uiState, toggleSettings, pushToast } from "$lib/stores/ui";
  import { vaultPath } from "$lib/stores/vault";
  import { setSavedTheme } from "$lib/stores/preferences";
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

  type VaultConfigStore = {
    get<T>(key: string): Promise<T | null>;
    set(key: string, value: unknown): Promise<void>;
    save(): Promise<void>;
  };

  let configStore: VaultConfigStore | null = null;
  let devices: string[] = [];
  let npuAvailable = false;
  let downloadMap: Record<string, ModelDownloadProgress> = {};
  let unsubscribeDownload: (() => void) | null = null;
  let lastVaultPath = "";

  const reportError = (
    err: unknown,
    fallback = "A settings action failed.",
  ) => {
    const message = err instanceof Error ? err.message : String(err);
    pushToast(message || fallback);
  };

  const persistConfigValue = async (key: string, value: unknown) => {
    if (!configStore) return;
    try {
      await configStore.set(key, value);
      await configStore.save();
    } catch (err) {
      reportError(err, "Could not save that preference.");
    }
  };

  const applyDocumentTheme = (
    theme: "Dark Warm" | "Dark Cool" | "Light Parchment",
  ) => {
    uiState.update((state) => ({ ...state, theme }));
    document.documentElement.setAttribute("data-theme", theme);
  };

  const loadConfig = async () => {
    const vault = get(vaultPath);
    if (!vault) {
      configStore = null;
      return;
    }

    try {
      const { load } = await import("@tauri-apps/plugin-store");
      const store = await load(`${vault}/.quillpaw/config.json`);
      configStore = store;

      const storedAiPath = await store.get<string>("aiModelPath");
      const storedDeviceMode = await store.get<string>("aiDeviceMode");
      const storedSttPath = await store.get<string>("sttModelPath");
      const storedDevice = await store.get<string>("sttDevice");

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
    } catch (err) {
      reportError(err, "Could not load vault settings.");
    }
  };

  const refreshStatus = async () => {
    try {
      const status = await tauriInvoke<AiModelStatus>("get_ai_status");
      setModelStatus(status);
      npuAvailable = await tauriInvoke<boolean>("detect_npu");
    } catch (err) {
      reportError(err, "Could not refresh AI status.");
    }
  };

  const refreshDevices = async () => {
    try {
      devices = await tauriInvoke<string[]>("list_audio_devices");
    } catch (err) {
      reportError(err, "Could not list audio devices.");
    }
  };

  const connectDownloadUpdates = async () => {
    unsubscribeDownload?.();
    unsubscribeDownload = null;

    try {
      unsubscribeDownload = await listen<ModelDownloadProgress>(
        "model-download-progress",
        (event) => {
          downloadMap = {
            ...downloadMap,
            [event.payload.model_id]: event.payload,
          };
          if (event.payload.error) {
            pushToast(event.payload.error);
          }
          if (event.payload.done && event.payload.path) {
            aiModelPath.set(event.payload.path);
            void persistConfigValue("aiModelPath", event.payload.path);
          }
        },
      );
    } catch (err) {
      reportError(err, "Could not listen for download progress.");
    }
  };

  const pickModelPath = async () => {
    try {
      const selected = await openDialog({
        filters: [{ name: "GGUF", extensions: ["gguf"] }],
      });
      if (typeof selected === "string") {
        aiModelPath.set(selected);
        await persistConfigValue("aiModelPath", selected);
      }
    } catch (err) {
      reportError(err, "Could not select that AI model.");
    }
  };

  const pickSttModel = async () => {
    try {
      const selected = await openDialog({
        filters: [{ name: "Model", extensions: ["bin", "gguf"] }],
      });
      if (typeof selected === "string") {
        sttModelPath.set(selected);
        await tauriInvoke("set_stt_model_path", { modelPath: selected });
        await persistConfigValue("sttModelPath", selected);
      }
    } catch (err) {
      reportError(err, "Could not select that speech model.");
    }
  };

  const applyDeviceMode = async (mode: "auto" | "cpu" | "npu") => {
    aiDeviceMode.set(mode);
    await persistConfigValue("aiDeviceMode", mode);
  };

  const applyAudioDevice = async (device: string) => {
    try {
      sttDevice.set(device);
      await tauriInvoke("set_audio_device", { deviceName: device });
      await persistConfigValue("sttDevice", device);
    } catch (err) {
      reportError(err, "Could not switch the input device.");
    }
  };

  const applyTheme = async (event: Event) => {
    const target = event.target as HTMLSelectElement;
    const theme = target.value as "Dark Warm" | "Dark Cool" | "Light Parchment";
    applyDocumentTheme(theme);
    try {
      await setSavedTheme(theme);
    } catch (err) {
      reportError(err, "Could not save that theme.");
    }
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
      reportError(err, "Could not change AI state.");
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
      reportError(err, "Could not start the download.");
    }
  };

  const downloadProgress = (id: string) => {
    const progress = downloadMap[id];
    if (!progress || !progress.total) return 0;
    return Math.min(100, (progress.downloaded / progress.total) * 100);
  };

  const initialize = async () => {
    await loadConfig();
    await refreshDevices();
    await connectDownloadUpdates();
  };

  onMount(() => {
    void initialize();
  });

  onDestroy(() => {
    unsubscribeDownload?.();
  });

  $: if ($vaultPath && $vaultPath !== lastVaultPath) {
    lastVaultPath = $vaultPath;
    void loadConfig();
  }
</script>

{#if open}
  <div class="drawer">
    <div class="header">
      <div>
        <div class="eyebrow">Workspace controls</div>
        <h3>Settings</h3>
      </div>
      <button type="button" class="ghost close-btn" on:click={toggleSettings}>
        Close
      </button>
    </div>

    <section class="section-card">
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

    <section class="section-card">
      <h4>Editor</h4>
      <label for="autosave-delay">Auto-save delay (ms)</label>
      <input id="autosave-delay" type="number" value="800" />
      <label for="line-numbers">Line numbers</label>
      <input id="line-numbers" type="checkbox" checked />
      <label for="focus-mode">Focus mode</label>
      <input id="focus-mode" type="checkbox" />
    </section>

    <section class="section-card">
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
        <button type="button" class="ghost" on:click={() => void pickModelPath()}>
          Set Model Path
        </button>
        <button type="button" class="primary" on:click={() => void toggleAi()}>
          {$aiEnabled ? "Unload AI" : "Enable AI"}
        </button>
      </div>
      <label for="ai-device-mode">Device Mode</label>
      <select
        id="ai-device-mode"
        value={$aiDeviceMode}
        on:change={(event) =>
          void applyDeviceMode(
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
        <button type="button" class="ghost" on:click={() => void downloadModel("phi-3-mini")}>
          Download Phi-3 Mini
        </button>
        <div class="progress">
          <div class="bar" style={`width:${downloadProgress("phi-3-mini")}%`}></div>
        </div>
        <button
          type="button"
          class="ghost"
          on:click={() => void downloadModel("qwen2.5-1.5b")}
        >
          Download Qwen2.5 1.5B
        </button>
        <div class="progress">
          <div class="bar" style={`width:${downloadProgress("qwen2.5-1.5b")}%`}></div>
        </div>
      </div>
    </section>

    <section class="section-card">
      <h4>Speech</h4>
      <label for="stt-device">Input Device</label>
      <select
        id="stt-device"
        value={$sttDevice}
        on:change={(event) =>
          void applyAudioDevice((event.currentTarget as HTMLSelectElement).value)}
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
      <button type="button" class="ghost" on:click={() => void pickSttModel()}>
        Set STT Model
      </button>
    </section>

    <section class="section-card">
      <h4>Privacy</h4>
      <label class="toggle-row">
        <input type="checkbox" />
        <span>Telemetry (default OFF)</span>
      </label>
      <p class="muted">Quillpaw keeps notes, models, and indexes on your device.</p>
    </section>
  </div>
{/if}

<style>
  .drawer {
    position: fixed;
    top: 72px;
    right: 0;
    bottom: 40px;
    width: min(420px, calc(100vw - 24px));
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    overflow: auto;
    z-index: 35;
    background: linear-gradient(180deg, var(--bg-panel), var(--glass));
    border-left: 1px solid var(--border);
    backdrop-filter: blur(20px);
    box-shadow: -18px 0 48px rgba(0, 0, 0, 0.28);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-3);
  }

  .eyebrow {
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  h3 {
    margin: 8px 0 0;
    font-size: 28px;
    letter-spacing: -0.04em;
  }

  .section-card {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-4);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: 20px;
    box-shadow: var(--shadow-soft);
  }

  h4 {
    margin: 0;
    font-size: 16px;
    color: var(--text-primary);
  }

  h5 {
    margin: var(--space-2) 0 0;
    color: var(--text-muted);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .muted {
    margin: 0;
    color: var(--text-secondary);
  }

  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-2);
  }

  .primary {
    background: linear-gradient(135deg, var(--accent), var(--accent2));
    border-color: transparent;
    color: #04111c;
    font-weight: 700;
  }

  .ghost {
    background: var(--bg-elevated);
    border-color: var(--border);
    color: var(--text-primary);
  }

  .downloads {
    display: grid;
    gap: var(--space-2);
  }

  .progress {
    height: 8px;
    border-radius: 999px;
    overflow: hidden;
    background: var(--glass);
    border: 1px solid var(--border-subtle);
  }

  .bar {
    height: 100%;
    background: linear-gradient(135deg, var(--accent), var(--accent2));
  }

  .toggle-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-primary);
  }

  .close-btn {
    min-width: 86px;
  }

  @media (max-width: 760px) {
    .drawer {
      top: 108px;
      left: 12px;
      right: 12px;
      bottom: 56px;
      width: auto;
      border-radius: 24px;
      border: 1px solid var(--border);
    }

    .row {
      grid-template-columns: 1fr;
    }
  }
</style>
