use std::process::Command;

fn main() {
    // Embed the short git commit hash (+ dirty flag) so any build — especially
    // dev builds during iteration — is identifiable in the UI. SemVer in
    // tauri.conf.json marks releases; this hash distinguishes builds between them.
    let hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    let label = if dirty { format!("{hash}-dirty") } else { hash };
    println!("cargo:rustc-env=GIT_HASH={label}");
    // Rebuild the embedded hash when HEAD or the index changes.
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/index");

    tauri_build::build();
    // Note: codesign must run AFTER the linker finishes, so it cannot run here.
    // Use scripts/sign-dev.sh after each `cargo build` / `npm run tauri dev`.
}
