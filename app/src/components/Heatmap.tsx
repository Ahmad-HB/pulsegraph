import { buildYearGrid, level, localKey } from "../lib/heatmap";
import type { DayValue } from "../types";

const MONTHS = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];

export function Heatmap({ days }: { days: DayValue[] }) {
  const today = new Date();
  const grid = buildYearGrid(today);
  const byDate = new Map(days.map((d) => [d.date, d.value]));
  const max = days.reduce((m, d) => Math.max(m, d.value), 0);

  // Month labels: show a month name at the first column whose first row is in a new month.
  const monthLabels = grid.map((week, i) => {
    const first = week[0];
    const prev = i > 0 ? grid[i - 1][0] : null;
    return !prev || prev.getMonth() !== first.getMonth() ? MONTHS[first.getMonth()] : "";
  });

  return (
    <div className="hm-wrap">
      <div className="hm-months">
        {monthLabels.map((m, i) => <span key={i} className="hm-month">{m}</span>)}
      </div>
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
