import { get, writable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { load } from "@tauri-apps/plugin-store";

import type { FileNode } from "$lib/types";
import { tauriInvoke } from "$lib/utils/tauri_bridge";
import { pushToast } from "$lib/stores/ui";

export const vaultPath = writable<string>("");
export const fileTree = writable<FileNode[]>([]);
export const focusedPath = writable<string | null>(null);
export const focusedIsFolder = writable<boolean>(false);

let unlistenVault: (() => void) | null = null;
let refreshTimer: ReturnType<typeof setTimeout> | null = null;

export async function openVault(): Promise<void> {
  try {
    const path = await tauriInvoke<string>("open_vault");
    vaultPath.set(path);
    await refreshTree();
    await tauriInvoke("build_search_index", { vaultPath: path });
    tauriInvoke("build_embeddings", { vaultPath: path }).catch(() => null);
    const store = await load(`${path}/.quillpaw/config.json`);
    await store.set("vaultPath", path);
    await store.save();
    await startWatcher();
  } catch (err) {
    pushToast(err instanceof Error ? err.message : String(err));
  }
}

export async function refreshTree(): Promise<void> {
  const path = get(vaultPath);
  if (!path) return;
  try {
    const tree = await tauriInvoke<FileNode[]>("get_file_tree", {
      vaultPath: path,
    });
    fileTree.set(tree);
  } catch (err) {
    pushToast(err instanceof Error ? err.message : String(err));
  }
}

export async function createNote(
  folder: string,
  title: string,
): Promise<string | null> {
  const path = get(vaultPath);
  if (!path) return null;
  try {
    const created = await tauriInvoke<string>("create_note", {
      vaultPath: path,
      folder,
      title,
    });
    await refreshTree();
    return created;
  } catch (err) {
    pushToast(err instanceof Error ? err.message : String(err));
    return null;
  }
}

export async function createFolder(relativePath: string): Promise<void> {
  const path = get(vaultPath);
  if (!path) return;
  try {
    await tauriInvoke("create_folder", {
      vaultPath: path,
      folderName: relativePath,
    });
    await refreshTree();
  } catch (err) {
    pushToast(err instanceof Error ? err.message : String(err));
  }
}

export async function renameItem(
  oldPath: string,
  newName: string,
): Promise<void> {
  try {
    await tauriInvoke("rename_item", { oldPath, newName });
    await refreshTree();
  } catch (err) {
    pushToast(err instanceof Error ? err.message : String(err));
  }
}

export async function deleteItem(path: string): Promise<void> {
  try {
    await tauriInvoke("delete_item", { path });
    await refreshTree();
  } catch (err) {
    pushToast(err instanceof Error ? err.message : String(err));
  }
}

export function setFocused(path: string | null, isFolder: boolean): void {
  focusedPath.set(path);
  focusedIsFolder.set(isFolder);
}

export function toVaultRelative(path: string, vault: string): string {
  if (path.startsWith(vault)) {
    return path.replace(vault, "").replace(/^[/\\]/, "");
  }
  return path;
}

async function startWatcher(): Promise<void> {
  if (unlistenVault) {
    unlistenVault();
    unlistenVault = null;
  }
  unlistenVault = await listen<string[]>("vault-changed", async () => {
    if (refreshTimer) clearTimeout(refreshTimer);
    refreshTimer = setTimeout(async () => {
      await refreshTree();
      const path = get(vaultPath);
      if (path) {
        await tauriInvoke("build_search_index", { vaultPath: path });
        tauriInvoke("build_embeddings", { vaultPath: path }).catch(() => null);
      }
    }, 400);
  });
}
