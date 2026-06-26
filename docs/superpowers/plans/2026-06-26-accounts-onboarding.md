# Local Accounts + Onboarding + Per-Profile Data Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add required local email/password accounts with per-profile settings/history/dictionary, fixing the nameless Home greeting (#21), plus an optional email-capture annex (Vercel) on signup.

**Architecture:** A new Rust `accounts` module owns `accounts.json` (argon2id-hashed credentials, current session). All per-profile data moves under `app_data_dir/profiles/<account_id>/`; `settings`/`history`/`recordings` resolve their paths through `accounts::profile_dir`. The frontend gains an Auth screen that gates the app; a Svelte session store drives the greeting. The email annex posts `{email,name}` to a Vercel serverless function only when the user opts in.

**Tech Stack:** Rust, Tauri 2, `argon2`, `uuid`, SvelteKit/Svelte 5, Vercel serverless (Node) + Vercel KV.

## Global Constraints

- All account data is **local to the device**; no backend, no sync (the email annex is best-effort marketing capture only).
- Passwords stored only as **argon2id** hashes; never in clear, never logged.
- Login is a **gate, not encryption**: v1 does not encrypt per-profile data.
- "Forgot password" is a **local reset** (no email verification).
- Existing user data must be **preserved**: the first account created claims the legacy root data.
- `models/` stays **global** (shared across profiles), not per-profile.
- `AccountPublic` never exposes `password_hash`.
- Email annex sends data **only** when the signup checkbox is ticked; failure is silent and non-blocking.
- `cargo check` + `npm run check` green; existing Rust tests stay green.

---

### Task 1: Dependencies + Account types + store round-trip

**Files:**
- Modify: `src-tauri/Cargo.toml` (deps)
- Create: `src-tauri/src/accounts.rs`
- Modify: `src-tauri/src/lib.rs` (`mod accounts;`)

**Interfaces:**
- Produces:
  - `struct Account { id: String, email: String, name: String, password_hash: String, created_at: i64, provider: String }`
  - `struct AccountPublic { id, email, name, created_at }` (Serialize)
  - `struct AccountStore { accounts: Vec<Account>, current_account_id: Option<String> }` with `load_from(&Path) -> AccountStore` and `save_to(&self, &Path) -> std::io::Result<()>`.

- [ ] **Step 1: Add dependencies**

In `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
# Local account credential hashing.
argon2 = "0.5"
# Stable per-account ids.
uuid = { version = "1", features = ["v4"] }
```

- [ ] **Step 2: Write the failing test**

Create `src-tauri/src/accounts.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: i64,
    #[serde(default = "default_provider")]
    pub provider: String,
}

fn default_provider() -> String { "local".to_string() }

#[derive(Debug, Clone, Serialize)]
pub struct AccountPublic {
    pub id: String,
    pub email: String,
    pub name: String,
    pub created_at: i64,
}

impl From<&Account> for AccountPublic {
    fn from(a: &Account) -> Self {
        AccountPublic { id: a.id.clone(), email: a.email.clone(), name: a.name.clone(), created_at: a.created_at }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountStore {
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub current_account_id: Option<String>,
}

impl AccountStore {
    pub fn load_from(path: &Path) -> AccountStore {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).ok(); }
        let json = serde_json::to_string_pretty(self).unwrap();
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("onnda-acct-{}-{}.json", tag, uuid::Uuid::new_v4()));
        p
    }

    #[test]
    fn store_roundtrip_and_missing_file_is_empty() {
        let path = tmp_path("rt");
        assert!(AccountStore::load_from(&path).accounts.is_empty());
        let store = AccountStore {
            accounts: vec![Account {
                id: "id1".into(), email: "a@b.com".into(), name: "Mig".into(),
                password_hash: "h".into(), created_at: 1, provider: "local".into(),
            }],
            current_account_id: Some("id1".into()),
        };
        store.save_to(&path).unwrap();
        let loaded = AccountStore::load_from(&path);
        assert_eq!(loaded.accounts.len(), 1);
        assert_eq!(loaded.current_account_id.as_deref(), Some("id1"));
        std::fs::remove_file(&path).ok();
    }
}
```

Add `mod accounts;` to `src-tauri/src/lib.rs`.

