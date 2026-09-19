use std::path::PathBuf;

fn main() {
    tauri_build::build();

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let native_dir = manifest_dir.parent().expect("workspace root").join("native");

    let mut build = cc::Build::new();
    build.cpp(true);
    build.std("c++17");
    build.file(native_dir.join("cpp").join("oculus_engine.cc"));
    build.include(native_dir.join("include"));

    // Check for Homebrew Tesseract and Leptonica on macOS
    if let Ok(brew_prefix) = std::process::Command::new("/opt/homebrew/bin/brew")
        .arg("--prefix")
        .output()
    {
        if brew_prefix.status.success() {
            let prefix_str = String::from_utf8_lossy(&brew_prefix.stdout).trim().to_string();
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
    } else {
        println!("cargo:rustc-link-lib=lept");
    }

    build.compile("oculus_native");

    println!("cargo:rerun-if-changed={}", native_dir.join("include").join("oculus_engine.h").display());
    println!("cargo:rerun-if-changed={}", native_dir.join("cpp").join("oculus_engine.cc").display());
}
