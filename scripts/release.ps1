param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$')]
    [string]$Version,

    [string]$Remote = "origin",
    [string]$Branch = "main",
    [string]$CommitMessage,
    [switch]$SkipTests,
    [switch]$NoPush,
    [switch]$IncludeAllChanges,
    [string[]]$TagNote
)

$ErrorActionPreference = "Stop"

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [Parameter(Mandatory = $true)]
        [scriptblock]$Command
    )

    Write-Host ""
    Write-Host "==> $Name" -ForegroundColor Cyan
    $global:LASTEXITCODE = 0
    & $Command
    if ($global:LASTEXITCODE -ne 0) {
        throw "Step '$Name' failed with exit code $global:LASTEXITCODE"
    }
}

function Assert-CleanWorktree {
    $status = git status --porcelain
    if ($status) {
        throw "Worktree is not clean. Commit existing changes first, or rerun with -IncludeAllChanges to include them in the release commit."
    }
}

function Get-GitRemoteUrl {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Remote
    )

    $url = git remote get-url $Remote
    if ($global:LASTEXITCODE -ne 0 -or -not $url) {
        throw "Git remote '$Remote' is not configured."
    }
    return $url.Trim()
}

function Assert-SourceRemote {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Remote,
        [Parameter(Mandatory = $true)]
        [string]$Branch
    )

    $upstream = git rev-parse --abbrev-ref --symbolic-full-name "@{u}" 2>$null
    if ($global:LASTEXITCODE -eq 0 -and $upstream -and $upstream.Trim() -ne "$Remote/$Branch") {
        throw "Branch '$Branch' tracks '$($upstream.Trim())'. Set it to '$Remote/$Branch' before releasing."
    }
}

function Set-Utf8NoBomText {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [string]$Text
    )

    $fullPath = [System.IO.Path]::GetFullPath((Join-Path (Get-Location) $Path))
    $encoding = [System.Text.UTF8Encoding]::new($false)
    [System.IO.File]::WriteAllText($fullPath, $Text, $encoding)
}

function Remove-ReleaseScratchFiles {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Version
    )

    $npmTarball = Join-Path "npm" "infynon-$Version.tgz"
    if (Test-Path -LiteralPath $npmTarball) {
        Remove-Item -LiteralPath $npmTarball -Force
    }

    $pycache = Join-Path "scripts" "__pycache__"
    if (Test-Path -LiteralPath $pycache) {
        Remove-Item -LiteralPath $pycache -Recurse -Force
    }
}

function Set-TextFileVersion {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [string]$Pattern,
        [Parameter(Mandatory = $true)]
        [string]$Replacement
    )

    $text = Get-Content -LiteralPath $Path -Raw
    $regex = [regex]::new($Pattern, [System.Text.RegularExpressions.RegexOptions]::Multiline)
    $updated = $regex.Replace($text, $Replacement, 1)
    if ($updated -eq $text) {
        if ($text.Contains($Replacement)) {
            return
        }
        throw "No version replacement was made in $Path"
    }
    Set-Utf8NoBomText -Path $Path -Text $updated
}

$tag = "v$Version"
$defaultCommitMessage = "chore(release): prepare $Version"
if (-not $CommitMessage) {
    $CommitMessage = $defaultCommitMessage
}

Invoke-Step "Check repository state" {
    $currentBranch = git branch --show-current
    if ($currentBranch -ne $Branch) {
        throw "Expected branch '$Branch', found '$currentBranch'"
    }

    Assert-SourceRemote -Remote $Remote -Branch $Branch
    if (-not $IncludeAllChanges) {
        Assert-CleanWorktree
    }

    if (git tag -l $tag) {
        throw "Tag $tag already exists locally"
    }
}

Invoke-Step "Update release metadata" {
    Set-TextFileVersion -Path "Cargo.toml" -Pattern '^version = "([^"]+)"' -Replacement "version = `"$Version`""

    $packageJsonPath = "npm/package.json"
    $packageJson = Get-Content -LiteralPath $packageJsonPath -Raw | ConvertFrom-Json
    $packageJson.version = $Version
    foreach ($dependencyName in @($packageJson.optionalDependencies.PSObject.Properties.Name)) {
        if ($dependencyName -like "infynon-*") {
            $packageJson.optionalDependencies.$dependencyName = $Version
        }
    }
    Set-Utf8NoBomText -Path $packageJsonPath -Text (($packageJson | ConvertTo-Json -Depth 20) + [Environment]::NewLine)

    $platformPackagePaths = @(
        "npm/platforms/cli-darwin-arm64/package.json",
        "npm/platforms/cli-darwin-x64/package.json",
        "npm/platforms/cli-linux-arm64/package.json",
        "npm/platforms/cli-linux-x64/package.json",
        "npm/platforms/cli-windows-x64/package.json"
    )
    foreach ($platformPackagePath in $platformPackagePaths) {
        $platformPackage = Get-Content -LiteralPath $platformPackagePath -Raw | ConvertFrom-Json
        $platformPackage.version = $Version
        Set-Utf8NoBomText -Path $platformPackagePath -Text (($platformPackage | ConvertTo-Json -Depth 20) + [Environment]::NewLine)
    }

    Set-TextFileVersion `
        -Path "go/internal/installer/installer.go" `
        -Pattern '^\s*version\s*=\s*"([^"]+)"' `
        -Replacement "`tversion = `"$Version`""
}

Invoke-Step "Verify release metadata" {
    python scripts/verify-release-versions.py $tag
}

if (-not $SkipTests) {
    Invoke-Step "Run Rust checks" {
        cargo check
        cargo test
    }

    Invoke-Step "Run Go checks" {
        Push-Location go
        try {
            go test ./...
        } finally {
            Pop-Location
        }
    }

    Invoke-Step "Validate npm package" {
        Push-Location npm
        try {
            npm pack
            $tarball = "infynon-$Version.tgz"
            if (Test-Path $tarball) {
                Remove-Item $tarball -Force
            }
        } finally {
            Pop-Location
        }
    }
}

Invoke-Step "Commit release metadata" {
    Remove-ReleaseScratchFiles -Version $Version
    if ($IncludeAllChanges) {
        git add -A
    } else {
        git add Cargo.toml Cargo.lock npm/package.json go/internal/installer/installer.go npm/platforms/cli-darwin-arm64/package.json npm/platforms/cli-darwin-x64/package.json npm/platforms/cli-linux-arm64/package.json npm/platforms/cli-linux-x64/package.json npm/platforms/cli-windows-x64/package.json
    }
    git commit -m $CommitMessage
}

Invoke-Step "Create annotated tag" {
    $tagArgs = @("-a", $tag, "-m", "Release $tag")
    foreach ($note in $TagNote) {
        if ($note) {
            $tagArgs += @("-m", $note)
        }
    }
    git tag @tagArgs
}

if (-not $NoPush) {
    Invoke-Step "Push branch and tag" {
        git push $Remote $Branch
        git push $Remote $tag
    }

    Write-Host ""
    $remoteUrl = Get-GitRemoteUrl -Remote $Remote
    Write-Host "Release $tag pushed to $Remote ($remoteUrl)." -ForegroundColor Green
    Write-Host "GitHub Actions will build binaries, publish GitHub Releases, and publish npm." -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "Release $tag prepared locally. Push $Branch and $tag when ready." -ForegroundColor Yellow
}
