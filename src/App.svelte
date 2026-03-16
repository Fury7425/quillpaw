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

  import {
    openVault,
    vaultPath,
    focusedPath,
    focusedIsFolder,
    toVaultRelative,
    createNote,
    renameItem,
    deleteItem,
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
  } from "$lib/stores/ui";
  import { registerShortcuts } from "$lib/utils/shortcuts";
  import { tauriInvoke } from "$lib/utils/tauri_bridge";

  let appWindow = getCurrentWindow();
  let leftWidth = 280;
  let rightWidth = 320;

  const handleWindow = (action: "min" | "max" | "close") => {
    if (action === "min") appWindow.minimize();
    if (action === "max") appWindow.toggleMaximize();
    if (action === "close") appWindow.close();
  };

  const shouldIgnoreShortcut = (event: KeyboardEvent) => {
    const target = event.target as HTMLElement | null;
    if (!target) return false;
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

  const startResize = (side: "left" | "right", event: PointerEvent) => {
    event.preventDefault();
    const onMove = (moveEvent: PointerEvent) => {
      if (side === "left") {
        leftWidth = Math.min(Math.max(200, moveEvent.clientX), 480);
      } else {
        const width = window.innerWidth - moveEvent.clientX;
        rightWidth = Math.min(Math.max(240, width), 520);
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

  const renameFocused = async () => {
    const path = get(focusedPath);
    if (!path) return;
    const name = window.prompt("New name");
    if (!name) return;
    await renameItem(path, name);
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

  const deleteFocused = async () => {
    const path = get(focusedPath);
    if (!path) return;
    const confirmed = window.confirm("Delete selected item?");
    if (!confirmed) return;
    await deleteItem(path);
  };

  onMount(() => {
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
      { key: "/", ctrl: true, handler: () => toggleRight() },
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
      { key: "d", ctrl: true, handler: wrap(() => createDrawingQuick()) },
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
    return () => {
      unregister();
    };
  });

  $: wordCount =
    $noteBody.trim().length === 0
      ? 0
      : $noteBody.trim().split(/\s+/).filter(Boolean).length;
</script>

<div
  class="app-shell"
  style={`--left-width:${$uiState.leftOpen ? leftWidth : 0}px; --right-width:${$uiState.rightOpen ? rightWidth : 0}px;`}
>
  <header class="titlebar" data-tauri-drag-region>
    <div class="title" data-tauri-drag-region>
      <PenNib size={16} />
      <span>Quillpaw</span>
    </div>
    <div class="window-buttons">
      <button
        class="window-btn"
        on:click={() => handleWindow("min")}
        aria-label="Minimize"
      >
        <Minus size={14} />
      </button>
      <button
        class="window-btn"
        on:click={() => handleWindow("max")}
        aria-label="Maximize"
      >
        <Square size={14} />
      </button>
      <button
        class="window-btn danger"
        on:click={() => handleWindow("close")}
        aria-label="Close"
      >
        <X size={14} />
      </button>
    </div>
  </header>

  {#if $uiState.leftOpen}
    <aside class="left">
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
    {#if $uiState.lectureOpen}
      <LectureMode />
    {/if}
    <TabBar />
    <Editor />
  </main>

  {#if $uiState.rightOpen}
    <aside class="right">
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
    <div class="status-left">
      <span class="pill">Words: {wordCount}</span>
      {#if $activeNote?.tags?.length}
        <span class="pill">Tags: {$activeNote.tags.join(", ")}</span>
      {/if}
    </div>
    <div class="status-right">Your notes. Your machine. Your den.</div>
  </footer>

  <Settings open={$uiState.settingsOpen} />
  <SearchModal open={$uiState.searchOpen} mode={$uiState.searchMode} />
  <CommandPalette open={$uiState.commandOpen} />
  <DrawingCanvas />

  {#if !$vaultPath}
    <div class="onboarding">
      <div class="card">
        <div class="logo"><PenNib size={32} /></div>
        <h2>Quillpaw</h2>
        <p>Your notes. Your machine. Your den.</p>
        <div class="actions">
          <button class="primary" on:click={openVault}>Open Vault</button>
          <button on:click={openVault}>Create Vault</button>
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
    grid-template-rows: 36px 1fr 28px;
    height: 100%;
    background: var(--bg-base);
  }
  .titlebar {
    grid-column: 1 / 4;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-3);
  }
  .title {
    display: flex;
    gap: var(--space-2);
    align-items: center;
    font-family: var(--font-ui);
    color: var(--text-primary);
  }
  .window-buttons {
    display: flex;
    gap: var(--space-1);
  }
  .window-btn {
    width: 28px;
    height: 24px;
    display: grid;
    place-items: center;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
  }
  .window-btn.danger {
    border-color: var(--danger);
    color: var(--danger);
  }
  .left,
  .right {
    background: var(--bg-panel);
    overflow: auto;
    position: relative;
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
    width: 4px;
    background: transparent;
    cursor: col-resize;
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
  }
  .status {
    grid-column: 1 / 4;
    background: var(--bg-panel);
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-4);
    color: var(--text-secondary);
  }
  .pill {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 2px 8px;
    margin-right: var(--space-2);
  }
  .onboarding {
    position: fixed;
    inset: 0;
    background: var(--overlay);
    display: grid;
    place-items: center;
    z-index: 30;
  }
  .onboarding .card {
    background: var(--bg-elevated);
    padding: var(--space-8);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    box-shadow: var(--shadow-strong);
    text-align: center;
    width: 360px;
  }
  .onboarding .logo {
    width: 52px;
    height: 52px;
    display: grid;
    place-items: center;
    margin: 0 auto var(--space-3);
    border-radius: 50%;
    background: var(--accent-glow);
    border: 1px solid var(--accent);
    color: var(--accent);
  }
  .onboarding h2 {
    margin: 0 0 var(--space-2);
    font-size: 20px;
  }
  .onboarding p {
    margin: 0 0 var(--space-4);
    color: var(--text-secondary);
  }
  .actions {
    display: flex;
    gap: var(--space-3);
    justify-content: center;
  }
  .actions .primary {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--bg-base);
  }
  .toast-host {
    position: fixed;
    right: var(--space-4);
    bottom: calc(var(--space-4) + 28px);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    z-index: 40;
  }
  .toast {
    background: var(--bg-elevated);
    border: 1px solid var(--accent);
    color: var(--text-primary);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
  }
</style>
