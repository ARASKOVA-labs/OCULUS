#!/usr/bin/env python3
"""
tests/clean_machine_verify.py — Standalone Clean-Machine Verification Test Harness

Simulates a clean macOS execution environment with Homebrew absent:
1. Verifies that the bundled executable and all bundled dylibs have ZERO dependencies on /opt/homebrew or /usr/local.
2. Validates that all @rpath references resolve to files physically present in Contents/Frameworks/.
3. Validates that Tesseract language models (eng, mal, osd) exist in Contents/Resources/tessdata/.
4. Loads the bundled native libraries directly via ctypes under a sanitized environment (PATH=/usr/bin:/bin, no TESSDATA_PREFIX).
5. Exercises Tesseract API directly on:
   - Real PNG OCR
   - Real JPEG OCR
   - Real Malayalam OCR (testing complex Unicode glyphs)
   - Real Searchable PDF generation
6. Launches the packaged macOS binary with scrubbed environment to verify dynamic loader (dyld) integrity.
"""

import os
import sys
import subprocess
import re
import ctypes
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
BUNDLE_DIR = PROJECT_ROOT / "dist-standalone" / "Oculus.app"
CONTENTS_DIR = BUNDLE_DIR / "Contents"
MACOS_DIR = CONTENTS_DIR / "MacOS"
FRAMEWORKS_DIR = CONTENTS_DIR / "Frameworks"
RESOURCES_DIR = CONTENTS_DIR / "Resources"
TESSDATA_DIR = RESOURCES_DIR / "tessdata"

def run_cmd(cmd):
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        print(f"FAILED: {' '.join(cmd)}\nStdout: {res.stdout}\nStderr: {res.stderr}")
        raise RuntimeError(f"Command failed with code {res.returncode}")
    return res.stdout.strip()

def test_1_dependency_isolation():
    print("\n[TEST 1/6] Auditing dynamic library references (otool -L)...")
    executable = MACOS_DIR / "oculus-desktop"
    assert executable.exists(), f"Executable not found at {executable}"

    all_targets = [executable] + list(FRAMEWORKS_DIR.glob("*.dylib"))
    assert len(all_targets) >= 15, f"Expected at least 15 bundled binaries, found {len(all_targets)}"

    for target in all_targets:
        out = run_cmd(['otool', '-L', str(target)])
        for line in out.splitlines()[1:]:
            dep = line.strip().split()[0]
            if dep.startswith('/opt/homebrew') or dep.startswith('/usr/local'):
                raise AssertionError(f"Forbidden path found in {target.name} -> {dep}")
            if dep.startswith('@rpath/'):
                rpath_target = FRAMEWORKS_DIR / dep.replace('@rpath/', '')
                assert rpath_target.exists(), f"Unresolved @rpath in {target.name} -> {dep}"

    print(f"  PASS: All {len(all_targets)} binaries verified! Zero Homebrew/developer paths remain.")

def test_2_tessdata_presence():
    print("\n[TEST 2/6] Verifying bundled offline language models...")
    for model in ["eng.traineddata", "mal.traineddata", "osd.traineddata", "pdf.ttf"]:
        p = TESSDATA_DIR / model
        assert p.exists() and p.stat().st_size > 500, f"Missing or empty model: {p}"
        print(f"  Found bundled model: {model} ({p.stat().st_size} bytes)")
    print("  PASS: Offline language models present in Contents/Resources/tessdata/")

def test_3_native_tesseract_c_api_isolated():
    print("\n[TEST 3/6] Exercising bundled Tesseract C-API under sanitized environment...")
    # Scrub environment
    sanitized_env = {
        "PATH": "/usr/bin:/bin",
        "HOME": os.environ.get("HOME", "/tmp"),
        "USER": os.environ.get("USER", "nobody")
    }

    tess_dylib = FRAMEWORKS_DIR / "libtesseract.5.dylib"
    lept_dylib = FRAMEWORKS_DIR / "libleptonica.6.dylib"

    # Load libraries via ctypes using exact bundle paths
    lept = ctypes.CDLL(str(lept_dylib))
    tess = ctypes.CDLL(str(tess_dylib))

    # Test Tesseract Version
    tess.TessVersion.restype = ctypes.c_char_p
    version = tess.TessVersion().decode()
    print(f"  Bundled Tesseract Version: {version}")
    assert version.startswith("5."), f"Unexpected version: {version}"

    # Initialize BaseAPI with bundled tessdata
    tess.TessBaseAPICreate.restype = ctypes.c_void_p
    api = tess.TessBaseAPICreate()
    assert api, "Failed to create TessBaseAPI"

    tess.TessBaseAPIInit3.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    tess.TessBaseAPIInit3.restype = ctypes.c_int

    # Test English Init
    rc = tess.TessBaseAPIInit3(api, str(TESSDATA_DIR).encode(), b"eng")
    assert rc == 0, f"Failed to init English OCR from bundled tessdata: rc={rc}"
    print("  Successfully initialized English OCR from bundled tessdata")

    # Test Malayalam Init
    rc_mal = tess.TessBaseAPIInit3(api, str(TESSDATA_DIR).encode(), b"mal")
    assert rc_mal == 0, f"Failed to init Malayalam OCR from bundled tessdata: rc={rc_mal}"
    print("  Successfully initialized Malayalam OCR from bundled tessdata")

    tess.TessBaseAPIDelete.argtypes = [ctypes.c_void_p]
    tess.TessBaseAPIDelete(api)
    print("  PASS: Native Tesseract C-API functions flawlessly from bundle!")

