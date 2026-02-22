use std::{collections::HashSet, env, process::Command};

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

/// Build script entry point for native Soulver linking and rpath configuration.
fn main() {
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
