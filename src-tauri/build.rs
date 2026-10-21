use std::path::PathBuf;

fn main() {
    tauri_build::build();

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let native_dir = manifest_dir
        .parent()
        .expect("workspace root")
        .join("native");

    // The build script runs on the host; the *target* is what we are compiling for.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let is_msvc = target_env == "msvc";

    let mut build = cc::Build::new();
    build.cpp(true);
    if is_msvc {
        // cc only maps `std()` to /std: on newer releases, so pass the flags explicitly.
        build.flag("/std:c++17");
        build.flag("/EHsc");
        build.flag_if_supported("/utf-8");
    } else {
        build.std("c++17");
    }
    build.file(native_dir.join("cpp").join("oculus_engine.cc"));
    build.include(native_dir.join("include"));

    if target_os == "windows" {
        link_windows(&mut build);
    } else {
        link_posix(&mut build);
    }

    build.compile("oculus_native");

    println!(
        "cargo:rerun-if-changed={}",
        native_dir.join("include").join("oculus_engine.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        native_dir.join("cpp").join("oculus_engine.cc").display()
    );
}

/// macOS (Homebrew) and Linux (pkg-config: libtesseract-dev, libleptonica-dev).
fn link_posix(build: &mut cc::Build) {
    let is_linux = std::env::var("CARGO_CFG_TARGET_OS")
        .map(|os| os == "linux")
        .unwrap_or(false);

    // Check for Homebrew Tesseract and Leptonica on macOS
    if let Ok(brew_prefix) = std::process::Command::new("/opt/homebrew/bin/brew")
        .arg("--prefix")
        .output()
    {
        if brew_prefix.status.success() {
            let prefix_str = String::from_utf8_lossy(&brew_prefix.stdout)
                .trim()
                .to_string();
            let prefix_path = PathBuf::from(&prefix_str);
            build.include(prefix_path.join("include"));
            println!("cargo:rustc-link-search=native={}/lib", prefix_str);
        }
    }

    if let Ok(tess) = pkg_config::Config::new().probe("tesseract") {
        for include in tess.include_paths {
            build.include(include);
        }
        for lib in tess.libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    } else if is_linux {
        // A silent fallback here would build the stub engine (no OCR) without any warning.
        panic!(
            "pkg-config could not find 'tesseract'. Install it with:\n  \
             sudo apt install libtesseract-dev libleptonica-dev pkg-config"
        );
    } else {
        // Direct link fallback on macOS Homebrew
        println!("cargo:rustc-link-lib=tesseract");
    }

    if let Ok(lept) = pkg_config::Config::new().probe("lept") {
        for include in lept.include_paths {
            build.include(include);
        }
        for lib in lept.libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    } else if is_linux {
        panic!(
            "pkg-config could not find 'lept'. Install it with: sudo apt install libleptonica-dev"
        );
    } else {
        println!("cargo:rustc-link-lib=lept");
    }

    // On Linux the real Tesseract code path is required; a missing header is then a loud compile
    // error rather than a silent stub engine (same as the Windows build).
    if is_linux {
        build.define("OCULUS_HAS_TESSERACT", "1");
    }
}

/// Windows (MSVC): static Tesseract + Leptonica from vcpkg, so no DLLs have to be shipped.
///
/// Setup (see RELEASING.md):
///   vcpkg install "tesseract[core]:x64-windows-static-md"
///   set VCPKG_ROOT=<vcpkg checkout>
///   set VCPKGRS_TRIPLET=x64-windows-static-md      (the vcpkg crate's default when not crt-static)
///
/// `x64-windows-static-md` = static libraries built against the dynamic MSVC runtime, which is the
/// runtime Rust links by default, so the CRTs match.
#[cfg(windows)]
fn link_windows(build: &mut cc::Build) {
    println!("cargo:rerun-if-env-changed=VCPKG_ROOT");
    println!("cargo:rerun-if-env-changed=VCPKGRS_TRIPLET");
    println!("cargo:rerun-if-env-changed=VCPKGRS_DYNAMIC");

    let mut found_tesseract = false;
    // tesseract pulls leptonica in as a dependency; asking for leptonica too is harmless and
    // guards against a port that does not record the dependency.
    for port in ["tesseract", "leptonica"] {
        match vcpkg::Config::new().copy_dlls(false).find_package(port) {
            Ok(lib) => {
                // Visible in the CI log; shows exactly what was resolved on a first Windows run.
                println!(
                    "cargo:warning=vcpkg: {} -> triplet {}, ports [{}], libs [{}]",
                    port,
                    lib.vcpkg_triplet,
                    lib.ports.join(", "),
                    lib.found_names.join(", ")
                );
                for include in &lib.include_paths {
                    build.include(include);
                }
                if port == "tesseract" {
                    found_tesseract = true;
                }
            }
            Err(e) => {
                if port == "tesseract" {
                    panic!(
                        "vcpkg could not find the 'tesseract' port: {e}\n\
                         Install it with:  vcpkg install \"tesseract[core]:x64-windows-static-md\"\n\
                         and set VCPKG_ROOT to the vcpkg checkout and VCPKGRS_TRIPLET=x64-windows-static-md."
                    );
                } else if found_tesseract {
                    println!("cargo:warning=vcpkg: leptonica not resolved separately ({e}); relying on tesseract's dependency list");
                } else {
                    panic!("vcpkg could not find the 'leptonica' port: {e}");
                }
            }
        }
    }

    // Make the engine use the real Tesseract code path; a missing header is then a loud compile
    // error rather than a silent stub engine.
    build.define("OCULUS_HAS_TESSERACT", "1");

    // Windows system libraries that the static Tesseract/Leptonica dependency chain
    // (libarchive, curl, image codecs) can reference. Extra entries are harmless.
    for lib in [
        "ws2_32", "bcrypt", "crypt32", "user32", "advapi32", "shlwapi", "wldap32", "normaliz",
        "secur32", "iphlpapi",
    ] {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }
}

#[cfg(not(windows))]
fn link_windows(_build: &mut cc::Build) {
    panic!(
        "Building OCULUS for Windows requires a Windows host with vcpkg (static Tesseract); \
         cross-compiling from this host is not supported."
    );
}
