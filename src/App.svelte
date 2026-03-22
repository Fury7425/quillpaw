<script lang="ts">
  import { onMount } from "svelte";
  import { PenNib, Minus, Square, X } from "phosphor-svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { get } from "svelte/store";

  import FileTree from "$lib/components/FileTree.svelte";
  import Editor from "$lib/components/Editor.svelte";
  import AIPanel from "$lib/components/AIPanel.svelte";
  import TabBar from "$lib/components/TabBar.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import SearchModal from "$lib/components/SearchModal.svelte";
  import CommandPalette from "$lib/components/CommandPalette.svelte";
  import LectureMode from "$lib/components/LectureMode.svelte";
  import DrawingCanvas from "$lib/components/DrawingCanvas.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";

  import {
    openVault,
    createVault,
    vaultPath,
    focusedPath,
    focusedIsFolder,
    toVaultRelative,
    createNote,
    deleteItem,
    renameRequest,
  } from "$lib/stores/vault";
  import {
    activeNote,
    noteBody,
    openTabs,
    activePath,
    openNote,
    closeTab,
    saveActiveNote,
    requestInsert,
  } from "$lib/stores/editor";
  import {
    uiState,
    toggleLeft,
    toggleRight,
    openSearch,
    openCommand,
    toggleSettings,
    closeOverlays,
    toggleLecture,
    toasts,
    openDrawing,
    pushToast,
  } from "$lib/stores/ui";
  import { registerShortcuts } from "$lib/utils/shortcuts";
  import { tauriInvoke } from "$lib/utils/tauri_bridge";

  let appWindow: ReturnType<typeof getCurrentWindow> | null = null;
  let leftWidth = 300;
  let rightWidth = 340;
  let deleteDialogOpen = false;
  let deleteMessage = "";
  let userClosedRight = false;
  let lastRuntimeIssue = "";

  const formatError = (err: unknown) =>
    err instanceof Error ? err.message : String(err);

  const surfaceIssue = (err: unknown, fallback = "Something went wrong.") => {
    const message = formatError(err) || fallback;
    if (!message || message === lastRuntimeIssue) return;
    lastRuntimeIssue = message;
    pushToast(message);
    window.setTimeout(() => {
      if (lastRuntimeIssue === message) {
        lastRuntimeIssue = "";
      }
    }, 1800);
  };

  const runUiTask = async (
    task: () => Promise<unknown> | unknown,
    fallback?: string,
  ) => {
    try {
      await task();
    } catch (err) {
      surfaceIssue(err, fallback);
    }
  };

  const handleWindow = async (action: "min" | "max" | "close") => {
    const currentWindow = appWindow;
    if (!currentWindow) return;
    await runUiTask(async () => {
      if (action === "min") {
        await currentWindow.minimize();
      } else if (action === "max") {
        await currentWindow.toggleMaximize();
      } else {
        await currentWindow.close();
      }
    }, "Window controls are unavailable right now.");
  };

  const shouldIgnoreShortcut = (event: KeyboardEvent) => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) return false;
    return (
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.getAttribute("contenteditable") === "true"
    );
  };

  const cycleTab = (direction: number) => {
    const tabs = get(openTabs);
    if (!tabs.length) return;
    const current = get(activePath);
    const index = tabs.findIndex((tab) => tab.path === current);
    const nextIndex =
      index === -1 ? 0 : (index + direction + tabs.length) % tabs.length;
    openNote(tabs[nextIndex].path);
  };

  const toggleRightPanel = () => {
    const isOpen = get(uiState).rightOpen;
    toggleRight();
    userClosedRight = isOpen;
  };

  const startResize = (side: "left" | "right", event: PointerEvent) => {
    event.preventDefault();
    const onMove = (moveEvent: PointerEvent) => {
      if (side === "left") {
        leftWidth = Math.min(Math.max(240, moveEvent.clientX), 460);
      } else {
        const width = window.innerWidth - moveEvent.clientX;
        rightWidth = Math.min(Math.max(280, width), 540);
      }
    };
    const onUp = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  };

  const createNoteFromShortcut = async (useFolderPicker: boolean) => {
    const vault = get(vaultPath);
    if (!vault) return;
    const title = window.prompt("Note title");
    if (!title) return;
    let folder = "";
    if (useFolderPicker) {
      const folderInput = window.prompt("Folder (relative to vault)");
      if (folderInput) folder = folderInput;
    } else if (get(focusedPath)) {
      const focus = get(focusedPath);
      const isFolder = get(focusedIsFolder);
      const base = isFolder
        ? focus
        : focus?.split(/[/\\]/).slice(0, -1).join("/") || vault;
      folder = toVaultRelative(base ?? vault, vault);
    }
    const path = await createNote(folder, title);
    if (path) openNote(path);
  };

  const renameFocused = () => {
    const path = get(focusedPath);
    if (!path) return;
    renameRequest.set(path);
  };

  const createDrawingQuick = async () => {
    const vault = get(vaultPath);
    if (!vault) return;
    const style = getComputedStyle(document.documentElement);
    const baseColor = style.getPropertyValue("--bg-base").trim();
    const filename = `drawing-${Date.now()}.json`;
    await tauriInvoke("save_drawing", {
      vaultPath: vault,
      filename,
      drawingJson: JSON.stringify({
        version: 2,
        canvas: { width: 1920, height: 1080, background: baseColor },
        strokes: [],
        shapes: [],
        text_layers: [],
      }),
    });
    requestInsert(`![[${filename}]]`);
    openDrawing(filename);
  };

  const deleteFocused = () => {
    const path = get(focusedPath);
    if (!path) return;
    deleteMessage = "Delete the selected item?";
    deleteDialogOpen = true;
  };

  const onConfirmDelete = async () => {
    const path = get(focusedPath);
    if (path) {
      await deleteItem(path);
    }
    deleteDialogOpen = false;
  };

  const onCancelDelete = () => {
    deleteDialogOpen = false;
  };

  const basename = (path: string) =>
    path.split(/[/\\]/).filter(Boolean).pop() ?? path;

  onMount(() => {
    try {
      appWindow = getCurrentWindow();
    } catch (err) {
      surfaceIssue(err, "Window integration could not be initialized.");
    }

    const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
      event.preventDefault();
      surfaceIssue(event.reason, "An unexpected action failed.");
    };

    const handleRuntimeError = (event: ErrorEvent) => {
      surfaceIssue(event.error ?? event.message, "An unexpected app error occurred.");
    };

    const wrap =
      (handler: (event: KeyboardEvent) => void) => (event: KeyboardEvent) => {
        if (shouldIgnoreShortcut(event)) return;
        handler(event);
      };

    const unregister = registerShortcuts([
      {
        key: "n",
        ctrl: true,
        handler: wrap(() => createNoteFromShortcut(false)),
      },
      {
        key: "n",
        ctrl: true,
        shift: true,
        handler: wrap(() => createNoteFromShortcut(true)),
      },
      { key: "p", ctrl: true, handler: wrap(() => openCommand()) },
      {
        key: "f",
        ctrl: true,
        shift: true,
        handler: () => openSearch("keyword"),
      },
      { key: "s", ctrl: true, shift: true, handler: () => openSearch("smart") },
      { key: "b", ctrl: true, handler: () => toggleLeft() },
      {
        key: "/",
        ctrl: true,
        handler: () => toggleRightPanel(),
      },
      {
        key: "w",
        ctrl: true,
        handler: wrap(() => get(activePath) && closeTab(get(activePath) ?? "")),
      },
      { key: "tab", ctrl: true, handler: wrap(() => cycleTab(1)) },
      {
        key: "tab",
        ctrl: true,
        shift: true,
        handler: wrap(() => cycleTab(-1)),
      },
      { key: "s", ctrl: true, handler: wrap(() => saveActiveNote()) },
      { key: ",", ctrl: true, handler: wrap(() => toggleSettings()) },
      {
        key: "d",
        ctrl: true,
        handler: wrap(() => {
          void runUiTask(
            () => createDrawingQuick(),
            "Could not create a drawing canvas.",
          );
        }),
      },
      {
        key: "l",
        ctrl: true,
        shift: true,
        handler: wrap(() => toggleLecture()),
      },
      { key: "f2", handler: wrap(() => renameFocused()) },
      { key: "delete", handler: wrap(() => deleteFocused()) },
      { key: "escape", handler: () => closeOverlays(), preventDefault: false },
    ]);

    const handleResize = () => {
      if (window.innerWidth < 1120) {
        if (get(uiState).rightOpen) {
          uiState.update((state) => ({ ...state, rightOpen: false }));
        }
      } else if (!userClosedRight && !get(uiState).rightOpen) {
        uiState.update((state) => ({ ...state, rightOpen: true }));
      }
    };

    window.addEventListener("resize", handleResize);
    window.addEventListener("unhandledrejection", handleUnhandledRejection);
    window.addEventListener("error", handleRuntimeError);
    handleResize();

    return () => {
      unregister();
      window.removeEventListener("resize", handleResize);
      window.removeEventListener("unhandledrejection", handleUnhandledRejection);
      window.removeEventListener("error", handleRuntimeError);
    };
  });

  $: wordCount =
    $noteBody.trim().length === 0
      ? 0
      : $noteBody.trim().split(/\s+/).filter(Boolean).length;

  $: vaultLabel = $vaultPath ? basename($vaultPath) : "No vault open";

  $: selectionLabel = $focusedPath
    ? $focusedIsFolder
      ? "Folder selected"
      : "Note selected"
    : "Nothing selected";

  $: workspaceTitle = $activeNote
    ? $activeNote.title
    : $vaultPath
      ? "Quiet focus, quick capture."
      : "A calmer space for local-first notes.";

  $: workspaceSubtitle = $activeNote
    ? `${$activeNote.modified} · ${wordCount} words`
    : $vaultPath
      ? "Open a note, press Ctrl+N, or drop a file into the editor."
      : "Open a vault to unlock search, AI sidekicks, and sketchable notes.";
