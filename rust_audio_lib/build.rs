#[cfg(any(target_os = "macos", target_os = "ios"))]
fn main() {
    println!("cargo:rustc-link-lib=framework=CoreAudio");
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
fn main() {
    eprintln!("This library requires macos or ios target");
}