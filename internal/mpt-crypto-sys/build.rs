//! Build script for mpt-crypto-sys.
//!
//! Resolves the self-contained STATIC archive (`libmpt-crypto.a` /
//! `mpt-crypto-static.lib`, with secp256k1 + OpenSSL merged in) in three tiers:
//!
//! 1. `MPT_CRYPTO_LIB_DIR` env var (offline / custom builds).
//! 2. `vendor/lib/<rust-target>/` committed in this crate (git-checkout flow).
//! 3. Downloaded from the upstream GitHub release, verified by SHA-256
//!    against `BUNDLE_SHA256`, cached in `OUT_DIR`.
//!
//! The archive is STATICALLY linked into the consuming binary, so there is no
//! shared library to ship or locate at runtime — no rpath, no copy step. The
//! archive's own dependencies (the C++ runtime + OpenSSL's OS libraries) are
//! co-linked as system libraries, read from the `mpt-crypto-static.link-libs.txt`
//! manifest staged next to the archive (with per-platform fallbacks).

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Upstream mpt-crypto release tag this crate is built against.
/// Must match the version whose headers generated `src/bindings.rs`.
const MPT_CRYPTO_VERSION: &str = "1.0.4";

/// SHA-256 of `mpt-crypto-natives-<MPT_CRYPTO_VERSION>.tar.gz`.
/// Computed at release time; verified on every download.
///
/// Update via `scripts/fetch_upstream.sh` which prints the new value.
const BUNDLE_SHA256: &str = "d1ab71ca8d23028acdc2a877602e7a44e9b14b6a2cc33976888a793533804c9d";

fn main() {
    // docs.rs builds in a network-isolated sandbox with no native
    // `libmpt-crypto` available, so tier-3 download resolution would fail.
    // rustdoc type-checks the crate but never performs the final native link,
    // so skipping resolution + link directives lets `cargo doc` succeed while
    // producing identical documentation. (docs.rs sets `DOCS_RS=1`.)
    if env::var_os("DOCS_RS").is_some() {
        println!("cargo:rerun-if-changed=build.rs");
        return;
    }

    let target = env::var("TARGET").expect("cargo did not set TARGET");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("cargo did not set OUT_DIR"));
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let archive = archive_filename(&target);
    let lib_dir = resolve_library_dir(&target, &archive, &manifest_dir, &out_dir);

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    // Statically link the whole self-contained archive INTO the consumer, then
    // the system libraries it depends on. Order matters for static linking: the
    // archive's undefined symbols are resolved by libraries listed AFTER it.
    println!("cargo:rustc-link-lib=static={}", archive_link_name(&target));
    for lib in system_libs(&lib_dir, &target) {
        println!("cargo:rustc-link-lib=dylib={lib}");
    }

    println!("cargo:rerun-if-env-changed=MPT_CRYPTO_LIB_DIR");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/bindings.rs");
}

/// The static-archive filename staged per platform.
fn archive_filename(target: &str) -> String {
    if target.contains("apple-darwin") || target.contains("linux") {
        "libmpt-crypto.a".into()
    } else if target.contains("windows") {
        "mpt-crypto-static.lib".into()
    } else {
        panic!(
            "mpt-crypto-sys: unsupported target `{target}`. \
             Supported: *-apple-darwin, *-linux-gnu, x86_64-pc-windows-msvc."
        );
    }
}

/// The `cargo:rustc-link-lib=static=<name>` link name — the linker forms
/// `lib<name>.a` (unix) / `<name>.lib` (MSVC), matching the staged filenames.
fn archive_link_name(target: &str) -> &'static str {
    if target.contains("windows") {
        "mpt-crypto-static"
    } else {
        "mpt-crypto"
    }
}

