// Thin wrapper over Tauri's invoke() that normalises errors.
//
// Rust commands return Result<T, String>; on Err, Tauri rejects the promise
// with that string. We re-wrap it as an Error so the UI can rely on
// `err.message` everywhere.

import { invoke as tauriInvoke } from "@tauri-apps/api/core";

export async function invoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await tauriInvoke<T>(command, args);
  } catch (e) {
    // Tauri rejects with the Err(String) payload (or an object).
    const message = typeof e === "string" ? e : (e as Error)?.message ?? String(e);
    throw new Error(message);
  }
}
