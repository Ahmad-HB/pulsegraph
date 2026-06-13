import type { Snapshot } from "../types";

export function StreaksCard({ snap }: { snap: Snapshot }) {
  return (
    <div className="card">
      <h4>Streaks</h4>
      <div className="metrics">
        <div className="metric">
          <div className="n flame">{snap.current_streak}🔥</div><div className="k">Current</div>
          <div className="d">{snap.current_range ? `${snap.current_range[0]} → ${snap.current_range[1]}` : ""}</div>
        </div>
        <div className="metric">
          <div className="n flame">{snap.longest_streak}</div><div className="k">Longest</div>
          <div className="d">{snap.longest_range ? `${snap.longest_range[0]} → ${snap.longest_range[1]}` : ""}</div>
        </div>
      </div>
    </div>
  );
}
