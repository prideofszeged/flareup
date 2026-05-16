use std::{
    collections::HashSet,
    env,
    path::{Path, PathBuf},
    process::Command,
};

/// Builds the runtime library search path for `libSoulverWrapper.so`.
///
/// This keeps both common Swift install locations so packaged binaries work
/// with either tarball installs (`/opt/swift/...`) or apt installs
/// (`/usr/lib/swift/...`), while still preferring co-located app libraries.
fn soulver_wrapper_runpath() -> String {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();

    if let Ok(swift_libdir) = env::var("SWIFT_LIBDIR") {
        if !swift_libdir.is_empty() && seen.insert(swift_libdir.clone()) {
            entries.push(swift_libdir);
        }
    }

    for entry in [
        "/opt/swift/usr/lib/swift/linux",
        "/usr/lib/swift/linux",
        "$ORIGIN",
        "$ORIGIN/../../Vendor/SoulverCore-linux",
    ] {
        if seen.insert(entry.to_string()) {
            entries.push(entry.to_string());
        }
    }

    entries.join(":")
}

/// Patches an ELF object with the provided rpath and fails hard on non-zero
/// `patchelf` exit so broken runtime linkage is detected during build.
fn set_rpath(binary: &str, rpath: &str) {
    let status = Command::new("patchelf")
        .arg("--set-rpath")
        .arg(rpath)
        .arg(binary)
        .status()
        .unwrap_or_else(|e| panic!("Failed to spawn patchelf for {binary}: {e}"));
    assert!(
        status.success(),
        "patchelf exited with {status} while patching {binary}"
    );
}

/// Runs a git command in the repository root and returns trimmed stdout.
fn git_output(repo_root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?;
    Some(value.trim().to_string())
}

/// Resolves the git metadata directory, including worktree setups.
fn resolve_git_dir(repo_root: &Path) -> Option<PathBuf> {
    let raw = git_output(repo_root, &["rev-parse", "--git-dir"])?;
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        Some(path)
    } else {
        Some(repo_root.join(path))
    }
}

/// Emits build script rerun triggers for git revision and dirty-state changes.
fn emit_git_rerun_hints(repo_root: &Path) {
    let Some(git_dir) = resolve_git_dir(repo_root) else {
        return;
    };

    let head = git_dir.join("HEAD");
    let index = git_dir.join("index");
    println!("cargo:rerun-if-changed={}", head.display());
    println!("cargo:rerun-if-changed={}", index.display());

    if let Ok(head_contents) = std::fs::read_to_string(&head) {
        if let Some(reference) = head_contents.strip_prefix("ref: ") {
            let ref_path = git_dir.join(reference.trim());
            println!("cargo:rerun-if-changed={}", ref_path.display());
        }
    }
}

/// Exposes a CLI version with git build metadata.
///
/// Example: `0.1.4+g74a9e26` or `0.1.4+g74a9e26.dirty`.
fn emit_cli_version(repo_root: &Path) {
    let package_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    let short_sha = git_output(repo_root, &["rev-parse", "--short=8", "HEAD"])
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "nogit".to_string());
    let dirty = git_output(repo_root, &["status", "--porcelain", "--untracked-files=no"])
        .map(|out| !out.is_empty())
        .unwrap_or(false);
    let build_id = if dirty {
        format!("g{short_sha}.dirty")
    } else {
        format!("g{short_sha}")
    };

    println!("cargo:rustc-env=FLARE_CLI_VERSION={package_version}+{build_id}");
}

/// Build script entry point for native Soulver linking and rpath configuration.
fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let repo_root = manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest_dir.clone());

    emit_git_rerun_hints(&repo_root);
    emit_cli_version(&repo_root);

    // Ensure the Swift wrapper can find SoulverCore when installed via deb:
    // libSoulverWrapper.so lives in .../SoulverWrapper/.build/release
    // and SoulverCore is in .../SoulverWrapper/Vendor/SoulverCore-linux.
    set_rpath(
        "SoulverWrapper/.build/release/libSoulverWrapper.so",
        &soulver_wrapper_runpath(),
    );

    set_rpath(
        "SoulverWrapper/Vendor/SoulverCore-linux/libSoulverCoreDynamic.so",
        "$ORIGIN",
    );

    println!("cargo:rustc-link-search=native=SoulverWrapper/.build/release");

    println!("cargo:rustc-link-lib=SoulverWrapper");

    // Deployed/bundled app rpath (deb installs libs to /usr/lib/flare/...)
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib/flare/SoulverWrapper/.build/release");

    // Dev-time rpaths are intentionally embedded to support running binaries
    // directly from target/{debug,release} during local development.
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../../SoulverWrapper/.build/release");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../../SoulverWrapper/Vendor/SoulverCore-linux");

    tauri_build::build();
}
