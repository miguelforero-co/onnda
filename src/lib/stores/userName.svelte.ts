// Display name for the Home greeting. Will be populated by onboarding (sign-in,
// or "¿cómo debería llamarte?"). Persisted under "onnda.userName". Empty until set.
const KEY = "onnda.userName";

class UserNameStore {
  value = $state<string>(typeof localStorage !== "undefined"
    ? (localStorage.getItem(KEY) ?? "")
    : "");

  constructor() {
    $effect.root(() => {
      $effect(() => {
        try { localStorage.setItem(KEY, this.value); } catch { /* ignore */ }
      });
    });
  }

  set(name: string) { this.value = name.trim(); }
}

export const userName = new UserNameStore();