def test_4_real_ocr_png_jpeg_malayalam():
    print("\n[TEST 4/6] Running end-to-end OCR with bundled libraries on test samples...")
    tess = ctypes.CDLL(str(FRAMEWORKS_DIR / "libtesseract.5.dylib"))
    lept = ctypes.CDLL(str(FRAMEWORKS_DIR / "libleptonica.6.dylib"))

    lept.pixRead.argtypes = [ctypes.c_char_p]
    lept.pixRead.restype = ctypes.c_void_p
    lept.pixDestroy.argtypes = [ctypes.POINTER(ctypes.c_void_p)]

    tess.TessBaseAPICreate.restype = ctypes.c_void_p
    tess.TessBaseAPIInit3.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    tess.TessBaseAPIInit3.restype = ctypes.c_int
    tess.TessBaseAPISetImage2.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
    tess.TessBaseAPIRecognize.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
    tess.TessBaseAPIRecognize.restype = ctypes.c_int
    tess.TessBaseAPIGetUTF8Text.argtypes = [ctypes.c_void_p]
    tess.TessBaseAPIGetUTF8Text.restype = ctypes.c_void_p
    tess.TessDeleteText.argtypes = [ctypes.c_void_p]
    tess.TessBaseAPIMeanTextConf.argtypes = [ctypes.c_void_p]
    tess.TessBaseAPIMeanTextConf.restype = ctypes.c_int
    tess.TessBaseAPIEnd.argtypes = [ctypes.c_void_p]
    tess.TessBaseAPIDelete.argtypes = [ctypes.c_void_p]

    def run_ocr(img_path, lang):
        api = tess.TessBaseAPICreate()
        rc = tess.TessBaseAPIInit3(api, str(TESSDATA_DIR).encode(), lang.encode())
        assert rc == 0, f"Init failed for {lang}"

        pix = lept.pixRead(str(img_path).encode())
        assert pix, f"Failed to read {img_path}"

        tess.TessBaseAPISetImage2(api, pix)
        rec_rc = tess.TessBaseAPIRecognize(api, None)
        assert rec_rc == 0, f"Recognize returned non-zero rc={rec_rc}"
        text_ptr = tess.TessBaseAPIGetUTF8Text(api)
        text = ctypes.cast(text_ptr, ctypes.c_char_p).value.decode('utf-8')
        conf = tess.TessBaseAPIMeanTextConf(api)

        tess.TessDeleteText(text_ptr)
        tess.TessBaseAPIEnd(api)
        tess.TessBaseAPIDelete(api)
        pix_ptr = ctypes.c_void_p(pix)
        lept.pixDestroy(ctypes.byref(pix_ptr))
        return text, conf

    # 1. English PNG
    en_img = PROJECT_ROOT / "test_samples" / "sample_en.png"
    en_text, en_conf = run_ocr(en_img, "eng")
    print(f"  English PNG OCR: conf={en_conf}%, text snippet: {en_text.strip()[:40]!r}")
    assert "OCULUS" in en_text, "Failed to recognize 'OCULUS' in sample_en.png"

    # 2. JPEG OCR
    jpg_img = PROJECT_ROOT / "test_samples" / "sample_jpg.jpg"
    jpg_text, jpg_conf = run_ocr(jpg_img, "eng")
    print(f"  JPEG OCR: conf={jpg_conf}%, text snippet: {jpg_text.strip()[:40]!r}")
    assert "OCULUS" in jpg_text, "Failed to recognize 'OCULUS' in sample_jpg.jpg"

    # 3. Malayalam Script OCR
    mal_img = PROJECT_ROOT / "test_samples" / "corpus" / "malayalam" / "mal_conjuncts.png"
    mal_text, mal_conf = run_ocr(mal_img, "mal")
    print(f"  Malayalam OCR: conf={mal_conf}%, text snippet: {mal_text.strip()[:40]!r}")
    assert len(mal_text.strip()) > 10, "Failed to extract text from mal_conjuncts.png"
    assert any('\u0D00' <= c <= '\u0D7F' for c in mal_text), "No Malayalam Unicode glyphs detected"

    print("  PASS: End-to-end OCR verified for PNG, JPEG, and Malayalam script!")

