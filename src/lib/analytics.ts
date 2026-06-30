import { invoke } from "@tauri-apps/api/core";

/** Fire-and-forget. The opt-in guard lives in Rust; this never throws. */
export async function track(event: string, props?: Record<string, unknown>): Promise<void> {
  try {
    await invoke("track_event", { event, props: props ?? null });
  } catch {
    // analytics must never break the UI
  }
}
