<#
.SYNOPSIS
  Build the OCULUS NSIS installer for Windows (x64).

.DESCRIPTION
  scripts\release_windows.ps1

  Needs: Visual Studio Build Tools (C++), Rust (x86_64-pc-windows-msvc), bun, and vcpkg with
  "tesseract[core]:x64-windows-static-md" installed (set VCPKG_ROOT), plus `bun install` in
  araskova-doccraft\. Output: dist-release\OCULUS_<version>_x64-setup.exe (+ .sha256).
  Windows ships without native PDF-page OCR for now (see RELEASING.md).
#>
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Root = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $Root

$LogDir = Join-Path $Root 'build-logs'
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
Start-Transcript -Path (Join-Path $LogDir 'release_windows.log') -Force | Out-Null

try {
  $Version = (Get-Content 'src-tauri\tauri.conf.json' -Raw | ConvertFrom-Json).version
  Write-Host "==> Building OCULUS $Version (x64)"

  # Static Tesseract/Leptonica from vcpkg (see src-tauri/build.rs).
  if (-not $env:VCPKG_ROOT) {
    if ($env:VCPKG_INSTALLATION_ROOT) { $env:VCPKG_ROOT = $env:VCPKG_INSTALLATION_ROOT }
    else { throw 'VCPKG_ROOT is not set. Install vcpkg and: vcpkg install "tesseract[core]:x64-windows-static-md"' }
  }
  if (-not $env:VCPKGRS_TRIPLET) { $env:VCPKGRS_TRIPLET = 'x64-windows-static-md' }
  Write-Host "    VCPKG_ROOT=$env:VCPKG_ROOT  VCPKGRS_TRIPLET=$env:VCPKGRS_TRIPLET"

  # Language data must exist; tauri.windows.conf.json copies it to <install dir>\tessdata.
  foreach ($f in 'eng.traineddata', 'mal.traineddata', 'osd.traineddata', 'pdf.ttf') {
    if (-not (Test-Path (Join-Path $Root "tessdata\$f"))) { throw "missing tessdata\$f" }
  }

  # Keep the build machine's folder names out of the shipped binary.
  $env:RUSTFLAGS = "$env:RUSTFLAGS --remap-path-prefix=$Root=oculus --remap-path-prefix=$env:USERPROFILE=~".Trim()
  $env:CC_ENABLE_DEBUG_OUTPUT = '1'   # cc-rs prints the exact cl.exe command lines

  # The Tauri CLI comes from the UI submodule's node_modules (bun creates .exe or .cmd shims).
  $Bin = Join-Path $Root 'araskova-doccraft\node_modules\.bin'
  $Tauri = @('tauri.cmd', 'tauri.exe', 'tauri') | ForEach-Object { Join-Path $Bin $_ } |
    Where-Object { Test-Path $_ } | Select-Object -First 1
  if (-not $Tauri) {
    Write-Host "Contents of ${Bin}:"; Get-ChildItem $Bin -ErrorAction SilentlyContinue | ForEach-Object Name
    throw "Tauri CLI not found in $Bin. Run: cd araskova-doccraft; bun install --frozen-lockfile"
  }
  Write-Host "    tauri CLI: $Tauri"

  Push-Location (Join-Path $Root 'src-tauri')
  try {
    & $Tauri build --bundles nsis --verbose
    if ($LASTEXITCODE -ne 0) { throw "tauri build failed with exit code $LASTEXITCODE" }
  } finally { Pop-Location }

  $Setup = Get-ChildItem (Join-Path $Root 'target\release\bundle\nsis') -Filter '*-setup.exe' -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
  if (-not $Setup) { throw 'expected an NSIS *-setup.exe under target\release\bundle\nsis was not built' }

  $Out = Join-Path $Root 'dist-release'
  New-Item -ItemType Directory -Force -Path $Out | Out-Null
  $Dest = Join-Path $Out "OCULUS_${Version}_x64-setup.exe"
  Copy-Item $Setup.FullName $Dest -Force
  $Hash = (Get-FileHash $Dest -Algorithm SHA256).Hash.ToLower()
  # Same "<hash>  <file>" format as shasum, so `sha256sum -c` works.
  [IO.File]::WriteAllText("$Dest.sha256", "$Hash  $(Split-Path $Dest -Leaf)`n")

  Write-Host "==> Done: $Dest"
} finally {
  Stop-Transcript | Out-Null
}
