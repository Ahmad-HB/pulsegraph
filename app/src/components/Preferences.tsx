import { PRESETS, rampFor, type Theme, type PaletteName, type Mode } from "../lib/theme";

const PRESET_NAMES: Exclude<PaletteName, "custom">[] = ["github-green", "blue", "purple", "amber"];

export function Preferences({
  theme, setTheme, onBack, unreadable,
}: { theme: Theme; setTheme: (t: Theme) => void; onBack: () => void; unreadable: number; }) {
  const ramp = rampFor(theme);

  const setStop = (i: number, color: string) => {
    const next = [...rampFor(theme)];
    next[i] = color;
    setTheme({ ...theme, palette: "custom", customRamp: next });
  };

  return (
    <div className="prefs">
      <div className="row" style={{ justifyContent: "space-between" }}>
        <h3 style={{ margin: 0 }}>Preferences</h3>
        <button className="back" onClick={onBack}>← Back</button>
      </div>

      <div className="row">
        <span style={{ width: 70 }}>Palette</span>
        <div className="swatch">
          {PRESET_NAMES.map((name) => (
            <button
              key={name}
              className={theme.palette === name ? "on" : ""}
              title={name}
              style={{ background: `linear-gradient(135deg, ${PRESETS[name][1]}, ${PRESETS[name][4]})` }}
              onClick={() => setTheme({ ...theme, palette: name })}
            />
          ))}
        </div>
      </div>

      <div className="row">
        <span style={{ width: 70 }}>Mode</span>
        {(["dark", "light"] as Mode[]).map((m) => (
          <button key={m} className={`chip ${theme.mode === m ? "on" : ""}`} onClick={() => setTheme({ ...theme, mode: m })}>{m}</button>
        ))}
      </div>

      <div className="row" style={{ alignItems: "flex-start" }}>
        <span style={{ width: 70 }}>Custom</span>
        <div>
          <div className="ramp">
            {ramp.map((c, i) => (
              <input key={i} type="color" value={c} onChange={(e) => setStop(i, e.target.value)} />
            ))}
          </div>
          <div className="preview">
            {ramp.map((c, i) => <span key={i} style={{ background: c }} />)}
          </div>
        </div>
      </div>

      {unreadable > 0 && (
        <div className="row"><span className="empty">{unreadable} unreadable transcript lines skipped</span></div>
      )}
    </div>
  );
}
