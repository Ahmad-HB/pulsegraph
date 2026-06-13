import { useEffect, useRef, useState } from "react";

export function Dropdown({
  value, placeholder, options, onChange, labels, clearable = true,
}: {
  value: string | null;
  placeholder: string;
  options: string[];
  onChange: (v: string | null) => void;
  labels?: Record<string, string>;
  clearable?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const label = (v: string) => labels?.[v] ?? v;

  useEffect(() => {
    if (!open) return;
    const onDoc = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, [open]);

  const select = (v: string | null) => { onChange(v); setOpen(false); };

  return (
    <div className={`dd ${open ? "open" : ""}`} ref={ref}>
      <button className="dd-btn" onClick={() => setOpen((o) => !o)}>
        <span className="dd-label">{value === null ? placeholder : label(value)}</span>
        <span className="dd-caret">▾</span>
      </button>
      {open && (
        <div className="dd-menu">
          {clearable && (
            <button className={`dd-item ${value === null ? "on" : ""}`} onClick={() => select(null)}>{placeholder}</button>
          )}
          {options.map((o) => (
            <button key={o} className={`dd-item ${value === o ? "on" : ""}`} onClick={() => select(o)} title={label(o)}>{label(o)}</button>
          ))}
        </div>
      )}
    </div>
  );
}
