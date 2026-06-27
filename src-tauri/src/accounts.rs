use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime};

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

pub fn profile_subdir(base: &Path, current: Option<&str>) -> PathBuf {
    match current {
        Some(id) => base.join("profiles").join(id),
        None => base.to_path_buf(),
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("onnda-acct-{}-{}.json", tag, uuid::Uuid::new_v4()));
        p
    }

    fn tmp_dir(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("onnda-dir-{}-{}", tag, uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn email_and_password_validation() {
        assert!(validate_email("a@b.com").is_ok());
        assert!(validate_email("nope").is_err());
        assert!(validate_password("longenough").is_ok());
        assert!(validate_password("short").is_err());
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

    #[test]
    fn hash_then_verify() {
        let h = hash_password("s3cret-pw").unwrap();
        assert!(h.starts_with("$argon2id$"));
        assert!(verify_password("s3cret-pw", &h));
        assert!(!verify_password("wrong", &h));
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
}
