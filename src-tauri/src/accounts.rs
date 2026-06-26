use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
