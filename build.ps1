# build.ps1 — Build Aurotype for Windows
# Usage: powershell -ExecutionPolicy Bypass -File build.ps1
#
# Prerequisites:
#   - Rust toolchain (rustup)
#   - Node.js + bun (or npm)
#   - Python 3.12+ with uv
#   - PyInstaller (installed via: cd engine && uv sync --group dev)

$ErrorActionPreference = "Stop"

# Detect target triple
$triple = (rustc -vV | Select-String "host:").ToString().Split(" ")[1]
Write-Host "[build] Target triple: $triple"

# Step 1: Build Python engine with PyInstaller
Write-Host "[build] Building Python engine..."
Push-Location engine
uv run pyinstaller aurotype-engine.spec --noconfirm
Pop-Location

# Step 2: Copy binary to Tauri sidecar location
$binDir = "src-tauri\binaries"
if (-not (Test-Path $binDir)) { New-Item -ItemType Directory -Path $binDir | Out-Null }

$src = "engine\dist\aurotype-engine.exe"
$dst = "$binDir\aurotype-engine-$triple.exe"
Write-Host "[build] Copying $src -> $dst"
Copy-Item $src $dst -Force

# Step 3: Build Tauri app
Write-Host "[build] Building Tauri app..."
bun run tauri build

Write-Host "[build] Done! Check src-tauri\target\release\bundle\ for the installer."
