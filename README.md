# pulsegraph

**Your AI token usage, at a glance.** pulsegraph turns the transcripts that
Claude Code already writes to your disk into a GitHub-style contribution
heatmap — so you can see how much you're using AI, day by day.

No accounts, no network, no scraping: it reads `~/.claude/projects/**/*.jsonl`
locally and computes everything on your machine.

> **Status:** the `core` engine and a terminal **CLI** are working today. A
> cross-platform menu-bar / system-tray app (the primary experience) is the next
> milestone and is built on the same `core`.

## What it shows

- A full-year **heatmap** of your usage intensity.
- Switchable metric: **cost ($)**, **billable tokens** (input + output +
  cache-writes), **output tokens**, or **raw total** (everything incl. cache
  reads).
- Filter by **project** or **model**.
- Stat cards: total, best day, average per active day, and **current / longest
  streak**.

Cost is an estimate from a bundled per-model price table (cache writes and reads
priced with the documented multipliers); unknown models show `—` rather than a
fabricated number.

## Install / run (CLI)

Requires a recent stable Rust toolchain.

```bash
git clone https://github.com/Ahmad-HB/pulsegraph.git
cd pulsegraph

# Render the heatmap (defaults to the cost metric)
cargo run -p pulsegraph-cli -- --metric cost

# Other metrics and filters
cargo run -p pulsegraph-cli -- --metric billable --project my-repo
cargo run -p pulsegraph-cli -- --metric output --model claude-opus-4-8

# Machine-readable output
cargo run -p pulsegraph-cli -- --metric cost --json
```

Flags: `--metric <cost|billable|output|raw>`, `--project <name>`,
`--model <id>`, `--json`.

## How it works

```
~/.claude/projects/**/*.jsonl
        │  discover + stream-parse (tolerant of malformed lines)
        ▼
   UsageEvent (source-agnostic)
        │  aggregate by local-day / project / model
        ▼
   metrics + streaks + cost   ──►  CLI heatmap (and, soon, the tray app)
```

An incremental SQLite cache keyed by `(file, mtime)` means only changed
transcript files are re-parsed, so repeated runs are near-instant. The internal
`UsageEvent` is source-agnostic by design: support for other AI agents (Cursor,
Copilot, …) can be added later by writing a new parser, with no changes to
aggregation, metrics, or the UI.

## Project layout

- `crates/core` — `pulsegraph-core`: discovery, parsing, aggregation, pricing,
  streaks, and the incremental cache. Fully unit-tested.
- `crates/cli` — `pulsegraph-cli`: the `pulsegraph` terminal binary.
- `docs/superpowers/` — design spec and implementation plan.

## License

pulsegraph is **dual-licensed**:

- Open source under **AGPL-3.0** — see [`LICENSE`](./LICENSE).
- A **commercial license** is available for proprietary / closed-source use —
  see [`COMMERCIAL.md`](./COMMERCIAL.md).

© 2026 Ahmad Hbahbeh.