- [ ] **Step 3: Run test to verify it fails, then passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml store_roundtrip`
Expected: compiles and PASSES once deps resolve (the code above is complete). If it fails to compile, fix the missing dep, not the logic.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/accounts.rs src-tauri/src/lib.rs
git commit -m "feat(accounts): Account/AccountStore types + JSON round-trip"
```

---

### Task 2: Password hashing + verification

**Files:**
- Modify: `src-tauri/src/accounts.rs`

**Interfaces:**
- Produces: `hash_password(pw: &str) -> Result<String, String>`, `verify_password(pw: &str, hash: &str) -> bool`.

- [ ] **Step 1: Write the failing test**

In `accounts.rs` `mod tests`:

```rust
    #[test]
    fn hash_then_verify() {
        let h = hash_password("s3cret-pw").unwrap();
        assert!(h.starts_with("$argon2id$"));
        assert!(verify_password("s3cret-pw", &h));
        assert!(!verify_password("wrong", &h));
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml hash_then_verify`
Expected: FAIL — `hash_password` not found.

- [ ] **Step 3: Implement**

Add to `accounts.rs` (top-level):

```rust
use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

pub fn hash_password(pw: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pw.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify_password(pw: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default().verify_password(pw.as_bytes(), &parsed).is_ok(),
        Err(_) => false,
    }
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml hash_then_verify`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/accounts.rs
git commit -m "feat(accounts): argon2id password hashing + verification"
```

---

### Task 3: Per-profile path resolution + legacy-claim helper (pure)

**Files:**
- Modify: `src-tauri/src/accounts.rs`

**Interfaces:**
- Produces:
  - `profile_subdir(base: &Path, current: Option<&str>) -> PathBuf` — `base/profiles/<id>` if `Some`, else `base` (legacy fallback).
  - `claim_legacy(base: &Path, profile: &Path) -> std::io::Result<()>` — moves `settings.json`, `history.json`, `recordings/` from `base` into `profile` if they exist at `base` and not yet in `profile`. Idempotent.

- [ ] **Step 1: Write the failing test**

In `mod tests`:

```rust
    fn tmp_dir(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("onnda-dir-{}-{}", tag, uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn profile_subdir_maps_current_or_falls_back() {
        let base = std::path::Path::new("/data");
        assert_eq!(profile_subdir(base, Some("abc")), std::path::Path::new("/data/profiles/abc"));
        assert_eq!(profile_subdir(base, None), base.to_path_buf());
    }

    #[test]
    fn claim_moves_legacy_data_once() {
        let base = tmp_dir("claim");
        std::fs::write(base.join("settings.json"), "{}").unwrap();
        std::fs::write(base.join("history.json"), "[]").unwrap();
        std::fs::create_dir_all(base.join("recordings")).unwrap();
        std::fs::write(base.join("recordings/a.wav"), "x").unwrap();

        let profile = base.join("profiles/acc1");
        claim_legacy(&base, &profile).unwrap();

        assert!(profile.join("settings.json").exists());
        assert!(profile.join("history.json").exists());
        assert!(profile.join("recordings/a.wav").exists());
        assert!(!base.join("settings.json").exists());
        // Idempotent: second call is a no-op, no panic.
        claim_legacy(&base, &profile).unwrap();
        std::fs::remove_dir_all(&base).ok();
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml claim_moves_legacy_data_once`
Expected: FAIL — functions not defined.

- [ ] **Step 3: Implement**

Add to `accounts.rs`:

```rust
use std::path::{Path, PathBuf};

pub fn profile_subdir(base: &Path, current: Option<&str>) -> PathBuf {
    match current {
        Some(id) => base.join("profiles").join(id),
        None => base.to_path_buf(),
    }
}

pub fn claim_legacy(base: &Path, profile: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(profile)?;
    for name in ["settings.json", "history.json", "recordings"] {
        let from = base.join(name);
        let to = profile.join(name);
        if from.exists() && !to.exists() {
            std::fs::rename(&from, &to)?;
        }
    }
    Ok(())
}
```

(Adjust the existing `use std::path::Path;` import to the combined `use std::path::{Path, PathBuf};`.)

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml accounts`
Expected: all accounts tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/accounts.rs
git commit -m "feat(accounts): profile_subdir resolution + idempotent legacy-claim"
```

---

### Task 4: Rewire settings/history paths through the profile dir

**Files:**
- Modify: `src-tauri/src/accounts.rs` (add `app_data_dir`-aware `profile_dir` + `store_path` + `current_id`)
- Modify: `src-tauri/src/settings.rs` (`settings_path`, add `clear_cache`)
- Modify: `src-tauri/src/history.rs` (`history_path`, `recordings_dir`)

**Interfaces:**
- Produces:
  - `accounts::store_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf` → `app_data_dir/accounts.json`.
  - `accounts::current_id<R: Runtime>(app: &AppHandle<R>) -> Option<String>`.
  - `accounts::profile_dir<R: Runtime>(app: &AppHandle<R>) -> PathBuf` (creates it).
  - `settings::clear_cache()`.

- [ ] **Step 1: Add Tauri-aware resolvers in `accounts.rs`**

```rust
use tauri::{AppHandle, Manager, Runtime};

pub fn store_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path().app_data_dir().expect("no app data dir").join("accounts.json")
}

pub fn current_id<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    AccountStore::load_from(&store_path(app)).current_account_id
}

pub fn profile_dir<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    let base = app.path().app_data_dir().expect("no app data dir");
    let dir = profile_subdir(&base, current_id(app).as_deref());
    std::fs::create_dir_all(&dir).ok();
    dir
}
```

- [ ] **Step 2: Point settings at the profile dir + add cache clear**

In `src-tauri/src/settings.rs`, change `settings_path`:

```rust
fn settings_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    crate::accounts::profile_dir(app).join("settings.json")
}
```

Add below the `CACHE` static:

```rust
/// Drop the cached settings — call on login/logout/signup so the next load
/// reads the newly-active profile.
pub fn clear_cache() {
    *CACHE.lock().unwrap() = None;
}
```

- [ ] **Step 3: Point history at the profile dir**

In `src-tauri/src/history.rs`:

```rust
fn history_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    crate::accounts::profile_dir(app).join("history.json")
}

