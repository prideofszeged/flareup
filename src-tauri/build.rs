use std::process::Command;

fn main() {
    // Ensure the Swift wrapper can find SoulverCore when installed via deb:
    // libSoulverWrapper.so lives in .../SoulverWrapper/.build/release
    // and SoulverCore is in .../SoulverWrapper/Vendor/SoulverCore-linux.
    let _ = Command::new("patchelf")
        .arg("--set-rpath")
        .arg("/opt/swift/usr/lib/swift/linux:$ORIGIN:$ORIGIN/../../Vendor/SoulverCore-linux")
        .arg("SoulverWrapper/.build/release/libSoulverWrapper.so")
        .status()
        .expect("Failed to patch elf for libSoulverWrapper");

    let _ = Command::new("patchelf")
        .arg("--set-rpath")
        .arg("$ORIGIN")
        .arg("SoulverWrapper/Vendor/SoulverCore-linux/libSoulverCoreDynamic.so")
        .status()
        .expect("Failed to patch elf for libSoulverCoreDynamic");

    println!("cargo:rustc-link-search=native=SoulverWrapper/.build/release");

    println!("cargo:rustc-link-lib=SoulverWrapper");

    // Deployed/bundled app rpath (deb installs libs to /usr/lib/flare/...)
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib/flare/SoulverWrapper/.build/release");

    // Dev-time rpath: from target/{debug,release}/ back to source tree
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../../SoulverWrapper/.build/release");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../../SoulverWrapper/Vendor/SoulverCore-linux");

    tauri_build::build();
}
