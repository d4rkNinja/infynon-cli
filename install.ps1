$ErrorActionPreference = "Stop"

$repo = "d4rkNinja/infynon-cli"
$installDir = "$env:USERPROFILE\.infynon\bin"
$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture

if ($arch -ne "X64") {
    throw "[infynon] Unsupported Windows architecture: $arch"
}

$target = "x86_64-pc-windows-msvc"
$release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
$tag = $release.tag_name
if (-not $tag) {
    throw "[infynon] Could not determine the latest release tag."
}

$url = "https://github.com/$repo/releases/download/$tag/infynon-$target.exe"
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Invoke-WebRequest -Uri $url -OutFile "$installDir\infynon.exe"
Copy-Item "$installDir\infynon.exe" "$installDir\infynon-pkg.exe" -Force

$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$installDir;$userPath", "User")
}

Write-Host "[infynon] Installed $tag to $installDir\infynon.exe"