fn recordings_dir<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    crate::accounts::profile_dir(app).join("recordings")
}
```

- [ ] **Step 4: Verify**

Run: `cargo check --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml`
Expected: green (pre-login, `current_id` is `None` → paths resolve to the legacy root, so existing behavior is unchanged until an account exists).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/accounts.rs src-tauri/src/settings.rs src-tauri/src/history.rs
git commit -m "feat(accounts): resolve settings/history under per-profile dir"
```

---

### Task 5: signup / login / logout / current / list / reset

**Files:**
- Modify: `src-tauri/src/accounts.rs`

**Interfaces:**
- Produces (all `<R: Runtime>(app: &AppHandle<R>, ...)`):
  - `signup(app, name, email, password) -> Result<AccountPublic, String>` — validates, hashes, creates id, **claims legacy data into the new profile**, sets `current_account_id`, clears settings cache.
  - `login(app, email, password) -> Result<AccountPublic, String>`.
  - `logout(app) -> Result<(), String>`.
  - `current(app) -> Option<AccountPublic>`.
  - `list(app) -> Vec<AccountPublic>`.
  - `reset_password(app, email, new_password) -> Result<(), String>`.

- [ ] **Step 1: Write failing tests for validation + duplicate email (pure split)**

First extract pure validators so they are unit-testable. In `mod tests`:

```rust
    #[test]
    fn email_and_password_validation() {
        assert!(validate_email("a@b.com").is_ok());
        assert!(validate_email("nope").is_err());
        assert!(validate_password("longenough").is_ok());
        assert!(validate_password("short").is_err());
    }
```

- [ ] **Step 2: Implement validators, run tests**

```rust
pub fn validate_email(email: &str) -> Result<(), String> {
    let e = email.trim();
    if e.len() >= 3 && e.contains('@') && e.split('@').nth(1).map_or(false, |d| d.contains('.')) {
        Ok(())
    } else {
        Err("Email inválido".into())
    }
}

pub fn validate_password(pw: &str) -> Result<(), String> {
    if pw.chars().count() >= 8 { Ok(()) } else { Err("La contraseña debe tener al menos 8 caracteres".into()) }
}
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml email_and_password_validation`
Expected: PASS.

- [ ] **Step 3: Implement the account operations**

