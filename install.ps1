$ErrorActionPreference = "Stop"

$repo = "d4rkNinja/infynon-cli"
$homeDir = [Environment]::GetFolderPath("UserProfile")
if (-not $homeDir) {
    $homeDir = $env:USERPROFILE
}
if (-not $homeDir) {
    throw "[infynon] Could not resolve the user profile directory."
}

$installDir = Join-Path $homeDir ".infynon\bin"
$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture

if ($arch -ne "X64") {
    throw "[infynon] Unsupported Windows architecture: $arch"
}

try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
} catch {}

function Invoke-InfynonDownload {
    param(
        [Parameter(Mandatory = $true)][string]$Uri,
        [Parameter(Mandatory = $true)][string]$OutFile
    )

    Invoke-WebRequest -Uri $Uri -OutFile $OutFile -Headers @{ "User-Agent" = "infynon-installer" }
}

function Get-ExpectedChecksum {
    param(
        [Parameter(Mandatory = $true)][string]$ChecksumsPath,
        [Parameter(Mandatory = $true)][string]$AssetName
    )

    foreach ($line in Get-Content -LiteralPath $ChecksumsPath) {
        if ($line -match '^([A-Fa-f0-9]{64})\s+[* ]?(.+)$') {
            $name = Split-Path -Leaf $Matches[2].Trim()
            if ($name -eq $AssetName) {
                return $Matches[1].ToLowerInvariant()
            }
        }
    }

    throw "[infynon] checksums.txt does not include $AssetName."
}

function Test-PathEntry {
    param(
        [Parameter(Mandatory = $true)][string]$PathValue,
        [Parameter(Mandatory = $true)][string]$Entry
    )

    $trimChars = [char[]]@([char]'\', [char]'/')
    $normalizedEntry = $Entry.Trim().TrimEnd($trimChars)
    foreach ($candidate in ($PathValue -split ';')) {
        if ($candidate.Trim().TrimEnd($trimChars) -ieq $normalizedEntry) {
            return $true
        }
    }

    return $false
}

$target = "x86_64-pc-windows-msvc"
$assetName = "infynon-$target.exe"
$release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest" -Headers @{ "User-Agent" = "infynon-installer" }
$tag = $release.tag_name
if (-not $tag) {
    throw "[infynon] Could not determine the latest release tag."
}

$url = "https://github.com/$repo/releases/download/$tag/$assetName"
$checksumsUrl = "https://github.com/$repo/releases/download/$tag/checksums.txt"

New-Item -ItemType Directory -Force -Path $installDir | Out-Null

$binPath = Join-Path $installDir "infynon.exe"
$pkgPath = Join-Path $installDir "infynon-pkg.exe"
$tempId = [Guid]::NewGuid().ToString("N")
$tempBinary = Join-Path $installDir "infynon.exe.download-$tempId"
$tempChecksums = Join-Path $installDir "checksums.txt.download-$tempId"

try {
    Invoke-InfynonDownload -Uri $url -OutFile $tempBinary
    Invoke-InfynonDownload -Uri $checksumsUrl -OutFile $tempChecksums

    $expected = Get-ExpectedChecksum -ChecksumsPath $tempChecksums -AssetName $assetName
    $actual = (Get-FileHash -LiteralPath $tempBinary -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) {
        throw "[infynon] SHA-256 mismatch for $assetName."
    }

    Move-Item -LiteralPath $tempBinary -Destination $binPath -Force
    Copy-Item -LiteralPath $binPath -Destination $pkgPath -Force
} finally {
    Remove-Item -LiteralPath $tempBinary -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $tempChecksums -Force -ErrorAction SilentlyContinue
}

$expectedVersion = $tag -replace '^v', ''
$reportedVersion = & $binPath --version 2>&1
if ($LASTEXITCODE -ne 0) {
    throw "[infynon] Installed binary failed to run: $reportedVersion"
}
$versionFields = $reportedVersion -split '\s+' | ForEach-Object { $_ -replace '^v', '' }
if ($versionFields -notcontains $expectedVersion) {
    throw "[infynon] Installed binary did not report version $expectedVersion."
}

$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($null -eq $userPath) {
    $userPath = ""
}

if (-not (Test-PathEntry -PathValue $userPath -Entry $installDir)) {
    $newUserPath = if ([string]::IsNullOrWhiteSpace($userPath)) { $installDir } else { "$installDir;$userPath" }
    [Environment]::SetEnvironmentVariable("PATH", $newUserPath, "User")
}

Write-Host "[infynon] Installed $tag to $binPath"
