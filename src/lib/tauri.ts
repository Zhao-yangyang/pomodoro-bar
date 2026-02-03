export const isTauri = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export async function invokeTauri<T>(
  command: string,
  args?: Record<string, unknown>,
) {
  if (!isTauri()) {
    throw new Error("Not running inside Tauri.");
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

export async function listenTauri<T>(
  event: string,
  handler: (payload: T) => void,
) {
  if (!isTauri()) {
    return () => {};
  }

  const { listen } = await import("@tauri-apps/api/event");
  const unlisten = await listen<T>(event, (evt) => handler(evt.payload));
  return unlisten;
}