```rust
pub fn signup<R: Runtime>(app: &AppHandle<R>, name: String, email: String, password: String) -> Result<AccountPublic, String> {
    validate_email(&email)?;
    validate_password(&password)?;
    let path = store_path(app);
    let mut store = AccountStore::load_from(&path);
    let email_norm = email.trim().to_lowercase();
    if store.accounts.iter().any(|a| a.email.to_lowercase() == email_norm) {
        return Err("Ya existe una cuenta con ese email".into());
    }
    let now = app_now_ms(app);
    let acct = Account {
        id: uuid::Uuid::new_v4().to_string(),
        email: email.trim().to_string(),
        name: name.trim().to_string(),
        password_hash: hash_password(&password)?,
        created_at: now,
        provider: "local".into(),
    };
    // Claim legacy root data into this new profile (first account only — the
    // helper is a no-op when the destination already has the files).
    let base = app.path().app_data_dir().expect("no app data dir");
    let profile = profile_subdir(&base, Some(&acct.id));
    claim_legacy(&base, &profile).map_err(|e| e.to_string())?;

    let pubacct = AccountPublic::from(&acct);
    store.accounts.push(acct);
    store.current_account_id = Some(pubacct.id.clone());
    store.save_to(&path).map_err(|e| e.to_string())?;
    crate::settings::clear_cache();
    Ok(pubacct)
}

pub fn login<R: Runtime>(app: &AppHandle<R>, email: String, password: String) -> Result<AccountPublic, String> {
    let path = store_path(app);
    let mut store = AccountStore::load_from(&path);
    let email_norm = email.trim().to_lowercase();
    let acct = store.accounts.iter().find(|a| a.email.to_lowercase() == email_norm)
        .ok_or_else(|| "Email o contraseña incorrectos".to_string())?;
    if !verify_password(&password, &acct.password_hash) {
        return Err("Email o contraseña incorrectos".into());
    }
    let pubacct = AccountPublic::from(acct);
    store.current_account_id = Some(pubacct.id.clone());
    store.save_to(&path).map_err(|e| e.to_string())?;
    crate::settings::clear_cache();
    Ok(pubacct)
}

pub fn logout<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let path = store_path(app);
    let mut store = AccountStore::load_from(&path);
    store.current_account_id = None;
    store.save_to(&path).map_err(|e| e.to_string())?;
    crate::settings::clear_cache();
    Ok(())
}

pub fn current<R: Runtime>(app: &AppHandle<R>) -> Option<AccountPublic> {
    let store = AccountStore::load_from(&store_path(app));
    let id = store.current_account_id.as_ref()?;
    store.accounts.iter().find(|a| &a.id == id).map(AccountPublic::from)
}

pub fn list<R: Runtime>(app: &AppHandle<R>) -> Vec<AccountPublic> {
    AccountStore::load_from(&store_path(app)).accounts.iter().map(AccountPublic::from).collect()
}

pub fn reset_password<R: Runtime>(app: &AppHandle<R>, email: String, new_password: String) -> Result<(), String> {
    validate_password(&new_password)?;
    let path = store_path(app);
    let mut store = AccountStore::load_from(&path);
    let email_norm = email.trim().to_lowercase();
    let acct = store.accounts.iter_mut().find(|a| a.email.to_lowercase() == email_norm)
        .ok_or_else(|| "No hay una cuenta con ese email en este Mac".to_string())?;
    acct.password_hash = hash_password(&new_password)?;
    store.save_to(&path).map_err(|e| e.to_string())?;
    Ok(())
}

fn app_now_ms<R: Runtime>(_app: &AppHandle<R>) -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}
```

- [ ] **Step 4: Verify compile + tests**

Run: `cargo check --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml accounts`
Expected: green.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/accounts.rs
git commit -m "feat(accounts): signup/login/logout/current/list/reset with legacy claim"
```

---

### Task 6: Tauri commands for accounts

**Files:**
- Modify: `src-tauri/src/accounts.rs` (command wrappers) or `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (`generate_handler!`)

**Interfaces:**
- Produces commands: `account_signup`, `account_login`, `account_logout`, `account_current`, `account_list`, `account_reset_password`.

- [ ] **Step 1: Add command wrappers**

In `src-tauri/src/accounts.rs`:

