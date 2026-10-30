//! Embeds the .exe icon and version info (res/app.rc) into the Windows executable.
//!
//! `embed-resource` finds the resource compiler by itself: on the MSVC toolchain it
//! locates `rc.exe` inside the installed Windows SDK (no Developer Command Prompt
//! needed); on the GNU/MinGW toolchain it uses `windres`.
//!
//! If no resource compiler is available the build still succeeds - the game just
//! has no icon on the .exe file in Explorer. The window/taskbar icon is set at
//! runtime and is unaffected.

fn main() {
    println!("cargo:rerun-if-changed=res/app.rc");
    println!("cargo:rerun-if-changed=res/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    if let Err(e) = embed_resource::compile("res/app.rc", embed_resource::NONE).manifest_optional() {
        println!("cargo:warning=could not embed the .exe icon ({e}). The game still works; see README 'Building from source'.");
    }
}
