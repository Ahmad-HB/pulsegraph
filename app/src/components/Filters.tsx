export function Filters({
  projects, models, project, model, setProject, setModel,
}: {
  projects: string[]; models: string[];
  project: string | null; model: string | null;
  setProject: (p: string | null) => void; setModel: (m: string | null) => void;
}) {
  return (
    <div className="filters">
      <select value={project ?? ""} onChange={(e) => setProject(e.target.value || null)}>
        <option value="">All projects</option>
        {projects.map((p) => <option key={p} value={p}>{p}</option>)}
      </select>
      <select value={model ?? ""} onChange={(e) => setModel(e.target.value || null)}>
        <option value="">All models</option>
        {models.map((m) => <option key={m} value={m}>{m}</option>)}
      </select>
    </div>
  );
}
