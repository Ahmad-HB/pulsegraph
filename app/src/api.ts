import { invoke } from "@tauri-apps/api/core";
import type { Snapshot, Metric } from "./types";

export async function refresh(): Promise<void> {
  await invoke("refresh");
}

export async function getSnapshot(
  metric: Metric,
  project: string | null,
  model: string | null,
): Promise<Snapshot> {
  return invoke<Snapshot>("get_snapshot", { metric, project, model });
}
