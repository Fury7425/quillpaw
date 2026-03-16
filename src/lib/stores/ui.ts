import { writable } from "svelte/store";

export type Toast = {
  id: string;
  message: string;
  type: "error" | "info";
};

export type UiState = {
  leftOpen: boolean;
  rightOpen: boolean;
  searchOpen: boolean;
  commandOpen: boolean;
  settingsOpen: boolean;
  lectureOpen: boolean;
  searchMode: "keyword" | "semantic" | "smart";
  drawingOpen: boolean;
  drawingFile: string | null;
  theme: "Dark Warm" | "Dark Cool" | "Light Parchment";
};

export const uiState = writable<UiState>({
  leftOpen: true,
  rightOpen: true,
  searchOpen: false,
  commandOpen: false,
  settingsOpen: false,
  lectureOpen: false,
  searchMode: "keyword",
  drawingOpen: false,
  drawingFile: null,
  theme: "Dark Warm",
});

export const toasts = writable<Toast[]>([]);

export function pushToast(
  message: string,
  type: "error" | "info" = "error",
): void {
  const id = crypto.randomUUID();
  toasts.update((list) => [...list, { id, message, type }]);
  setTimeout(() => {
    toasts.update((list) => list.filter((toast) => toast.id !== id));
  }, 4000);
}

export function toggleLeft(): void {
  uiState.update((state) => ({ ...state, leftOpen: !state.leftOpen }));
}

export function toggleRight(): void {
  uiState.update((state) => ({ ...state, rightOpen: !state.rightOpen }));
}

export function openSearch(mode: UiState["searchMode"]): void {
  uiState.update((state) => ({ ...state, searchOpen: true, searchMode: mode }));
}

export function closeSearch(): void {
  uiState.update((state) => ({ ...state, searchOpen: false }));
}

export function openCommand(): void {
  uiState.update((state) => ({ ...state, commandOpen: true }));
}

export function closeCommand(): void {
  uiState.update((state) => ({ ...state, commandOpen: false }));
}

export function toggleSettings(): void {
  uiState.update((state) => ({ ...state, settingsOpen: !state.settingsOpen }));
}

export function closeOverlays(): void {
  uiState.update((state) => ({
    ...state,
    searchOpen: false,
    commandOpen: false,
    settingsOpen: false,
  }));
}

export function toggleLecture(): void {
  uiState.update((state) => ({ ...state, lectureOpen: !state.lectureOpen }));
}

export function openDrawing(filename: string): void {
  uiState.update((state) => ({
    ...state,
    drawingOpen: true,
    drawingFile: filename,
  }));
}

export function closeDrawing(): void {
  uiState.update((state) => ({
    ...state,
    drawingOpen: false,
    drawingFile: null,
  }));
}
