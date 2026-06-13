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

export async function quit(): Promise<void> {
  await invoke("quit");
}

export async function getAvatar(): Promise<string | null> {
  return invoke<string | null>("get_avatar");
}
// Opens the native image picker; resolves to a data URL, or null if cancelled.
export async function pickAvatar(): Promise<string | null> {
  return invoke<string | null>("pick_avatar");
}
export async function clearAvatar(): Promise<void> {
  await invoke("clear_avatar");
}
