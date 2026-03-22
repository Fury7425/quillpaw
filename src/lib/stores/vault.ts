import { get, writable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";

import type { FileNode } from "$lib/types";
import { tauriInvoke } from "$lib/utils/tauri_bridge";
import { pushToast } from "$lib/stores/ui";
import { setLastVaultPath } from "$lib/stores/preferences";

export const vaultPath = writable<string>("");
export const fileTree = writable<FileNode[]>([]);
export const focusedPath = writable<string | null>(null);
export const focusedIsFolder = writable<boolean>(false);
export const renameRequest = writable<string | null>(null);

let unlistenVault: (() => void) | null = null;
let refreshTimer: ReturnType<typeof setTimeout> | null = null;

const formatError = (err: unknown) =>
  err instanceof Error ? err.message : String(err);

const isMissingVaultError = (message: string) =>
  message.includes("no longer exists") || message.includes("not a folder");

const persistLastVaultPath = async (path: string | null) => {
  try {
    await setLastVaultPath(path);
  } catch (err) {
    pushToast(formatError(err) || "Could not remember that vault.");
  }
};

const rebuildKeywordIndex = (path: string) => {
  void tauriInvoke("build_search_index", { vaultPath: path }).catch(() => null);
};

const activateVault = async (path: string) => {
  vaultPath.set(path);
  await refreshTree();
  rebuildKeywordIndex(path);
  try {
    await startWatcherListener();
  } catch (err) {
    pushToast(formatError(err) || "Live vault sync is unavailable right now.");
  }
};

const clearVaultState = async (forgetLastVault = false) => {
  vaultPath.set("");
  fileTree.set([]);
  focusedPath.set(null);
  focusedIsFolder.set(false);
  renameRequest.set(null);
  if (forgetLastVault) {
    await persistLastVaultPath(null);
  }
};

export async function openVault(): Promise<void> {
  try {
    const path = await tauriInvoke<string>("open_vault");
    await activateVault(path);
    await persistLastVaultPath(path);
  } catch (err) {
    pushToast(formatError(err));
  }
}

export async function restoreVault(path: string): Promise<boolean> {
  try {
    const restoredPath = await tauriInvoke<string>("restore_vault", {
      vaultPath: path,
    });
    await activateVault(restoredPath);
    await persistLastVaultPath(restoredPath);
    return true;
  } catch (err) {
    const message = formatError(err) || "Could not restore the last vault.";
    await clearVaultState(isMissingVaultError(message));
    pushToast(message);
    return false;
  }
}

export async function createVault(): Promise<void> {
  try {
    const path = await tauriInvoke<string>("open_vault");
    await tauriInvoke("create_note", {
      vaultPath: path,
      folder: "",
      title: "Welcome",
    });
    await activateVault(path);
    await persistLastVaultPath(path);
    pushToast("Vault created! Start writing.", "info");
  } catch (err) {
    pushToast(formatError(err));
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
    pushToast(formatError(err));
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
    pushToast(formatError(err));
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
    pushToast(formatError(err));
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
    pushToast(formatError(err));
  }
}

export async function deleteItem(path: string): Promise<void> {
  try {
    await tauriInvoke("delete_item", { path });
    await refreshTree();
  } catch (err) {
    pushToast(formatError(err));
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

async function startWatcherListener(): Promise<void> {
  if (unlistenVault) {
    return;
  }

  unlistenVault = await listen<string[]>("vault-changed", async () => {
    if (refreshTimer) clearTimeout(refreshTimer);
    refreshTimer = setTimeout(async () => {
      const path = get(vaultPath);
      if (!path) return;
      await refreshTree();
      rebuildKeywordIndex(path);
    }, 300);
  });
}
