import { useEffect, useState, useCallback } from "react";
import { load, type Store } from "@tauri-apps/plugin-store";
import { defaultTheme, type Theme } from "../lib/theme";
import type { Metric } from "../types";

export type Settings = { metric: Metric; theme: Theme };
const DEFAULTS: Settings = { metric: "cost", theme: defaultTheme };

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULTS);
  const [store, setStore] = useState<Store | null>(null);

  useEffect(() => {
    let alive = true;
    (async () => {
      const s = await load("settings.json", { autoSave: true, defaults: {} });
      const saved = (await s.get<Settings>("settings")) ?? DEFAULTS;
      if (alive) { setStore(s); setSettings({ ...DEFAULTS, ...saved }); }
    })();
    return () => { alive = false; };
  }, []);

  const update = useCallback((patch: Partial<Settings>) => {
    setSettings((prev) => {
      const next = { ...prev, ...patch };
      store?.set("settings", next);
      return next;
    });
  }, [store]);

  return { settings, update };
}