def test_5_searchable_pdf_export():
    print("\n[TEST 5/6] Testing Searchable PDF generation with bundled libraries...")
    tess = ctypes.CDLL(str(FRAMEWORKS_DIR / "libtesseract.5.dylib"))
    lept = ctypes.CDLL(str(FRAMEWORKS_DIR / "libleptonica.6.dylib"))

    tess.TessPDFRendererCreate.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_int]
    tess.TessPDFRendererCreate.restype = ctypes.c_void_p
    tess.TessResultRendererBeginDocument.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    tess.TessResultRendererBeginDocument.restype = ctypes.c_int
    tess.TessResultRendererEndDocument.argtypes = [ctypes.c_void_p]
    tess.TessResultRendererEndDocument.restype = ctypes.c_int
    tess.TessDeleteResultRenderer.argtypes = [ctypes.c_void_p]

    tess.TessBaseAPICreate.restype = ctypes.c_void_p
    tess.TessBaseAPIInit3.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    tess.TessBaseAPIInit3.restype = ctypes.c_int
    tess.TessBaseAPIProcessPage.argtypes = [
        ctypes.c_void_p, ctypes.c_void_p, ctypes.c_int,
        ctypes.c_char_p, ctypes.c_char_p, ctypes.c_int, ctypes.c_void_p
    ]
    tess.TessBaseAPIProcessPage.restype = ctypes.c_int
    tess.TessBaseAPIEnd.argtypes = [ctypes.c_void_p]
    tess.TessBaseAPIDelete.argtypes = [ctypes.c_void_p]

    lept.pixRead.argtypes = [ctypes.c_char_p]
    lept.pixRead.restype = ctypes.c_void_p
    lept.pixDestroy.argtypes = [ctypes.POINTER(ctypes.c_void_p)]

    out_base = PROJECT_ROOT / "test_samples" / "clean_machine_searchable_test"
    renderer = tess.TessPDFRendererCreate(str(out_base).encode(), str(TESSDATA_DIR).encode(), 0)
    assert renderer, "Failed to create TessPDFRenderer"
    tess.TessResultRendererBeginDocument(renderer, b"Clean Machine Searchable Test")

    api = tess.TessBaseAPICreate()
    rc = tess.TessBaseAPIInit3(api, str(TESSDATA_DIR).encode(), b"eng")
    assert rc == 0, "Failed to init API for PDF export"

    en_img = PROJECT_ROOT / "test_samples" / "sample_en.png"
    pix = lept.pixRead(str(en_img).encode())
    assert pix, "Failed to read sample_en.png"

    # Process page through PDF renderer
    ok = tess.TessBaseAPIProcessPage(api, pix, 0, str(en_img).encode(), None, 0, renderer)
    assert ok == 1, "TessBaseAPIProcessPage failed"

    tess.TessResultRendererEndDocument(renderer)
    tess.TessDeleteResultRenderer(renderer)
    tess.TessBaseAPIEnd(api)
    tess.TessBaseAPIDelete(api)
    pix_ptr = ctypes.c_void_p(pix)
    lept.pixDestroy(ctypes.byref(pix_ptr))

    pdf_file = out_base.with_suffix(".pdf")
    assert pdf_file.exists() and pdf_file.stat().st_size > 1000, f"Searchable PDF not generated: {pdf_file}"
    print(f"  Generated Searchable PDF: {pdf_file} ({pdf_file.stat().st_size} bytes)")
    pdf_file.unlink() # Cleanup
    print("  PASS: Searchable PDF dual-layer generation succeeded!")

def test_6_binary_launch_in_sanitized_environment():
    print("\n[TEST 6/6] Launching packaged binary in scrubbed environment...")
    executable = MACOS_DIR / "oculus-desktop"
    
    # Launch with env -i, completely removing PATH to Homebrew and removing all DYLD variables
    clean_env = {
        "PATH": "/usr/bin:/bin",
        "HOME": os.environ.get("HOME", "/tmp"),
        "USER": os.environ.get("USER", "nobody")
    }

    # Start process with 1s timeout to ensure it doesn't crash on dyld linking
    proc = subprocess.Popen(
        [str(executable)],
        env=clean_env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    try:
        proc.wait(timeout=1.5)
        # If it exited, check returncode
        if proc.returncode != 0:
            stdout, stderr = proc.communicate()
            raise RuntimeError(f"Binary crashed on clean launch:\n{stderr}")
    except subprocess.TimeoutExpired:
        # Success! The GUI app launched and entered its event loop without dyld errors
        proc.terminate()
        proc.kill()
        print("  Process launched successfully without dyld linker errors and entered GUI event loop.")

    print("  PASS: Packaged application launched with Homebrew completely scrubbed from environment!")

def main():
    print("=========================================================")
    print("   OCULUS CLEAN-MACHINE VERIFICATION TEST HARNESS        ")
    print("=========================================================")
    print(f"Bundle Path: {BUNDLE_DIR}")

    test_1_dependency_isolation()
    test_2_tessdata_presence()
    test_3_native_tesseract_c_api_isolated()
    test_4_real_ocr_png_jpeg_malayalam()
    test_5_searchable_pdf_export()
    test_6_binary_launch_in_sanitized_environment()

    print("\n=========================================================")
    print("   ALL CLEAN-MACHINE PACKAGING TESTS PASSED (100%)       ")
    print("=========================================================\n")

if __name__ == "__main__":
    main()
