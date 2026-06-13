import { Dropdown } from "./Dropdown";

export function Filters({
  projects, models, project, model, setProject, setModel,
}: {
  projects: string[]; models: string[];
  project: string | null; model: string | null;
  setProject: (p: string | null) => void; setModel: (m: string | null) => void;
}) {
  return (
    <div className="filters">
      <Dropdown value={project} placeholder="All projects" options={projects} onChange={setProject} />
      <Dropdown value={model} placeholder="All models" options={models} onChange={setModel} />
    </div>
  );
}