```rust
#[tauri::command]
pub fn account_signup<R: Runtime>(app: AppHandle<R>, name: String, email: String, password: String) -> Result<AccountPublic, String> {
    signup(&app, name, email, password)
}
#[tauri::command]
pub fn account_login<R: Runtime>(app: AppHandle<R>, email: String, password: String) -> Result<AccountPublic, String> {
    login(&app, email, password)
}
#[tauri::command]
pub fn account_logout<R: Runtime>(app: AppHandle<R>) -> Result<(), String> { logout(&app) }
#[tauri::command]
pub fn account_current<R: Runtime>(app: AppHandle<R>) -> Option<AccountPublic> { current(&app) }
#[tauri::command]
pub fn account_list<R: Runtime>(app: AppHandle<R>) -> Vec<AccountPublic> { list(&app) }
#[tauri::command]
pub fn account_reset_password<R: Runtime>(app: AppHandle<R>, email: String, new_password: String) -> Result<(), String> {
    reset_password(&app, email, new_password)
}
```

- [ ] **Step 2: Register commands**

In `src-tauri/src/lib.rs` `generate_handler!`, add:

```rust
            accounts::account_signup,
            accounts::account_login,
            accounts::account_logout,
            accounts::account_current,
            accounts::account_list,
            accounts::account_reset_password,
```

