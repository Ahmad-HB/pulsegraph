import { Dropdown } from "./Dropdown";
import type { Metric } from "../types";

const METRIC_OPTS: Metric[] = ["cost", "billable", "output", "raw"];
const METRIC_LABELS: Record<Metric, string> = { cost: "Cost $", billable: "Billable", output: "Output", raw: "Raw" };

export function Filters({
  metric, setMetric, projects, models, project, model, setProject, setModel,
}: {
  metric: Metric; setMetric: (m: Metric) => void;
  projects: string[]; models: string[];
  project: string | null; model: string | null;
  setProject: (p: string | null) => void; setModel: (m: string | null) => void;
}) {
  return (
    <div className="filters">
      <Dropdown
        value={metric}
        placeholder=""
        options={METRIC_OPTS}
        labels={METRIC_LABELS}
        clearable={false}
        onChange={(v) => setMetric((v ?? "cost") as Metric)}
      />
      <Dropdown value={project} placeholder="All projects" options={projects} onChange={setProject} />
      <Dropdown value={model} placeholder="All models" options={models} onChange={setModel} />
    </div>
  );
}
