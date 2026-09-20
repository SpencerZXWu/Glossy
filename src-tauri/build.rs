use std::path::{Path, PathBuf};
use std::time::Duration;

fn main() {
    stage_webview2_loader();
    tauri_build::build();
    link_resources_into_test_binaries();
}

/// `true` for the target this project ships: Windows built with the GNU toolchain.
fn is_gnu_windows() -> bool {
    std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu")
}

/// Stops a build that cannot provide the DLL `bundle.resources` lists, with the
/// reason, rather than letting `tauri_build` fail on a missing resource path.
fn missing_loader(why: &str) -> ! {
    panic!(
        "cannot stage resources/WebView2Loader.dll: {why}. This project builds the \
         Windows app with the GNU toolchain (`rustup toolchain install \
         stable-x86_64-pc-windows-gnu --component rustfmt --component clippy`), whose \
         WebView2 SDK build is where that DLL comes from; MSVC links the loader into \
         the executable instead. `bundle.resources` in tauri.conf.json lists the file, \
         so a build that cannot stage it cannot bundle either."
    )
}

/// Keeps a copy of `WebView2Loader.dll` in `resources/`, at the path
/// `bundle.resources` in `tauri.conf.json` hands to the installer.
///
/// The GNU toolchain cannot link the WebView2 loader the way MSVC does, so
/// `glossy.exe` imports `WebView2Loader.dll` and Windows refuses to start it when
/// the file is not next to the executable ("WebView2Loader.dll was not found").
/// `tauri_build` copies the DLL out of the `webview2-com-sys` build directory
/// beside the built executable, which is enough for a local run, but that copy
/// never reaches the installer: a bundle ships only what `bundle.resources`
/// lists. Staging the file before `tauri_build::build()` runs, at the path the
/// configuration names, is what puts it into the installer.
fn stage_webview2_loader() {
    let dest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("WebView2Loader.dll");

    if !is_gnu_windows() {
        // MSVC links the loader statically and has no file to give. A copy staged
        // by an earlier GNU build is still the loader to ship.
        if dest.is_file() {
            return;
        }
        missing_loader("this toolchain does not produce it");
    }

    let Some(profile_dir) = std::env::var("OUT_DIR")
        .ok()
        .map(PathBuf::from)
        .and_then(|out_dir| out_dir.ancestors().nth(3).map(Path::to_path_buf))
    else {
        return;
    };

    let arch = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => "x64",
        Ok("x86") => "x86",
        Ok("aarch64") => "arm64",
        _ => return,
    };

    // The build script of `webview2-com-sys` may still be running, so wait for the
    // loader to appear instead of losing the race and shipping without it.
    let mut source = None;
    for attempt in 0..40 {
        source = find_webview2_loader(&profile_dir, arch);
        if source.is_some() {
            break;
        }
        if attempt < 39 {
            std::thread::sleep(Duration::from_millis(250));
        }
    }

    let Some(source) = source else {
        if dest.is_file() {
            println!(
                "cargo:warning=no WebView2Loader.dll below {}; keeping the copy already \
                 staged in resources/",
                profile_dir.display()
            );
            return;
        }
        missing_loader(&format!(
            "there is no WebView2Loader.dll below {}",
            profile_dir.display()
        ));
    };

    let unchanged = std::fs::read(&dest)
        .ok()
        .zip(std::fs::read(&source).ok())
        .is_some_and(|(dest, source)| dest == source);
    if unchanged {
        return;
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).expect("failed to create the resources directory");
    }
    std::fs::copy(&source, &dest).expect("failed to stage WebView2Loader.dll");
}

/// `target/<profile>/WebView2Loader.dll` is the copy `tauri_build` leaves beside
/// the executable; the one under the `webview2-com-sys` build directory is the
/// original shipped by the WebView2 SDK.
fn find_webview2_loader(profile_dir: &Path, arch: &str) -> Option<PathBuf> {
    let beside_executable = profile_dir.join("WebView2Loader.dll");
    if beside_executable.is_file() {
        return Some(beside_executable);
    }

    std::fs::read_dir(profile_dir.join("build"))
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("webview2-com-sys")
        })
        .map(|entry| {
            entry
                .path()
                .join("out")
                .join(arch)
                .join("WebView2Loader.dll")
        })
        .find(|candidate| candidate.is_file())
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
