//! Update check command (check-only, no signing infra — D-14).
//!
//! Queries the GitHub Releases API for the latest published release and compares
//! its version to the running app version. No auto-install in Phase 1; wiring a
//! real signed updater (tauri-plugin-updater + endpoints + minisign keypair) is
//! deferred user_setup and would replace this module. Any failure (offline, no
//! releases yet → 404, rate-limited) is surfaced as a benign "up to date" with the
//! error captured, so the UI shows "Estás al día" without a hard failure.

use tauri::{AppHandle, Runtime};

const RELEASES_LATEST_API: &str =
    "https://api.github.com/repos/miguelforero-co/onnda/releases/latest";

#[derive(serde::Serialize)]
pub struct UpdateStatus {
    pub up_to_date: bool,
    pub available_version: Option<String>,
    pub current_version: String,
    pub error: Option<String>,
}

/// Parse a version string ("v1.7.0", "1.7.0", "app-v1.7.0") into numeric components.
/// Returns None if no dotted numeric core is found.
fn parse_version(s: &str) -> Option<Vec<u64>> {
    let core = s.trim_start_matches(|c: char| !c.is_ascii_digit());
    let nums: Vec<u64> = core
        .split('.')
        .map(|p| {
            p.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
        })
        .take_while(|p| !p.is_empty())
        .filter_map(|p| p.parse::<u64>().ok())
        .collect();
    if nums.is_empty() {
        None
    } else {
        Some(nums)
    }
}

/// True if `latest` is strictly newer than `current` (component-wise semver compare).
fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => {
            let n = l.len().max(c.len());
            for i in 0..n {
                let lv = l.get(i).copied().unwrap_or(0);
                let cv = c.get(i).copied().unwrap_or(0);
                if lv != cv {
                    return lv > cv;
                }
            }
            false
        }
        // If either side can't be parsed, treat a literal difference as "newer".
        _ => latest.trim_start_matches('v') != current.trim_start_matches('v'),
    }
}

#[tauri::command]
pub async fn check_for_updates<R: Runtime>(app: AppHandle<R>) -> Result<UpdateStatus, String> {
    let current = app.package_info().version.to_string();

    let up_to_date_with = |error: Option<String>| UpdateStatus {
        up_to_date: true,
        available_version: None,
        current_version: current.clone(),
        error,
    };

    let client = match reqwest::Client::builder()
        .user_agent(format!("onnda/{current}"))
        .build()
    {
        Ok(c) => c,
        Err(e) => return Ok(up_to_date_with(Some(e.to_string()))),
    };

    let resp = match client
        .get(RELEASES_LATEST_API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return Ok(up_to_date_with(Some(e.to_string()))),
    };

    // 404 = no releases published yet → benign "up to date".
    if !resp.status().is_success() {
        return Ok(up_to_date_with(Some(format!("HTTP {}", resp.status()))));
    }

    let body = match resp.text().await {
        Ok(b) => b,
        Err(e) => return Ok(up_to_date_with(Some(e.to_string()))),
    };

    let tag = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| {
            v.get("tag_name")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
        });

    match tag {
        Some(tag) if is_newer(&tag, &current) => Ok(UpdateStatus {
            up_to_date: false,
            available_version: Some(tag.trim_start_matches('v').to_string()),
            current_version: current,
            error: None,
        }),
        Some(_) => Ok(up_to_date_with(None)),
        None => Ok(up_to_date_with(Some("unexpected response from GitHub".into()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_version_variants() {
        assert_eq!(parse_version("v1.7.0"), Some(vec![1, 7, 0]));
        assert_eq!(parse_version("1.7.0"), Some(vec![1, 7, 0]));
        assert_eq!(parse_version("app-v2.0"), Some(vec![2, 0]));
        assert_eq!(parse_version("nope"), None);
    }

    #[test]
    fn newer_detection() {
        assert!(is_newer("v1.8.0", "1.7.0"));
        assert!(is_newer("1.7.1", "1.7.0"));
        assert!(!is_newer("1.7.0", "1.7.0"));
        assert!(!is_newer("v1.6.0", "1.7.0"));
        assert!(is_newer("2.0", "1.9.9"));
    }
}
