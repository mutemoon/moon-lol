(window as any).IS_DESKTOP =
  (window as any).IS_DESKTOP ||
  (window as any).__TAURI__ !== undefined ||
  (window as any).__TAURI_INTERNALS__ !== undefined ||
  import.meta.env.VITE_IS_DESKTOP === "true";
