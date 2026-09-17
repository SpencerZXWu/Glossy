use std::path::PathBuf;

fn main() {
    tauri_build::build();
    link_resources_into_test_binaries();
}

/// `tauri_build` embeds the generated resources (icon, version info and, most
/// importantly, the application manifest) into binary targets only. Windows
/// resolves `TaskDialogIndirect` — which the WebView2 stack imports — only
/// through the side-by-side common controls v6 assembly that the manifest
/// requests, so without it a `cargo test` harness dies with
/// `STATUS_ENTRYPOINT_NOT_FOUND` (0xC0000139) before `main` runs.
///
/// The linker reports `.rsrc merge failure: multiple non-default manifests`
/// for binary targets, which receive the same archive from `tauri_build`;
/// keeping the first resource directory leaves the executable with the correct
/// manifest. Build scripts cannot tell which kind of target is being compiled
/// (`cargo test` runs harnesses for `src/lib.rs`, `src/main.rs` and the doc
/// tests), so every target gets the archive.
fn link_resources_into_test_binaries() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let Ok(out_dir) = std::env::var("OUT_DIR") else {
        return;
    };

    let resources = PathBuf::from(out_dir).join("libresource.a");
    if resources.is_file() {
        // Plain `rustc-link-arg` covers every target, including the unit test
        // harness that `rustc-link-arg-tests` skips for a `rlib`-only package.
        println!("cargo:rustc-link-arg={}", resources.display());
    }
}