</script>

<div
  class="app-shell"
  style={`--left-width:${$uiState.leftOpen ? leftWidth : 0}px; --right-width:${$uiState.rightOpen ? rightWidth : 0}px;`}
>
  <header class="titlebar">
    <div class="title-cluster" data-tauri-drag-region>
      <div class="brand-mark">
        <PenNib size={18} />
      </div>
      <div class="brand-copy">
        <span class="eyebrow">Offline note studio</span>
        <div class="brand-row">
          <span class="brand-name">Quillpaw</span>
          <span class="vault-chip">{vaultLabel}</span>
        </div>
      </div>
    </div>

    <div class="title-actions">
      <button type="button" class="utility-btn" on:click={() => openSearch("smart")}>
        Search
      </button>
      <button type="button" class="utility-btn" on:click={toggleLeft}>
        {$uiState.leftOpen ? "Library" : "Show library"}
      </button>
      <button
        type="button"
        class="utility-btn"
        on:click={toggleRightPanel}
      >
        {$uiState.rightOpen ? "AI panel" : "Show AI"}
      </button>
      <button type="button" class="utility-btn" on:click={toggleSettings}>
        Settings
      </button>

      <div class="window-buttons">
        <button
          type="button"
          class="window-btn"
          on:click={() => void handleWindow("min")}
          aria-label="Minimize"
        >
          <Minus size={14} />
        </button>
        <button
          type="button"
          class="window-btn"
          on:click={() => void handleWindow("max")}
          aria-label="Maximize"
        >
          <Square size={14} />
        </button>
        <button
          type="button"
          class="window-btn danger"
          on:click={() => void handleWindow("close")}
          aria-label="Close"
        >
          <X size={14} />
        </button>
      </div>
    </div>
  </header>

  {#if $uiState.leftOpen}
    <aside class="left panel-shell">
      <FileTree />
      <div
        class="resizer"
        role="separator"
        aria-orientation="vertical"
        tabindex="-1"
        on:pointerdown={(event) => startResize("left", event)}
      ></div>
    </aside>
  {/if}

  <main>
    <div class="workspace-bar">
      <div class="workspace-copy">
        <span class="workspace-kicker">{selectionLabel}</span>
        <div class="workspace-title">{workspaceTitle}</div>
        <p class="workspace-subtitle">{workspaceSubtitle}</p>
      </div>
      <div class="workspace-actions">
        <button type="button" class="ghost" on:click={() => openCommand()}>
          Commands
        </button>
        <button type="button" class="ghost" on:click={() => openSearch("smart")}>
          Find
        </button>
        <button
          type="button"
          class="primary"
          on:click={() =>
            void runUiTask(
              () => createDrawingQuick(),
              "Could not create a drawing canvas.",
            )}
          disabled={!$vaultPath}
        >
          New sketch
        </button>
      </div>
    </div>

    {#if $uiState.lectureOpen}
      <LectureMode />
    {/if}

    <TabBar />
    <Editor />
  </main>

  {#if $uiState.rightOpen}
    <aside class="right panel-shell">
      <div
        class="resizer"
        role="separator"
        aria-orientation="vertical"
        tabindex="-1"
        on:pointerdown={(event) => startResize("right", event)}
      ></div>
      <AIPanel />
    </aside>
  {/if}

  <footer class="status">
    <div class="status-group">
      <span class="status-pill accent">{vaultLabel}</span>
      <span class="status-pill">{wordCount} words</span>
      {#if $activeNote?.tags?.length}
        <span class="status-pill">{$activeNote.tags.join(", ")}</span>
      {/if}
    </div>
    <div class="status-group muted">
      <span class="status-pill">{$uiState.theme}</span>
      <span class="status-copy">Private by default. Fast by design.</span>
    </div>
  </footer>

  <Settings open={$uiState.settingsOpen} />
  <SearchModal open={$uiState.searchOpen} mode={$uiState.searchMode} />
  <CommandPalette open={$uiState.commandOpen} />
  <DrawingCanvas />

  <ConfirmDialog
    open={deleteDialogOpen}
    message={deleteMessage}
    onConfirm={() => void runUiTask(() => onConfirmDelete(), "Delete failed.")}
    onCancel={onCancelDelete}
  />

  {#if !$vaultPath}
    <div class="onboarding">
      <div class="card">
        <div class="logo"><PenNib size={34} /></div>
        <div class="eyebrow">Local-first workspace</div>
        <h2>Research notes without the noise.</h2>
        <p>
          Capture ideas, search instantly, sketch inline, and keep everything on
          your own machine.
        </p>
        <div class="feature-strip">
          <span>Fast vaults</span>
          <span>Inline sketches</span>
          <span>Private AI tools</span>
        </div>
        <div class="actions">
          <button
            type="button"
            class="ghost"
            on:click={() =>
              void runUiTask(() => openVault(), "Could not open that vault.")
            }
          >
            Open Existing Vault
          </button>
          <button
            type="button"
            class="primary"
            on:click={() =>
              void runUiTask(
                () => createVault(),
                "Could not create that vault.",
              )
            }
          >
            Create New Vault
          </button>
        </div>
      </div>
    </div>
  {/if}

  <div class="toast-host">
    {#each $toasts as toast (toast.id)}
      <div class={`toast ${toast.type}`}>{toast.message}</div>
    {/each}
  </div>
</div>

<style>
  .app-shell {
    display: grid;
    grid-template-columns: var(--left-width) 1fr var(--right-width);
    grid-template-rows: 72px 1fr 40px;
    height: 100%;
    background:
      radial-gradient(circle at top left, rgba(88, 193, 255, 0.12), transparent 32%),
      radial-gradient(circle at top right, rgba(126, 240, 197, 0.08), transparent 22%),
      linear-gradient(180deg, rgba(7, 10, 18, 0.95), rgba(8, 13, 23, 0.98));
  }

  .titlebar {
    grid-column: 1 / 4;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: 0 var(--space-5);
    border-bottom: 1px solid var(--border);
    background: var(--glass);
    backdrop-filter: blur(18px);
  }

  .title-cluster {
    min-width: 0;
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .brand-mark {
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    border-radius: 14px;
    background: linear-gradient(135deg, var(--accent), var(--accent2));
    color: #04111c;
    box-shadow: 0 10px 30px rgba(88, 193, 255, 0.24);
  }

  .brand-copy {
    min-width: 0;
    display: grid;
    gap: 4px;
  }

  .eyebrow {
    color: var(--text-muted);
    font-size: 11px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .brand-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
  }

  .brand-name {
    color: var(--text-primary);
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .vault-chip {
    max-width: 240px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 5px 10px;
    border-radius: 999px;
    background: var(--accent-glow);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    font-size: 12px;
  }

  .title-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .utility-btn,
  .window-btn,
  .workspace-actions button {
    min-height: 38px;
  }

  .utility-btn {
    padding: 0 14px;
    border-radius: 12px;
    background: rgba(17, 27, 43, 0.82);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
  }

  .window-buttons {
    display: flex;
    gap: var(--space-1);
    margin-left: var(--space-2);
  }

  .window-btn {
    width: 34px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    background: rgba(17, 27, 43, 0.82);
    border: 1px solid var(--border-subtle);
  }

  .window-btn.danger {
    color: var(--danger);
    border-color: rgba(255, 139, 145, 0.28);
  }

  .panel-shell {
    position: relative;
    overflow: hidden;
    background: rgba(9, 14, 25, 0.72);
    backdrop-filter: blur(18px);
  }

  .left,
  .right {
    overflow: auto;
  }

  .left {
    border-right: 1px solid var(--border-subtle);
  }

  .right {
    border-left: 1px solid var(--border-subtle);
  }

  .resizer {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 6px;
    background: transparent;
    cursor: col-resize;
    user-select: none;
    transition: background var(--transition), box-shadow var(--transition);
    z-index: 10;
  }

  .resizer:hover {
    background: var(--accent-subtle);
    box-shadow: 0 0 0 1px rgba(88, 193, 255, 0.2);
  }

  .left .resizer {
    right: 0;
  }

  .right .resizer {
    left: 0;
  }

  main {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
    background: transparent;
  }

  .workspace-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-5) var(--space-3);
    border-bottom: 1px solid var(--border-subtle);
    background: linear-gradient(
      180deg,
      rgba(16, 24, 38, 0.78),
      rgba(11, 17, 29, 0.34)
    );
  }

  .workspace-copy {
    min-width: 0;
    display: grid;
    gap: 6px;
  }

  .workspace-kicker {
    color: var(--accent-bright);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .workspace-title {
    color: var(--text-primary);
    font-size: clamp(20px, 2vw, 28px);
    font-weight: 700;
    letter-spacing: -0.03em;
  }

  .workspace-subtitle {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .workspace-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .workspace-actions .ghost {
    background: rgba(15, 24, 39, 0.76);
    border-color: var(--border-subtle);
    color: var(--text-secondary);
    padding: 0 14px;
    border-radius: 12px;
  }

  .workspace-actions .primary {
    padding: 0 16px;
    border-radius: 12px;
    background: linear-gradient(135deg, var(--accent), var(--accent2));
    border-color: transparent;
    color: #04111c;
    font-weight: 700;
  }

  .status {
    grid-column: 1 / 4;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: 0 var(--space-5);
    border-top: 1px solid var(--border);
    background: var(--glass);
    backdrop-filter: blur(18px);
  }

  .status-group {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
  }

  .status-group.muted {
    color: var(--text-secondary);
  }

  .status-pill {
    padding: 5px 10px;
    border-radius: 999px;
    border: 1px solid var(--border-subtle);
    background: rgba(14, 22, 36, 0.8);
    color: inherit;
    white-space: nowrap;
  }

  .status-pill.accent {
    border-color: rgba(88, 193, 255, 0.22);
    color: var(--accent-bright);
  }

  .status-copy {
    color: var(--text-muted);
    white-space: nowrap;
  }

  .onboarding {
    position: fixed;
    inset: 0;
    background:
      radial-gradient(circle at top, rgba(88, 193, 255, 0.12), transparent 28%),
      rgba(3, 7, 14, 0.72);
    display: grid;
    place-items: center;
    z-index: 30;
    backdrop-filter: blur(20px);
  }

  .onboarding .card {
    width: min(580px, calc(100vw - 40px));
    padding: clamp(28px, 5vw, 42px);
    border-radius: 28px;
    border: 1px solid rgba(120, 158, 212, 0.18);
    background:
      linear-gradient(180deg, rgba(20, 29, 45, 0.96), rgba(12, 18, 30, 0.98));
    box-shadow:
      0 24px 80px rgba(0, 0, 0, 0.45),
      inset 0 1px 0 rgba(255, 255, 255, 0.04);
    text-align: center;
  }

  .onboarding .logo {
    width: 62px;
    height: 62px;
    display: grid;
    place-items: center;
    margin: 0 auto var(--space-4);
    border-radius: 20px;
    background: linear-gradient(135deg, var(--accent), var(--accent2));
    color: #04111c;
    box-shadow: 0 12px 40px rgba(88, 193, 255, 0.24);
  }

  .onboarding h2 {
    margin: 12px 0 10px;
    font-size: clamp(28px, 4vw, 40px);
    line-height: 1.04;
    letter-spacing: -0.04em;
  }

  .onboarding p {
    margin: 0 auto var(--space-5);
    max-width: 460px;
    color: var(--text-secondary);
    font-size: 15px;
    line-height: 1.7;
  }

  .feature-strip {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--space-2);
    margin-bottom: var(--space-5);
  }

  .feature-strip span {
    padding: 7px 12px;
    border-radius: 999px;
    border: 1px solid var(--border-subtle);
    background: rgba(14, 22, 36, 0.8);
    color: var(--text-secondary);
    font-size: 12px;
  }

  .actions {
    display: flex;
    gap: var(--space-3);
    justify-content: center;
    flex-wrap: wrap;
  }

  .actions .primary,
  .actions .ghost {
    min-height: 46px;
    padding: 0 18px;
    border-radius: 14px;
  }

  .actions .primary {
    background: linear-gradient(135deg, var(--accent), var(--accent2));
    border-color: transparent;
    color: #04111c;
    font-weight: 700;
  }

  .actions .ghost {
    background: rgba(14, 22, 36, 0.78);
    border-color: var(--border);
    color: var(--text-primary);
  }

  .toast-host {
    position: fixed;
    right: var(--space-5);
    bottom: calc(var(--space-5) + 40px);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    z-index: 50;
  }

  .toast {
    max-width: 340px;
    padding: 12px 14px;
    border-radius: 16px;
    background: rgba(13, 20, 33, 0.94);
    border: 1px solid rgba(88, 193, 255, 0.22);
    color: var(--text-primary);
    box-shadow: var(--shadow-soft);
  }

  .toast.error {
    border-color: rgba(255, 139, 145, 0.28);
  }

  @media (max-width: 1120px) {
    .titlebar {
      padding: 0 var(--space-4);
    }

    .title-actions {
      flex-wrap: wrap;
      justify-content: flex-end;
    }

    .workspace-bar {
      padding: var(--space-4);
      flex-direction: column;
      align-items: flex-start;
    }

    .status {
      padding: 0 var(--space-4);
    }
  }

  @media (max-width: 760px) {
    .app-shell {
      grid-template-columns: 1fr;
      grid-template-rows: auto 1fr 44px;
    }

    .titlebar,
    .status {
      grid-column: 1;
    }

    .left,
    .right {
      display: none;
    }

    .brand-row {
      flex-wrap: wrap;
    }

    .title-actions {
      width: 100%;
      justify-content: flex-start;
    }

    .window-buttons {
      margin-left: 0;
    }

    .workspace-actions {
      width: 100%;
    }

    .workspace-actions button {
      flex: 1 1 150px;
    }

    .status {
      flex-direction: column;
      justify-content: center;
      padding: var(--space-2) var(--space-4);
    }

    .status-group {
      width: 100%;
      justify-content: center;
      flex-wrap: wrap;
    }

    .toast-host {
      left: var(--space-4);
      right: var(--space-4);
      bottom: calc(var(--space-4) + 48px);
    }

    .toast {
      max-width: none;
    }
  }
</style>

