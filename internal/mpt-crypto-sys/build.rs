//! Build script for mpt-crypto-sys.
//!
//! Resolves the native `libmpt-crypto.{so,dylib,dll}` in three tiers:
//!
//! 1. `MPT_CRYPTO_LIB_DIR` env var (offline / custom builds).
//! 2. `vendor/lib/<rust-target>/` committed in this crate (git-checkout flow).
//! 3. Downloaded from the upstream GitHub release, verified by SHA-256
//!    against `BUNDLE_SHA256`, cached in `OUT_DIR`.
//!
//! After resolution, emits linker directives, rpath, and copies the shared
//! library next to the final build artifact so tests and examples find it
//! at runtime.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Upstream mpt-crypto release tag this crate is built against.
/// Must match the version whose headers generated `src/bindings.rs`.
const MPT_CRYPTO_VERSION: &str = "0.3.0-rc2";

/// SHA-256 of `mpt-crypto-natives-<MPT_CRYPTO_VERSION>.tar.gz`.
/// Computed at release time; verified on every download.
///
/// Update via `scripts/fetch_upstream.sh` which prints the new value.
const BUNDLE_SHA256: &str =
    "486c775b5b3c1fc18c3a9e8cbd4b458802b49eb307475df695fc899127226c0d";

fn main() {
    let target = env::var("TARGET").expect("cargo did not set TARGET");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("cargo did not set OUT_DIR"));
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let (lib_filename, is_windows) = platform_lib(&target);
    let lib_dir = resolve_library_dir(&target, &lib_filename, &manifest_dir, &out_dir);

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=mpt-crypto");

    emit_rpath(&target);
    copy_lib_to_output(&lib_dir.join(&lib_filename), &out_dir, is_windows);

    println!("cargo:rerun-if-env-changed=MPT_CRYPTO_LIB_DIR");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/bindings.rs");
}

fn platform_lib(target: &str) -> (String, bool) {
    if target.contains("apple-darwin") {
        ("libmpt-crypto.dylib".into(), false)
    } else if target.contains("linux") {
        ("libmpt-crypto.so".into(), false)
    } else if target.contains("windows") {
        ("mpt-crypto.dll".into(), true)
    } else {
        panic!(
            "mpt-crypto-sys: unsupported target `{target}`. \
             Supported: *-apple-darwin, *-linux-gnu, x86_64-pc-windows-msvc."
        );
    }
}

fn resolve_library_dir(
    target: &str,
    lib_filename: &str,
    manifest_dir: &Path,
    out_dir: &Path,
) -> PathBuf {
    // Priority 1: explicit override via environment variable.
    if let Ok(custom) = env::var("MPT_CRYPTO_LIB_DIR") {
        let path = PathBuf::from(&custom);
        assert!(
            path.join(lib_filename).exists(),
            "MPT_CRYPTO_LIB_DIR=`{custom}` does not contain `{lib_filename}`"
        );
        return path;
    }

    // Priority 2: vendored in the repository.
    let vendored = manifest_dir.join("vendor/lib").join(target);
    if vendored.join(lib_filename).exists() {
        return vendored;
    }

    // Priority 3: fetch from upstream release.
    let cache_dir = out_dir.join("vendor/lib").join(target);
    if !cache_dir.join(lib_filename).exists() {
        download_and_extract(target, &cache_dir, out_dir);
    }
    cache_dir
}

fn download_and_extract(target: &str, dest: &Path, out_dir: &Path) {
    let url = format!(
        "https://github.com/XRPLF/mpt-crypto/releases/download/\
         {v}/mpt-crypto-natives-{v}.tar.gz",
        v = MPT_CRYPTO_VERSION,
    );
    let tarball = out_dir.join("mpt-crypto-natives.tar.gz");

    println!("cargo:warning=mpt-crypto-sys: downloading {url}");
    let resp = ureq::get(&url)
        .call()
        .unwrap_or_else(|e| panic!("mpt-crypto-sys: download failed: {e}"));
    let mut file = fs::File::create(&tarball).unwrap();
    io::copy(&mut resp.into_reader(), &mut file).unwrap();
    drop(file);

    verify_sha256(&tarball, BUNDLE_SHA256);

    fs::create_dir_all(dest).unwrap();
    let upstream = rust_to_upstream(target);

    let gz = fs::File::open(&tarball).unwrap();
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(gz));

    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path = entry.path().unwrap().to_path_buf();
        let s = path.to_string_lossy();
        let prefix_with_dot = format!("./{upstream}/");
        let prefix = format!("{upstream}/");
        if s.starts_with(&prefix_with_dot) || s.starts_with(&prefix) {
            if let Some(filename) = path.file_name() {
                entry.unpack(dest.join(filename)).unwrap();
            }
        }
    }
}

fn verify_sha256(path: &Path, expected: &str) {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).unwrap();
    let actual = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(
        actual, expected,
        "mpt-crypto-sys: SHA-256 mismatch on downloaded bundle\n\
         expected: {expected}\n\
         actual:   {actual}\n\
         Possible tampering, corrupted download, or MPT_CRYPTO_VERSION mismatch."
    );
}

fn rust_to_upstream(target: &str) -> &'static str {
    if target.starts_with("aarch64-apple-darwin") {
        "darwin-aarch64"
    } else if target.starts_with("x86_64-apple-darwin") {
        "darwin-x86-64"
    } else if target.starts_with("aarch64-unknown-linux-gnu") {
        "linux-aarch64"
    } else if target.starts_with("x86_64-unknown-linux-gnu") {
        "linux-x86-64"
    } else if target.starts_with("s390x-unknown-linux-gnu") {
        "linux-s390x"
    } else if target.starts_with("x86_64-pc-windows-msvc") {
        "win32-x86-64"
    } else {
        panic!("mpt-crypto-sys: no upstream bundle for target `{target}`");
    }
}

fn emit_rpath(target: &str) {
    // Three search directories per platform:
    //   @loader_path     — same directory as the binary
    //                      (e.g. `target/debug/examples/<ex>` with dylib in `target/debug/examples/`)
    //   @loader_path/..  — parent directory
    //                      (e.g. `target/debug/deps/<test>` with dylib in `target/debug/`)
    //   @loader_path/../lib — installed layout (binaries in bin/, libs in lib/)
    if target.contains("apple-darwin") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/..");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../lib");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/..");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib");
    }
    // Windows has no rpath concept — DLL must be adjacent to the .exe
    // (handled by copy_lib_to_output) or on %PATH%.
}

fn copy_lib_to_output(src: &Path, out_dir: &Path, is_windows: bool) {
    // OUT_DIR = target/<profile>/build/<crate>-<hash>/out.
    // Walk up three levels to reach target/<profile>/.
    let mut target_dir = out_dir.to_path_buf();
    for _ in 0..3 {
        target_dir.pop();
    }

    let filename = src.file_name().expect("library path has no file name");
    let dst = target_dir.join(filename);

    // Best-effort. Some CI / read-only filesystems can't write here; the
    // build still succeeds if so, and the rpath will fail at runtime with
    // a clearer message than a mysterious copy failure.
    if let Err(e) = fs::copy(src, &dst) {
        println!(
            "cargo:warning=mpt-crypto-sys: could not copy {} to {}: {e}",
            src.display(),
            dst.display()
        );
    }

    // Mirror next to the examples directory on Windows so `cargo run
    // --example <name>` finds the DLL without mucking with %PATH%.
    if is_windows {
        let examples_dir = target_dir.join("examples");
        if examples_dir.exists() {
            let _ = fs::copy(src, examples_dir.join(filename));
        }
    }
}
