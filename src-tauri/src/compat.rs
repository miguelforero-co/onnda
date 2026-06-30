//! Hardware compatibility helpers for gating Apple Silicon-only features and
//! selecting appropriate defaults per hardware. All helpers that shell out
//! (`sw_vers`, `sysctl`) are split into a pure parser function so tests can
//! run without spawning subprocesses.
//!
//! Varios helpers solo se usan en el camino aarch64 (disponibilidad de Apple
//! Speech). En builds x86_64 (Intel) quedan sin uso a propósito → silenciamos
//! dead_code a nivel de módulo en vez de cfg-gatear (siguen cubiertos por tests).
#![allow(dead_code)]

/// Parse the major version from `sw_vers -productVersion` output.
/// Examples: "26.5.1\n" → 26, "" → 0, "garbage" → 0.
/// Fallback 0 is safe: it causes the Apple engine gate to stay closed.
fn parse_macos_major(s: &str) -> u32 {
    s.trim()
        .split('.')
        .next()
        .and_then(|x| x.parse().ok())
        .unwrap_or(0)
}

/// Parse physical RAM bytes from `sysctl hw.memsize` output, returning GiB.
/// Example: "hw.memsize: 25769803776\n" → 24.
/// Fallback 8 GiB is conservative: it causes `hardware_default_model` to
/// choose "small" rather than "large-v3-turbo".
fn parse_ram_gb(s: &str) -> u64 {
    s.trim()
        .split_whitespace()
        .last()
        .and_then(|tok| tok.parse::<u64>().ok())
        .map(|bytes| bytes / (1024 * 1024 * 1024))
        .unwrap_or(8)
}

/// Returns the major version of macOS at runtime by shelling out to `sw_vers`.
/// Returns 0 on any error (safe fallback: Apple engine gate stays closed).
pub fn macos_major_version() -> u32 {
    std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| parse_macos_major(&s))
        .unwrap_or(0)
}

/// Returns physical RAM in GiB by reading `sysctl hw.memsize`.
/// Returns 8 on any error (conservative fallback → picks "small" model).
pub fn physical_ram_gb() -> u64 {
    std::process::Command::new("sysctl")
        .arg("hw.memsize")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| parse_ram_gb(&s))
        .unwrap_or(8)
}

/// Returns the hardware-appropriate default model ID for first-run setup.
///
/// Rules:
/// - x86_64 (Intel, any RAM)          → "small"
/// - aarch64, RAM < 16 GiB            → "small"
/// - aarch64, RAM ≥ 16 GiB            → "large-v3-turbo" (historical default)
///
/// "apple-speech" is intentionally excluded as a default: it requires macOS
/// assets to download and has no user confirmation flow yet.
pub fn hardware_default_model() -> &'static str {
    #[cfg(not(target_arch = "aarch64"))]
    {
        return "small";
    }
    #[cfg(target_arch = "aarch64")]
    {
        if physical_ram_gb() < 16 {
            "small"
        } else {
            "large-v3-turbo"
        }
    }
}

/// Returns `true` if the ASR sidecar binary is bundled and resolvable.
/// Uses the Tauri shell plugin — only resolves the path, does NOT execute
/// the sidecar (no side-effects, no processes spawned).
pub fn sidecar_available<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> bool {
    use tauri_plugin_shell::ShellExt;
    app.shell().sidecar("asr").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_macos_major ---

    #[test]
    fn parse_macos_major_normal() {
        assert_eq!(parse_macos_major("26.5.1\n"), 26);
    }

    #[test]
    fn parse_macos_major_no_trailing_newline() {
        assert_eq!(parse_macos_major("14.4.1"), 14);
    }

    #[test]
    fn parse_macos_major_empty() {
        assert_eq!(parse_macos_major(""), 0);
    }

    #[test]
    fn parse_macos_major_garbage() {
        assert_eq!(parse_macos_major("garbage"), 0);
    }

    // --- parse_ram_gb ---

    #[test]
    fn parse_ram_gb_24gib() {
        // 25769803776 bytes = 24 GiB exactly
        assert_eq!(parse_ram_gb("hw.memsize: 25769803776\n"), 24);
    }

    #[test]
    fn parse_ram_gb_8gib() {
        assert_eq!(parse_ram_gb("hw.memsize: 8589934592\n"), 8);
    }

    #[test]
    fn parse_ram_gb_invalid() {
        assert_eq!(parse_ram_gb("nope"), 8);
    }

    #[test]
    fn parse_ram_gb_empty() {
        assert_eq!(parse_ram_gb(""), 8);
    }

    // --- hardware_default_model ---

    #[test]
    fn hardware_default_model_returns_valid_id() {
        let valid = ["small", "large-v3-turbo"];
        let result = hardware_default_model();
        assert!(
            valid.contains(&result),
            "hardware_default_model() returned unexpected id: {:?}",
            result
        );
    }
}
