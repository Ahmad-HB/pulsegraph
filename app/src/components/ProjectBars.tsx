import type { ProjectValue } from "../types";

export function ProjectBars({ items, fmt }: { items: ProjectValue[]; fmt: (v: number) => string }) {
  if (items.length === 0) return null;
  const max = items.reduce((m, p) => Math.max(m, p.value), 0) || 1;
  return (
    <div className="projbars">
      <div className="label">Top projects · today</div>
      {items.map((p) => (
        <div className="pb" key={p.name}>
          <span className="nm" title={p.name}>{p.name}</span>
          <span className="track"><span className="fill" style={{ width: `${(p.value / max) * 100}%` }} /></span>
          <span className="v">{fmt(p.value)}</span>
        </div>
      ))}
    </div>
  );
}
