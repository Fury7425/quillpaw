import { get, writable } from "svelte/store";

import type { NoteContent } from "$lib/types";
import { tauriInvoke } from "$lib/utils/tauri_bridge";
import { pushToast } from "$lib/stores/ui";

export type NoteTab = {
  path: string;
  title: string;
  dirty: boolean;
};

export const openTabs = writable<NoteTab[]>([]);
export const activePath = writable<string | null>(null);
export const activeNote = writable<NoteContent | null>(null);
export const noteBody = writable<string>("");
export const insertRequest = writable<string | null>(null);

const noteCache = new Map<string, NoteContent>();

export async function openNote(path: string): Promise<void> {
  try {
    let note = noteCache.get(path);
    if (!note) {
      note = await tauriInvoke<NoteContent>("read_note", { path });
      noteCache.set(path, note);
    }
    activeNote.set(note);
    noteBody.set(note.body);
    activePath.set(path);
    openTabs.update((tabs) => {
      const exists = tabs.find((tab) => tab.path === path);
      if (exists) return tabs;
      return [...tabs, { path, title: note.title, dirty: false }];
    });
  } catch (err) {
    pushToast(err instanceof Error ? err.message : String(err));
  }
}

export function updateBody(body: string): void {
  noteBody.set(body);
  const path = get(activePath);
  if (!path) return;
  openTabs.update((tabs) =>
    tabs.map((tab) => (tab.path === path ? { ...tab, dirty: true } : tab)),
  );
}

export async function saveActiveNote(): Promise<void> {
  const path = get(activePath);
  const body = get(noteBody);
  if (!path) return;
  try {
    await tauriInvoke("save_note", { path, content: body });
    openTabs.update((tabs) =>
      tabs.map((tab) => (tab.path === path ? { ...tab, dirty: false } : tab)),
    );
    const cached = noteCache.get(path);
    if (cached) {
      noteCache.set(path, { ...cached, body });
      activeNote.set({ ...cached, body });
    }
  } catch (err) {
    pushToast(err instanceof Error ? err.message : String(err));
  }
}

export function closeTab(path: string): void {
  openTabs.update((tabs) => tabs.filter((tab) => tab.path !== path));
  if (get(activePath) === path) {
    const remaining = get(openTabs);
    const next = remaining[remaining.length - 1];
    if (next) {
      openNote(next.path);
    } else {
      activePath.set(null);
      activeNote.set(null);
      noteBody.set("");
    }
  }
}

export function requestInsert(text: string): void {
  insertRequest.set(text);
  setTimeout(() => insertRequest.set(null), 0);
}
