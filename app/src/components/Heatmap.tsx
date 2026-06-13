import { buildYearGrid, level, localKey } from "../lib/heatmap";
import type { DayValue } from "../types";

const RAMP = ["#161b22", "#0e4429", "#006d32", "#26a641", "#39d353"];

export function Heatmap({ days }: { days: DayValue[] }) {
  const today = new Date();
  const grid = buildYearGrid(today);
  const byDate = new Map(days.map((d) => [d.date, d.value]));
  const max = days.reduce((m, d) => Math.max(m, d.value), 0);

  return (
    <div className="hm">
      {grid.map((week, wi) => (
        <div className="hm-col" key={wi}>
          {week.map((day, di) => {
            const future = day > today;
            const v = byDate.get(localKey(day)) ?? 0;
            const bg = future ? "transparent" : RAMP[level(v, max)];
            return <div className="hm-cell" key={di} style={{ background: bg }} />;
          })}
        </div>
      ))}
    </div>
  );
}
