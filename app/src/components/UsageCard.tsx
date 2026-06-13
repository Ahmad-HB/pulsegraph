import type { Snapshot } from "../types";

export function UsageCard({ snap, fmt }: { snap: Snapshot; fmt: (v: number) => string }) {
  return (
    <div className="card">
      <h4>Usage · last year</h4>
      <div className="metrics">
        <div className="metric"><div className="n">{fmt(snap.total)}</div><div className="k">Total</div></div>
        <div className="metric">
          <div className="n">{snap.best_day ? fmt(snap.best_day.value) : "—"}</div>
          <div className="k">Best day</div>
          <div className="d">{snap.best_day?.date ?? ""}</div>
        </div>
        <div className="metric"><div className="n">{fmt(snap.avg_per_active_day)}</div><div className="k">Avg / day</div></div>
      </div>
    </div>
  );
}
