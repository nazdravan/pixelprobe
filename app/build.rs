//! Puts the icon and the file description into pixelprobe.exe. Only for Windows targets; a
//! failure (e.g. no resource compiler when cross-checking from another OS) is a warning, so
//! the build itself never fails because of the icon.
fn main() {
    println!("cargo:rerun-if-changed=../assets/pixelprobe.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../assets/pixelprobe.ico");
    res.set("FileDescription", "pixelprobe - monitor test utility");
    res.set("ProductName", "pixelprobe");
    if let Err(e) = res.compile() {
        println!("cargo:warning=pixelprobe.exe without its icon: {e}");
    }
}
