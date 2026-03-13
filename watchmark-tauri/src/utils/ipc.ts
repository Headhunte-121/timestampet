import { invoke } from "@tauri-apps/api/core";

/**
 * A wrapper around Tauri's invoke that enforces a strict timeout.
 *
 * @param cmd The tauri command name
 * @param args The arguments to pass to the command
 * @param timeoutMs The maximum amount of time to wait in milliseconds (default: 15000)
 * @returns Promise that resolves to the command result
 */
export async function invokeWithTimeout<T>(
  cmd: string,
  args?: Record<string, unknown>,
  timeoutMs: number = 15000
): Promise<T> {
  let timeoutId: number;

  const timeoutPromise = new Promise<never>((_, reject) => {
    timeoutId = window.setTimeout(() => {
      reject(new Error(`[IPC Timeout] Command '${cmd}' exceeded ${timeoutMs}ms limit.`));
    }, timeoutMs);
  });

  try {
    const result = await Promise.race([
      invoke<T>(cmd, args),
      timeoutPromise
    ]);
    return result as T;
  } catch (error: any) {
    console.error(`Error in invokeWithTimeout for ${cmd}:`, error);

    let errMessage = error instanceof Error ? error.message : (typeof error === "string" ? error : JSON.stringify(error));

    // Specifically catch Data Mismatches or bounds issues and highlight them
    if (errMessage.includes("Data Mismatch") || errMessage.includes("out of range")) {
      console.warn(`[IPC Type Mismatch Warning] -> ${cmd}:`, errMessage);
      errMessage = `Backend mismatch: ${errMessage}`;
    }

    throw errMessage;
  } finally {
    clearTimeout(timeoutId!);
  }
}
