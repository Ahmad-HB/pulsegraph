import type { Metric } from "../types";

const METRICS: Metric[] = ["cost", "billable", "output", "raw"];
const LABEL: Record<Metric, string> = { cost: "Cost $", billable: "Billable", output: "Output", raw: "Raw" };

export function Header({ today, metric, setMetric }: { today: string; metric: Metric; setMetric: (m: Metric) => void; }) {
  return (
    <div className="header">
      <div className="header-top">
        <div className="title">PulseGraph <small>· Today</small></div>
        <div className="big">{today}</div>
      </div>
      <div className="chips">
        {METRICS.map((m) => (
          <button key={m} className={`chip ${m === metric ? "on" : ""}`} onClick={() => setMetric(m)}>{LABEL[m]}</button>
        ))}
      </div>
    </div>
  );
}