- [ ] **Step 3: Verify**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/accounts.rs src-tauri/src/lib.rs
git commit -m "feat(accounts): expose account commands"
```

---

### Task 7: Frontend session store + Auth screen + gate

**Files:**
- Create: `src/lib/auth.svelte.ts`
- Create: `src/lib/sections/Auth.svelte`
- Modify: `src/routes/+page.svelte` (gate on auth before the existing onboarding/app views)

**Interfaces:**
- Consumes: commands `account_current`, `account_signup`, `account_login`, `account_reset_password`, `account_logout`.
- Produces: `auth` store with `{ account: AccountPublic | null, load(), signup(...), login(...), logout(), resetPassword(...) }`.

- [ ] **Step 1: Session store**

Create `src/lib/auth.svelte.ts`:

```ts
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
```

- [ ] **Step 2: Auth screen**

Create `src/lib/sections/Auth.svelte` with two modes (Crear cuenta / Iniciar sesión) using the onnda flat design components. Crear cuenta fields: nombre, email, contraseña, confirmar; an optional checkbox `wantsNews` ("Quiero novedades de onnda"); microcopy "Tu cuenta es local en este Mac." Iniciar sesión fields: email, contraseña; link "Olvidé mi contraseña" toggling a reset form (email + nueva contraseña). On submit, call the matching `auth` method and surface returned error strings inline. Emit a `success` event the parent listens to. (Wire `wantsNews` to the annex in Task 10.)

- [ ] **Step 3: Gate the app**

In `src/routes/+page.svelte` `onMount`, before deciding the view, `await auth.load()`. If `auth.account` is null, set `view = "auth"`. Render `{#if view === "auth"}<Auth on:success={...} />{/if}` ahead of the onboarding/app branches. On `success`, re-run the existing settings load (now profile-scoped) and continue to onboarding/app.

- [ ] **Step 4: Verify**

Run: `npm run check`
Expected: green.

- [ ] **Step 5: Commit**

```bash
git add src/lib/auth.svelte.ts src/lib/sections/Auth.svelte src/routes/+page.svelte
git commit -m "feat(accounts): auth session store + Auth screen + app gate"
```

---

### Task 8: Account section in Settings + Home greeting (#21)

**Files:**
- Modify: `src/routes/+page.svelte` (Settings: "Cuenta" section; Home: greeting)

**Interfaces:**
- Consumes: `auth` store.

- [ ] **Step 1: Home greeting**

Replace the hardcoded "Hey," with `Hey, {auth.account?.name ?? ''}` (trim trailing comma/space when name is empty as a defensive fallback, though post-gate `account` is always set).

- [ ] **Step 2: Settings "Cuenta" section**

Add a section showing `auth.account.name` and `auth.account.email`, a "Cerrar sesión" button (`auth.logout()` → returns to Auth screen), and a "Cambiar contraseña" control that calls `auth.resetPassword(account.email, newPassword)`.

- [ ] **Step 3: Verify + manual smoke**

Run: `npm run check`
Expected: green. Then in dev: create an account → Home shows the name; existing history/settings still present (claimed); logout → Auth screen; create a 2nd account → fresh empty profile; log back into the 1st → its data returns.

- [ ] **Step 4: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat(accounts): Settings account section + named Home greeting (#21)"
```

---

### Task 9: Spec C — Vercel subscribe function

**Files:**
- Create: `api/subscribe.js` (Vercel serverless, Node)
- Create: `vercel.json` (if needed for the project)

**Interfaces:**
- Produces: `POST /api/subscribe` accepting `{ email, name }`, storing to Vercel KV, returning `{ ok: true }`.

- [ ] **Step 1: Implement the function**

Create `api/subscribe.js`:

```js
import { kv } from "@vercel/kv";

function validEmail(e) {
  return typeof e === "string" && /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(e);
}

export default async function handler(req, res) {
  if (req.method !== "POST") return res.status(405).json({ ok: false });
  const { email, name } = req.body ?? {};
  if (!validEmail(email)) return res.status(400).json({ ok: false, error: "invalid email" });
  const key = `sub:${email.toLowerCase()}`;
  const exists = await kv.get(key);
  if (!exists) {
    await kv.set(key, { email, name: typeof name === "string" ? name : "", ts: Date.now() });
    await kv.incr("sub:count");
  }
  return res.status(200).json({ ok: true });
}
```

- [ ] **Step 2: Deploy + smoke (external)**

Provision Vercel KV for the project and deploy. Smoke: `curl -X POST <url>/api/subscribe -H 'content-type: application/json' -d '{"email":"a@b.com","name":"Test"}'` → `{"ok":true}`; a second identical call must not double-count.

- [ ] **Step 3: Commit**

```bash
git add api/subscribe.js vercel.json
git commit -m "feat(emails): Vercel subscribe function backed by KV"
```

---

### Task 10: Wire signup → subscribe (opt-in, fire-and-forget)

**Files:**
- Create: `src/lib/subscribe.ts`
- Modify: `src/lib/sections/Auth.svelte` (call on signup when `wantsNews`)

**Interfaces:**
- Consumes: the deployed `/api/subscribe` URL.
- Produces: `subscribe(email: string, name: string): Promise<void>` — never throws.

- [ ] **Step 1: Frontend helper**

Create `src/lib/subscribe.ts`:

```ts
const ENDPOINT = "https://<your-vercel-app>.vercel.app/api/subscribe";

/** Best-effort marketing-list capture. Never blocks onboarding, never throws. */
export async function subscribe(email: string, name: string): Promise<void> {
  try {
    await fetch(ENDPOINT, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ email, name }),
    });
  } catch {
    // silent
  }
}
```

(Set `ENDPOINT` to the real deployed URL from Task 9.)

- [ ] **Step 2: Call on signup**

In `Auth.svelte`, after a successful `auth.signup(...)`, if `wantsNews` is checked, call `subscribe(email, name)` **without awaiting** the navigation on it (fire-and-forget): `void subscribe(email, name);`.

- [ ] **Step 3: Verify + smoke**

Run: `npm run check`
Expected: green. In dev, sign up with the box ticked → the email appears in KV; unticked → no request (verify with `read_network_requests`).

- [ ] **Step 4: Commit**

```bash
git add src/lib/subscribe.ts src/lib/sections/Auth.svelte
git commit -m "feat(emails): opt-in subscribe on signup (fire-and-forget)"
```

---

## Self-Review

- **Spec coverage:** local accounts + argon2 (Task 1,2) ✓; per-profile settings/history/dictionary via profile_dir (Task 3,4) — dictionary lives inside settings.json so it follows automatically ✓; required login gate (Task 7) ✓; legacy data claimed so nothing is lost (Task 3,5) ✓; multi-account on one Mac (Task 5 list/login + Task 8 manual smoke) ✓; local password reset (Task 5) ✓; named greeting #21 (Task 8) ✓; models/ stays global — untouched, only settings/history/recordings moved (Task 4) ✓; annex C Vercel + opt-in fire-and-forget (Task 9,10) ✓; `AccountPublic` hides hash (Task 1) ✓.
- **Type consistency:** `AccountStore.current_account_id: Option<String>`, `profile_subdir(base, Option<&str>)`, `current_id -> Option<String>` consistent across tasks; command names `account_*` match between Task 6 and Task 7's invoke calls; TS `resetPassword(email, newPassword)` matches command param `new_password` (Tauri camelCase mapping) ✓.
- **Placeholder scan:** Task 7 Step 2 and Task 8 describe UI assembly in prose (component composition in the existing onnda design system) rather than full markup — acceptable as these are layout-in-existing-system steps; all Rust logic and cross-task interfaces are concrete. `ENDPOINT` (Task 10) and Vercel URL are real external values filled from Task 9's deploy.
- **Decisions deferred to execution:** Vercel KV vs Postgres resolved to **KV** (Task 9).
