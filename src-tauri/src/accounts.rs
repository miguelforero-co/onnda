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
