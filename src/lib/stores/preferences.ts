export type ThemePreference = "Dark Warm" | "Dark Cool" | "Light Parchment";

type StoreLike = {
  get<T>(key: string): Promise<T | null>;
  set(key: string, value: unknown): Promise<void>;
  save(): Promise<void>;
};

type AppPreferences = {
  lastVaultPath: string | null;
  theme: ThemePreference;
};

const STORE_PATH = "app-preferences.json";
const DEFAULTS: AppPreferences = {
  lastVaultPath: null,
  theme: "Dark Warm",
};

let storePromise: Promise<StoreLike> | null = null;

const createBrowserFallbackStore = (): StoreLike => ({
  async get<T>(key: string): Promise<T | null> {
    if (typeof window === "undefined") {
      return (DEFAULTS[key as keyof AppPreferences] as T) ?? null;
    }
    const raw = window.localStorage.getItem(`${STORE_PATH}:${key}`);
    if (raw === null) {
      return (DEFAULTS[key as keyof AppPreferences] as T) ?? null;
    }
    return JSON.parse(raw) as T;
  },
  async set(key: string, value: unknown): Promise<void> {
    if (typeof window === "undefined") return;
    window.localStorage.setItem(`${STORE_PATH}:${key}`, JSON.stringify(value));
  },
  async save(): Promise<void> {},
});

async function getStore(): Promise<StoreLike> {
  if (!storePromise) {
    storePromise = (async () => {
      if (typeof window === "undefined") {
        return createBrowserFallbackStore();
      }

      try {
        const { load } = await import("@tauri-apps/plugin-store");
        return await load(STORE_PATH, {
          autoSave: 100,
          defaults: DEFAULTS,
        });
      } catch {
        return createBrowserFallbackStore();
      }
    })();
  }
  return storePromise;
}

export async function getSavedTheme(): Promise<ThemePreference> {
  const store = await getStore();
  const theme = await store.get<ThemePreference>("theme");
  if (
    theme === "Dark Warm" ||
    theme === "Dark Cool" ||
    theme === "Light Parchment"
  ) {
    return theme;
  }
  return DEFAULTS.theme;
}

export async function setSavedTheme(theme: ThemePreference): Promise<void> {
  const store = await getStore();
  await store.set("theme", theme);
  await store.save();
}

export async function getLastVaultPath(): Promise<string | null> {
  const store = await getStore();
  return (await store.get<string | null>("lastVaultPath")) ?? null;
}

export async function setLastVaultPath(path: string | null): Promise<void> {
  const store = await getStore();
  await store.set("lastVaultPath", path);
  await store.save();
}
