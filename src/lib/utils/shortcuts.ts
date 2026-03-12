export type ShortcutHandler = (event: KeyboardEvent) => void;
export const registerShortcut = (handler: ShortcutHandler) => window.addEventListener('keydown', handler);
