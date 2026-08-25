//! Puts Steam's redistributable library next to the executable.
//!
//! `steamworks-sys` unpacks the Steamworks SDK into its own `OUT_DIR` while it
//! builds. Cargo links against the import library from there, but the *runtime*
//! library (steam_api64.dll on Windows) has to sit beside the exe or the app
//! will not start. This script copies it across after every build.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use glob::glob;

/// Extensions that have to travel with the executable.
const RUNTIME: [&str; 3] = ["dll", "so", "dylib"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Look for the runtime library beside the executable, not in a system path.
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    }

    // OUT_DIR is <target>/<profile>/build/sam-<hash>/out, so three levels up is
    // the directory the executable is written to.
    let out_dir = env::var("OUT_DIR")?;
    let target_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .ok_or("could not work out the target directory from OUT_DIR")?
        .to_path_buf();

    let sources = find_runtime_libraries()?;

    if sources.is_empty() {
        // Not fatal. The link step gets what it needs directly from
        // steamworks-sys; only the copy is missing, and a second build fixes it
        // once that crate has finished unpacking the SDK.
        println!(
            "cargo:warning=Could not find steam_api64.dll to copy next to the executable. \
             Run the build again; if it still does not appear, copy it manually out of \
             target/<profile>/build/steamworks-sys-*/out/ into the folder holding sam.exe."
        );
        return Ok(());
    }

    for source in sources {
        let Some(file_name) = source.file_name() else {
            continue;
        };
        let destination = target_dir.join(file_name);
        // A failed copy is usually the app still running and holding the DLL
        // open, which is worth a clear warning rather than a wall of backtrace.
        if let Err(error) = fs::copy(&source, &destination) {
            println!(
                "cargo:warning=Could not copy {} to {}: {error}. Close sam.exe and build again.",
                source.display(),
                destination.display()
            );
        }
    }

    Ok(())
}

/// Every runtime library `steamworks-sys` has unpacked, newest build first.
///
/// The glob is relative, which is fine: Cargo runs build scripts with the crate
/// root as the working directory. CARGO_TARGET_DIR is honoured when set.
fn find_runtime_libraries() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let target_root = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "./target".to_owned());

    // The glob crate treats `\` as an escape character on every platform, so a
    // Windows path taken from CARGO_TARGET_DIR would silently match nothing.
    // Windows accepts forward slashes in paths, so rewriting is safe. Trailing
    // separators are dropped to avoid a doubled slash in the pattern.
    let target_root = target_root
        .trim_end_matches(['/', '\\'])
        .replace('\\', "/");

    let pattern = format!("{target_root}/**/build/steamworks-sys-*/out/**/*");

    let mut found: Vec<PathBuf> = glob(&pattern)?
        .filter_map(Result::ok)
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| RUNTIME.iter().any(|want| ext.eq_ignore_ascii_case(want)))
                .unwrap_or(false)
        })
        .collect();

    // Deduplicate by file name so a stale debug copy does not overwrite a fresh
    // release one. Whichever the glob yields first wins.
    let mut seen: Vec<std::ffi::OsString> = Vec::new();
    found.retain(|path| match path.file_name() {
        Some(name) if !seen.contains(&name.to_os_string()) => {
            seen.push(name.to_os_string());
            true
        }
        _ => false,
    });

    Ok(found)
}
