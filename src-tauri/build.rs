fn main() {
    println!("cargo:rerun-if-changed=icons/icon.ico");

    let windows = tauri_build::WindowsAttributes::new().window_icon_path("icons/icon.ico");
    let attrs = tauri_build::Attributes::new().windows_attributes(windows);

    tauri_build::try_build(attrs).expect("failed to run tauri build script");
}
