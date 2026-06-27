import { invoke } from "@tauri-apps/api/core";

export type AccountPublic = { id: string; email: string; name: string; created_at: number };

class AuthStore {
  account = $state<AccountPublic | null>(null);

  async load() {
    this.account = (await invoke<AccountPublic | null>("account_current")) ?? null;
  }
  async signup(name: string, email: string, password: string) {
    this.account = await invoke<AccountPublic>("account_signup", { name, email, password });
  }
  async login(email: string, password: string) {
    this.account = await invoke<AccountPublic>("account_login", { email, password });
  }
  async logout() {
    await invoke("account_logout");
    this.account = null;
  }
  async resetPassword(email: string, newPassword: string) {
    await invoke("account_reset_password", { email, newPassword });
  }
}

export const auth = new AuthStore();