fn resolve_library_dir(
    target: &str,
    archive: &str,
    manifest_dir: &Path,
    out_dir: &Path,
) -> PathBuf {
    // Priority 1: explicit override via environment variable.
    if let Ok(custom) = env::var("MPT_CRYPTO_LIB_DIR") {
        let path = PathBuf::from(&custom);
        assert!(
            path.join(archive).exists(),
            "MPT_CRYPTO_LIB_DIR=`{custom}` does not contain `{archive}`"
        );
        return path;
    }

    // Priority 2: vendored in the repository.
    let vendored = manifest_dir.join("vendor/lib").join(target);
    if vendored.join(archive).exists() {
        return vendored;
    }

    // Priority 3: fetch from upstream release.
    let cache_dir = out_dir.join("vendor/lib").join(target);
    if !cache_dir.join(archive).exists() {
        download_and_extract(target, &cache_dir, out_dir);
        // The bundle is expected to contain `<upstream-subdir>/<archive>`; fail
        // clearly here if extraction didn't produce it (e.g. an unexpected
        // release layout) rather than at a cryptic linker "cannot find -lmpt-crypto".
        assert!(
            cache_dir.join(archive).exists(),
            "mpt-crypto-sys: extracted bundle for `{target}` did not contain `{archive}` \
             (expected under `{}/`) — check the {MPT_CRYPTO_VERSION} release layout",
            rust_to_upstream(target),
        );
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
    // Bound the download so a hung connection fails the build instead of
    // blocking forever: 30s to connect, and a read timeout so a stalled stream
    // (no bytes) aborts. A healthy multi-MB/s CI download finishes well within.
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .timeout_read(Duration::from_secs(180))
        .build();
    let resp = agent
        .get(&url)
        .call()
        .unwrap_or_else(|e| panic!("mpt-crypto-sys: download failed: {e}"));
    let mut file = fs::File::create(&tarball).unwrap();
    io::copy(&mut resp.into_reader(), &mut file).unwrap();
    drop(file);

    verify_sha256(&tarball, BUNDLE_SHA256);

    fs::create_dir_all(dest).unwrap();
    let upstream = rust_to_upstream(target);

    // Unpack every file under the platform's subdir — the static archive plus
    // its `mpt-crypto-static.link-libs.txt` manifest (the shared library in the
    // bundle is simply ignored on the static path).
    let gz = fs::File::open(&tarball).unwrap();
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(gz));

    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path = entry.path().unwrap().to_path_buf();
        let s = path.to_string_lossy();
        let prefix_with_dot = format!("./{upstream}/");
        let prefix = format!("{upstream}/");
        if (s.starts_with(&prefix_with_dot) || s.starts_with(&prefix))
            && let Some(filename) = path.file_name()
        {
            entry.unpack(dest.join(filename)).unwrap();
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

/// The system libraries to co-link with the static archive, read from the
/// `mpt-crypto-static.link-libs.txt` manifest staged next to it (one name per
/// line, `#` comments and blanks ignored). These are the C++ runtime + OpenSSL's
/// OS deps the shared library would otherwise leave dynamic. Falls back to
/// per-platform defaults if the manifest is absent (e.g. an `MPT_CRYPTO_LIB_DIR`
/// override that stages only the archive).
fn system_libs(lib_dir: &Path, target: &str) -> Vec<String> {
    let manifest = lib_dir.join("mpt-crypto-static.link-libs.txt");
    if let Ok(contents) = fs::read_to_string(&manifest) {
        let libs: Vec<String> = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(String::from)
            .collect();
        if !libs.is_empty() {
            return libs;
        }
    }
    fallback_system_libs(target)
}

fn fallback_system_libs(target: &str) -> Vec<String> {
    let libs: &[&str] = if target.contains("apple-darwin") {
        &["c++"]
    } else if target.contains("linux") {
        &["stdc++", "pthread", "dl", "m"]
    } else if target.contains("windows") {
        &[
            "crypt32",
            "ws2_32",
            "advapi32",
            "user32",
            "gdi32",
            "bcrypt",
            "legacy_stdio_definitions",
        ]
    } else {
        &[]
    };
    libs.iter().map(|s| (*s).to_string()).collect()
}
