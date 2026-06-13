import { buildYearGrid, level, localKey } from "../lib/heatmap";
import type { DayValue } from "../types";

// Recent window — the full year overflows a menu-bar popover, and a ~8-month
// window comfortably covers actual usage while fitting the width.
const WEEKS = 34;

export function Heatmap({ days }: { days: DayValue[] }) {
  const today = new Date();
  const grid = buildYearGrid(today, WEEKS);
  const byDate = new Map(days.map((d) => [d.date, d.value]));
  const max = days.reduce((m, d) => Math.max(m, d.value), 0);

  return (
    <div className="hm-wrap">
      <div className="hm-body">
        <div className="hm-days">
          <span></span><span>Mon</span><span></span><span>Wed</span><span></span><span>Fri</span><span></span>
        </div>
        <div className="hm">
          {grid.map((week, wi) => (
            <div className="hm-col" key={wi}>
              {week.map((day, di) => {
                const future = day > today;
                const lvl = future ? -1 : level(byDate.get(localKey(day)) ?? 0, max);
                const cls = future ? "hm-cell future" : `hm-cell l${lvl}`;
                return <div className={cls} key={di} title={localKey(day)} />;
              })}
            </div>
          ))}
        </div>
      </div>
      <div className="hm-legend">
        <span>Less</span>
        <span className="hm-cell l0" /><span className="hm-cell l1" /><span className="hm-cell l2" />
        <span className="hm-cell l3" /><span className="hm-cell l4" />
        <span>More</span>
      </div>
    </div>
  );
}
