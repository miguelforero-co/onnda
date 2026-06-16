// Theme store (Svelte 5 runes). mode = user choice; resolved = actual theme applied.
// Persisted to localStorage under "onnda.theme" (key mirrored in app.html bootstrap).
export type ThemeMode = "light" | "dark" | "auto";

const KEY = "onnda.theme";

function readMode(): ThemeMode {
  if (typeof localStorage === "undefined") return "auto";
  const v = localStorage.getItem(KEY);
  return v === "light" || v === "dark" || v === "auto" ? v : "auto";
}

function systemDark(): boolean {
  return typeof window !== "undefined" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches;
}

class ThemeStore {
  mode = $state<ThemeMode>(readMode());
  #systemDark = $state<boolean>(systemDark());

  // The theme actually shown: "auto" follows the OS.
  resolved = $derived<"light" | "dark">(
    this.mode === "auto" ? (this.#systemDark ? "dark" : "light") : this.mode
  );

  constructor() {
    if (typeof window !== "undefined") {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      mq.addEventListener("change", (e) => { this.#systemDark = e.matches; });
    }
    // Keep <html data-theme> and storage in sync with resolved/mode.
    $effect.root(() => {
      $effect(() => {
        document.documentElement.setAttribute("data-theme", this.resolved);
      });
      $effect(() => {
        try { localStorage.setItem(KEY, this.mode); } catch { /* ignore */ }
      });
    });
  }

  set(mode: ThemeMode) { this.mode = mode; }
}

export const theme = new ThemeStore();
