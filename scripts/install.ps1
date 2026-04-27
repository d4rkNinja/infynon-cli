# One-liner: irm https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.ps1 | iex
$ErrorActionPreference = "Stop"

$repo = "d4rkNinja/infynon-cli"
$installDir = "$env:USERPROFILE\.infynon\bin"

Write-Host ""
Write-Host "  INFYNON — Universal Package Security Manager — Installer" -ForegroundColor Cyan
Write-Host ""

# Detect arch
$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
$target = "x86_64-pc-windows-msvc"
if ($arch -ne "X64") {
    Write-Host "  [!!] Windows $arch is not supported by prebuilt releases. Install from source:" -ForegroundColor Yellow
    Write-Host "    cargo install --git https://github.com/$repo"
    exit 1
}

Write-Host "  Platform: Windows $arch -> $target" -ForegroundColor Gray

# Get latest release
Write-Host "  Fetching latest release..." -ForegroundColor Gray
try {
    $release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
    $tag = $release.tag_name
} catch {
    $tag = $null
}

if (-not $tag) {
    Write-Host "  [!!] Could not find release. Building from source..." -ForegroundColor Yellow
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargo) {
        Write-Host "  Installing Rust..." -ForegroundColor Yellow
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
        & "$env:TEMP\rustup-init.exe" -y
        $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    }
    cargo install --git "https://github.com/$repo"
    $cargoBin = "$env:USERPROFILE\.cargo\bin"
    if ((Test-Path "$cargoBin\infynon.exe") -and -not (Test-Path "$cargoBin\infynon-pkg.exe")) {
        Copy-Item "$cargoBin\infynon.exe" "$cargoBin\infynon-pkg.exe"
    }
    Write-Host ""
    Write-Host "  [OK] Installed! Run: infynon pkg scan" -ForegroundColor Green
    exit 0
}

Write-Host "  Latest release: $tag" -ForegroundColor Gray

# Download
$binary = "infynon-${target}.exe"
$url = "https://github.com/$repo/releases/download/$tag/$binary"
Write-Host "  Downloading $url ..." -ForegroundColor Gray

New-Item -ItemType Directory -Force -Path $installDir | Out-Null
try {
    Invoke-WebRequest -Uri $url -OutFile "$installDir\infynon.exe"
} catch {
    Write-Host "  [!!] Download failed. Building from source..." -ForegroundColor Yellow
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargo) {
        Write-Host "  Installing Rust..." -ForegroundColor Yellow
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
        & "$env:TEMP\rustup-init.exe" -y
        $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    }
    cargo install --git "https://github.com/$repo"
    $cargoBin = "$env:USERPROFILE\.cargo\bin"
    if (Test-Path "$cargoBin\infynon.exe") {
        Copy-Item "$cargoBin\infynon.exe" "$installDir\infynon.exe" -Force
    } else {
        throw "cargo install completed but infynon.exe was not found"
    }
}

# Create copy for infynon-pkg
Copy-Item "$installDir\infynon.exe" "$installDir\infynon-pkg.exe"

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$installDir;$userPath", "User")
    $env:PATH = "$installDir;$env:PATH"
    Write-Host "  [OK] Added $installDir to PATH" -ForegroundColor Green
}

Write-Host ""
Write-Host "  [OK] infynon $tag installed to $installDir\infynon.exe" -ForegroundColor Green
Write-Host "  [OK] infynon-pkg.exe -> copy created" -ForegroundColor Green
Write-Host ""
Write-Host "  Restart your terminal, then:" -ForegroundColor Yellow
Write-Host "    infynon pkg scan                  # scan project for CVEs"
Write-Host "    infynon pkg npm install express    # secure install"
Write-Host ""
