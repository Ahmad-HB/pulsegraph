import { quit } from "../api";

function ago(ts: number): string {
  if (!ts) return "—";
  const secs = Math.max(0, Math.floor(Date.now() / 1000) - ts);
  if (secs < 60) return `${secs}s ago`;
  if (secs < 3600) return `${Math.floor(secs / 60)}m ago`;
  return `${Math.floor(secs / 3600)}h ago`;
}

export function Footer({ generatedAt, onSettings }: { generatedAt: number; onSettings: () => void }) {
  return (
    <div className="footer">
      <span>Updated {ago(generatedAt)}</span>
      <span className="menu">
        <button onClick={onSettings}>⚙︎ Settings</button>
        <button className="quit" onClick={() => quit()}>Quit</button>
      </span>
    </div>
  );
}
