import { useEffect, useMemo, useState } from "react";
import { useSnapshot } from "./hooks/useSnapshot";
import { useSettings } from "./hooks/useSettings";
import { themeToVars } from "./lib/theme";
import type { Metric } from "./types";
import { Header } from "./components/Header";
import { Filters } from "./components/Filters";
import { Heatmap } from "./components/Heatmap";
import { UsageCard } from "./components/UsageCard";
import { StreaksCard } from "./components/StreaksCard";
import { ProjectBars } from "./components/ProjectBars";
import { Footer } from "./components/Footer";
import { Preferences } from "./components/Preferences";
import { getAvatar, pickAvatar } from "./api";
import "./App.css";

function todayKey(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

export default function App() {
  const { settings, update } = useSettings();
  const [project, setProject] = useState<string | null>(null);
  const [model, setModel] = useState<string | null>(null);
  const [view, setView] = useState<"popover" | "prefs">("popover");
  const metric = settings.metric;
  const setMetric = (m: Metric) => update({ metric: m });

  const [avatar, setAvatarState] = useState<string | null>(null);
  useEffect(() => { getAvatar().then(setAvatarState).catch(() => {}); }, []);
  const chooseAvatar = async () => {
    try {
      const url = await pickAvatar();
      if (url) setAvatarState(url);
    } catch (e) {
      console.error("avatar pick failed", e);
    }
  };

  const snap = useSnapshot(metric, project, model);

  // Apply theme as CSS variables on the root.
  useEffect(() => {
    const vars = themeToVars(settings.theme);
    for (const [k, v] of Object.entries(vars)) document.documentElement.style.setProperty(k, v);
  }, [settings.theme]);

  const fmt = useMemo(() => {
    const compact = (n: number) =>
      n >= 1e9 ? (n / 1e9).toFixed(2) + "B"
      : n >= 1e6 ? (n / 1e6).toFixed(2) + "M"
      : n >= 1e3 ? (n / 1e3).toFixed(1) + "k"
      : String(Math.round(n));
    return (v: number) => (metric === "cost" ? `$${v.toFixed(2)}` : compact(v));
  }, [metric]);

  if (view === "prefs") {
    return <Preferences theme={settings.theme} setTheme={(t) => update({ theme: t })} onBack={() => setView("popover")} unreadable={snap?.unreadable_lines ?? 0} />;
  }

  const today = snap?.days.find((d) => d.date === todayKey())?.value ?? 0;

  return (
    <div className="pop">
      <Header today={snap ? fmt(today) : "…"} avatar={avatar} onAvatarClick={chooseAvatar} />
      <Filters metric={metric} setMetric={setMetric} projects={snap?.projects ?? []} models={snap?.models ?? []} project={project} model={model} setProject={setProject} setModel={setModel} />
      <div className="pbody">
        {snap && snap.days.length > 0 ? (
          <>
            <Heatmap days={snap.days} />
            <div className="cards">
              <UsageCard snap={snap} fmt={fmt} />
              <StreaksCard snap={snap} />
            </div>
            <ProjectBars items={snap.projects_today} fmt={fmt} />
          </>
        ) : (
          <p className="empty">No Claude Code usage found yet.</p>
        )}
      </div>
      <Footer generatedAt={snap?.generated_at ?? 0} onSettings={() => setView("prefs")} />
    </div>
  );
}
