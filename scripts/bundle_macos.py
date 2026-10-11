#!/usr/bin/env python3
"""
scripts/bundle_macos.py — Standalone macOS Bundling & Dynamic Library Relocation Script

Recursively traces all Homebrew dynamic library dependencies for OCULUS, bundles them
into an application bundle (Contents/Frameworks/), rewrites all install names and
cross-dependency load commands to @rpath, bundles Tesseract language models, and re-signs
binaries for clean-machine distribution.
"""

import os
import sys
import shutil
import subprocess
import re
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
# Ad-hoc signature by default; set APPLE_SIGNING_IDENTITY (a Developer ID) for a distributable signature.
SIGN_ID = os.environ.get("APPLE_SIGNING_IDENTITY", "-")
# A Developer ID signature needs the hardened runtime and a secure timestamp to be notarised.
SIGN_ARGS = [] if SIGN_ID == "-" else ["--options", "runtime", "--timestamp"]

def run_cmd(cmd, check=True):
    res = subprocess.run(cmd, capture_output=True, text=True)
    if check and res.returncode != 0:
        print(f"Command failed: {' '.join(cmd)}")
        print(f"Stdout: {res.stdout}")
        print(f"Stderr: {res.stderr}")
        raise RuntimeError(f"Command exited with code {res.returncode}")
    return res.stdout.strip()

def get_dependencies(binary_path):
    out = run_cmd(['otool', '-L', str(binary_path)])
    deps = []
    lines = out.splitlines()
    for line in lines[1:]:
        match = re.search(r'^\s*([^\s]+)', line)
        if match:
            deps.append(match.group(1))
    return deps

def trace_all_homebrew_deps(root_dylibs):
    visited = set()
    to_visit = list(root_dylibs)
    homebrew_libs = {}

    while to_visit:
        current = to_visit.pop()
        real = os.path.realpath(current)
        if real in visited:
            continue
        visited.add(real)
        
        base_name = os.path.basename(current)
        homebrew_libs[base_name] = real

        deps = get_dependencies(real)
        for dep in deps:
            if dep.startswith('/opt/homebrew') or dep.startswith('/usr/local'):
                if dep not in to_visit and os.path.realpath(dep) not in visited:
                    to_visit.append(dep)
            elif dep.startswith('@rpath/'):
                # Look in same directory as current dylib or Homebrew lib directories
                rpath_base = dep.replace('@rpath/', '')
                candidate = Path(real).parent / rpath_base
                if candidate.exists() and os.path.realpath(str(candidate)) not in visited:
                    to_visit.append(str(candidate))
                else:
                    for search_dir in [Path('/opt/homebrew/lib'), Path('/opt/homebrew/opt/webp/lib')]:
                        c2 = search_dir / rpath_base
                        if c2.exists() and os.path.realpath(str(c2)) not in visited:
                            to_visit.append(str(c2))

    return homebrew_libs

def bundle_application(source_executable, output_dir, existing_app=None):
    print("=== OCULUS STANDALONE MACOS BUNDLER ===")
    print(f"Source executable: {source_executable}")
    print(f"Output directory:  {output_dir}")

    app_dir = Path(existing_app) if existing_app else output_dir / "Oculus.app"
    contents_dir = app_dir / "Contents"
    macos_dir = contents_dir / "MacOS"
    frameworks_dir = contents_dir / "Frameworks"
    resources_dir = contents_dir / "Resources"
    tessdata_dir = resources_dir / "tessdata"

    if app_dir.exists() and not existing_app:
        shutil.rmtree(app_dir)

    macos_dir.mkdir(parents=True, exist_ok=True)
    frameworks_dir.mkdir(parents=True, exist_ok=True)
    tessdata_dir.mkdir(parents=True, exist_ok=True)

    # Create Info.plist
    info_plist_path = contents_dir / "Info.plist"
    info_plist_content = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>oculus-desktop</string>
    <key>CFBundleIdentifier</key>
    <string>com.araskova.oculus</string>
    <key>CFBundleName</key>
    <string>OCULUS</string>
    <key>CFBundleDisplayName</key>
    <string>OCULUS</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
