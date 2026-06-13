/** Map a value to a 0..4 intensity bucket given the period max. */
export function level(value: number, max: number): number {
  if (max <= 0 || value <= 0) return 0;
  const frac = value / max;
  if (frac > 0.75) return 4;
  if (frac > 0.5) return 3;
  if (frac > 0.25) return 2;
  return 1;
}

/** 53-week x 7-day grid (rows = Mon..Sun), Monday-aligned, ending in the week of `today`. */
export function buildYearGrid(today: Date): Date[][] {
  const WEEKS = 53;
  const end = new Date(today.getFullYear(), today.getMonth(), today.getDate());
  // Anchor the LAST column on the Monday of today's week, then back up
  // (WEEKS-1) weeks so today falls inside the final column.
  const start = new Date(end);
  const mondayOffset = (start.getDay() + 6) % 7; // days since Monday
  start.setDate(start.getDate() - mondayOffset - (WEEKS - 1) * 7);

  const weeks: Date[][] = [];
  for (let w = 0; w < WEEKS; w++) {
    const col: Date[] = [];
    for (let d = 0; d < 7; d++) {
      const cur = new Date(start);
      cur.setDate(start.getDate() + w * 7 + d);
      col.push(cur);
    }
    weeks.push(col);
  }
  return weeks;
}

/** YYYY-MM-DD in local time (matches core's local-day keys). */
export function localKey(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}
