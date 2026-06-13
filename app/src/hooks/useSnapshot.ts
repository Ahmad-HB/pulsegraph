import { useEffect, useState, useCallback, useRef } from "react";
import { getSnapshot, refresh } from "../api";
import type { Snapshot, Metric } from "../types";

export function useSnapshot(metric: Metric, project: string | null, model: string | null) {
  const [snap, setSnap] = useState<Snapshot | null>(null);
  const argsRef = useRef({ metric, project, model });
  argsRef.current = { metric, project, model };

  const load = useCallback(async () => {
    const { metric, project, model } = argsRef.current;
    setSnap(await getSnapshot(metric, project, model));
  }, []);

  // Mount-only: refresh from disk + reload, then every 60s.
  useEffect(() => {
    let alive = true;
    const tick = async () => { await refresh(); if (alive) await load(); };
    tick();
    const id = setInterval(tick, 60_000);
    return () => { alive = false; clearInterval(id); };
  }, [load]);

  // Re-aggregate in memory when metric/filter changes (no disk).
  useEffect(() => { load(); }, [metric, project, model, load]);

  return snap;
}
