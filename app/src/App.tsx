import { useEffect, useState, useCallback, useRef } from "react";
import { getSnapshot, refresh } from "./api";
import type { Snapshot, Metric } from "./types";
import { Heatmap } from "./components/Heatmap";
import "./App.css";

const METRICS: Metric[] = ["cost", "billable", "output", "raw"];

function fmt(metric: Metric, v: number): string {
  return metric === "cost" ? `$${v.toFixed(2)}` : Math.round(v).toLocaleString();
}

export default function App() {
  const [metric, setMetric] = useState<Metric>("cost");
  const [snap, setSnap] = useState<Snapshot | null>(null);

  const metricRef = useRef<Metric>(metric);
  metricRef.current = metric;

  const load = useCallback(async (m: Metric) => {
    setSnap(await getSnapshot(m, null, null));
  }, []);

  // Mount-only: prime + refresh-from-disk every 60s, reload the current metric.
  useEffect(() => {
    let alive = true;
    const tick = async () => {
      await refresh();
      if (alive) await load(metricRef.current);
    };
    tick();
    const id = setInterval(tick, 60_000);
    return () => { alive = false; clearInterval(id); };
  }, [load]);

  // Metric toggle: re-aggregate in memory only (no disk, no timer churn).
  useEffect(() => { load(metric); }, [metric, load]);

  const today = snap?.days.find((d) => d.date === todayKey())?.value ?? 0;

  return (
    <div className="pop">
      <div className="ph">
        <div className="t">PulseGraph</div>
        <div className="big">{snap ? fmt(metric, today) : "…"}</div>
      </div>
      <div className="toggles">
        {METRICS.map((m) => (
          <button
            key={m}
            className={`chip ${m === metric ? "on" : ""}`}
            onClick={() => setMetric(m)}
          >
            {m}
          </button>
        ))}
      </div>
      <div className="pbody">
        {snap && snap.days.length > 0 ? (
          <Heatmap days={snap.days} />
        ) : (
          <p className="empty">No Claude Code usage found yet.</p>
        )}
        {snap && (
          <div className="stats">
            Total {fmt(metric, snap.total)} · streak {snap.current_streak}d ·
            active {snap.active_days}d
          </div>
        )}
      </div>
    </div>
  );
}

function todayKey(): string {
  const d = new Date();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${d.getFullYear()}-${m}-${day}`;
}