"""
    if not existing_app:  # a Tauri-built .app already carries its own Info.plist and icon
        info_plist_path.write_text(info_plist_content, encoding="utf-8")

    # 1. Copy executable
    target_exec = macos_dir / "oculus-desktop"
    if Path(source_executable).resolve() != target_exec.resolve():
        shutil.copy2(source_executable, target_exec)
    os.chmod(target_exec, 0o755)
    print(f"[1/6] Copied executable -> {target_exec}")

    # 2. Trace all transitive Homebrew dylibs from initial dependencies
    exec_deps = get_dependencies(source_executable)
    initial_hb_deps = [d for d in exec_deps if d.startswith('/opt/homebrew') or d.startswith('/usr/local')]
    print(f"Direct Homebrew dependencies of executable: {len(initial_hb_deps)}")

    all_hb_libs = trace_all_homebrew_deps(initial_hb_deps)
    print(f"[2/6] Traced total transitive dynamic libraries: {len(all_hb_libs)}")
    for name, path in sorted(all_hb_libs.items()):
        print(f"  - {name} ({path})")

    # 3. Copy dylibs into Frameworks/
    name_mapping = {}
    for base_name, src_path in all_hb_libs.items():
        dst_path = frameworks_dir / base_name
        shutil.copy2(src_path, dst_path)
        os.chmod(dst_path, 0o755)
        name_mapping[src_path] = f"@rpath/{base_name}"
        name_mapping[base_name] = f"@rpath/{base_name}"
    print(f"[3/6] Bundled {len(all_hb_libs)} dynamic libraries into Contents/Frameworks/")

    # 4. Rewrite install names and dependencies for each bundled dylib
    print("[4/6] Rewriting dylib install names and cross-references via install_name_tool...")
    for base_name in all_hb_libs.keys():
        dylib_path = frameworks_dir / base_name
        # Set self ID
        run_cmd(['install_name_tool', '-id', f"@rpath/{base_name}", str(dylib_path)])

        # Ensure dylib has @loader_path in LC_RPATH so it can locate sibling dylibs
        d_load_cmds = run_cmd(['otool', '-l', str(dylib_path)])
        if '@loader_path' not in d_load_cmds:
            run_cmd(['install_name_tool', '-add_rpath', '@loader_path', str(dylib_path)])

        # Rewrite all references to other bundled libraries
        deps = get_dependencies(dylib_path)
        for dep in deps:
            if dep.startswith('/opt/homebrew') or dep.startswith('/usr/local'):
                dep_base = os.path.basename(dep)
                # If target is in our bundled libs
                if dep_base in all_hb_libs or dep in name_mapping:
                    run_cmd(['install_name_tool', '-change', dep, f"@rpath/{dep_base}", str(dylib_path)])

    # 5. Rewrite executable dependencies and set LC_RPATH
    print("[5/6] Updating main executable load commands...")
    for dep in exec_deps:
        if dep.startswith('/opt/homebrew') or dep.startswith('/usr/local'):
            dep_base = os.path.basename(dep)
            run_cmd(['install_name_tool', '-change', dep, f"@rpath/{dep_base}", str(target_exec)])

    # Add @executable_path/../Frameworks to LC_RPATH
    rpath_entry = "@executable_path/../Frameworks"
    load_cmds = run_cmd(['otool', '-l', str(target_exec)])
    if rpath_entry not in load_cmds:
        run_cmd(['install_name_tool', '-add_rpath', rpath_entry, str(target_exec)])
        print(f"  Added LC_RPATH: {rpath_entry}")

    # 6. Copy tessdata language models
    src_tessdata = PROJECT_ROOT / "tessdata"
    if src_tessdata.exists():
        for item in src_tessdata.glob("*"):
            if item.is_file():
                shutil.copy2(item, tessdata_dir / item.name)
        print(f"[6/6] Bundled language data from {src_tessdata} -> {tessdata_dir}")
    else:
        print("Warning: local tessdata directory not found, checking /opt/homebrew/share/tessdata...")
        for name in ["eng.traineddata", "mal.traineddata", "osd.traineddata", "pdf.ttf"]:
            src = Path("/opt/homebrew/share/tessdata") / name
            if src.exists():
                shutil.copy2(src, tessdata_dir / name)

    # 7. Ad-hoc code sign all dylibs and executable
    print("Code signing bundled artifacts with ad-hoc signature...")
    for dylib_path in frameworks_dir.glob("*.dylib"):
        run_cmd(['codesign', '-f', *SIGN_ARGS, '-s', SIGN_ID, str(dylib_path)])
    run_cmd(['codesign', '-f', *SIGN_ARGS, '-s', SIGN_ID, str(target_exec)])
    if existing_app:  # seal the whole bundle so the signature covers Info.plist and resources
        run_cmd(['codesign', '-f', '--deep', *SIGN_ARGS, '-s', SIGN_ID, str(app_dir)])

    # 8. Verification: Check for any remaining forbidden paths
    print("\n--- DEPENDENCY VERIFICATION AUDIT ---")
    forbidden_found = False

    def check_file(path):
        nonlocal forbidden_found
        dlist = get_dependencies(path)
        for d in dlist:
            if d.startswith('/opt/homebrew') or d.startswith('/usr/local'):
                print(f"FAIL: {path.name} still depends on forbidden path: {d}")
                forbidden_found = True
            elif d.startswith('@rpath/'):
                rpath_target = frameworks_dir / d.replace('@rpath/', '')
                if not rpath_target.exists():
                    print(f"FAIL: {path.name} depends on missing @rpath target: {d}")
                    forbidden_found = True

    check_file(target_exec)
    for dylib_path in frameworks_dir.glob("*.dylib"):
        check_file(dylib_path)

    if forbidden_found:
        print("\nVerification FAILED: Unresolved Homebrew paths found!")
        sys.exit(1)
    else:
        print("Verification PASSED: ZERO Homebrew or developer paths remain in bundle!")
        print(f"Standalone application created at: {app_dir}\n")

if __name__ == "__main__":
    src = PROJECT_ROOT / "target" / "debug" / "oculus-desktop"
    if not src.exists():
        src = PROJECT_ROOT / "src-tauri" / "target" / "debug" / "oculus-desktop"
    
    out = PROJECT_ROOT / "dist-standalone"
    existing = None
    if len(sys.argv) > 2 and sys.argv[1] == "--app":
        # usage: bundle_macos.py --app path/to/OCULUS.app  (relocate dylibs into an existing bundle)
        existing = Path(sys.argv[2])
        src = existing / "Contents" / "MacOS" / "oculus-desktop"
        out = existing.parent
    else:
        if len(sys.argv) > 1:
            src = Path(sys.argv[1])
        if len(sys.argv) > 2:
            out = Path(sys.argv[2])

    bundle_application(src, out, existing)
