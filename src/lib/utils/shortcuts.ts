export type Shortcut = {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  handler: (event: KeyboardEvent) => void;
  preventDefault?: boolean;
};

export function matchShortcut(event: KeyboardEvent, shortcut: Shortcut): boolean {
  const key = event.key.toLowerCase();
  if (key !== shortcut.key.toLowerCase()) return false;
  if (Boolean(shortcut.ctrl) !== (event.ctrlKey || event.metaKey)) return false;
  if (Boolean(shortcut.shift) !== event.shiftKey) return false;
  if (Boolean(shortcut.alt) !== event.altKey) return false;
  return true;
}

export function registerShortcuts(shortcuts: Shortcut[]): () => void {
  const handler = (event: KeyboardEvent) => {
    for (const shortcut of shortcuts) {
      if (matchShortcut(event, shortcut)) {
        if (shortcut.preventDefault !== false) {
          event.preventDefault();
        }
        shortcut.handler(event);
        break;
      }
    }
  };
  window.addEventListener('keydown', handler);
  return () => window.removeEventListener('keydown', handler);
}
