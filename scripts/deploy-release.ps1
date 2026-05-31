#!/usr/bin/env pwsh

<#
.SYNOPSIS
Deploy an INFYNON release tag using the repository release guardrails.

.DESCRIPTION
This script is intentionally conservative. It does not edit release versions.
Update Cargo/npm/Go metadata first, then run this script. It verifies the
metadata, pushes main, creates or reuses the annotated tag, pushes the tag, and
starts or watches the Release workflow.

Examples:
  powershell -ExecutionPolicy Bypass -File scripts\deploy-release.ps1
  powershell -ExecutionPolicy Bypass -File scripts\deploy-release.ps1 -Version 0.2.8
  powershell -ExecutionPolicy Bypass -File scripts\deploy-release.ps1 -Tag v0.2.8 -Dispatch
  powershell -ExecutionPolicy Bypass -File scripts\deploy-release.ps1 -Tag v0.2.8 -Dispatch -NoWatch
  powershell -ExecutionPolicy Bypass -File scripts\deploy-release.ps1 -Version 0.2.8 -DryRun
#>

[CmdletBinding()]
param(
    [string]$Version,
    [string]$Tag,
    [string]$Remote = "origin",
    [string]$Repo,
    [string]$Branch = "main",
    [switch]$Dispatch,
    [switch]$NoWatch,
    [switch]$DryRun,
    [int]$WatchIntervalSeconds = 20
)

$ErrorActionPreference = "Stop"

function Fail([string]$Message) {
    Write-Error $Message
    exit 1
}

function Run([string]$File, [string[]]$Arguments, [switch]$AllowFailure) {
    Write-Host "> $File $($Arguments -join ' ')"
    & $File @Arguments
    $code = $LASTEXITCODE
    if (($code -ne 0) -and (-not $AllowFailure)) {
        throw "Command failed with exit code ${code}: $File $($Arguments -join ' ')"
    }
    if ($AllowFailure) {
        return $code
    }
}

function Command-Succeeds([string]$File, [string[]]$Arguments) {
    & $File @Arguments *> $null
    return ($LASTEXITCODE -eq 0)
}

function Require-Command([string]$Name) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Fail "Required command not found on PATH: $Name"
    }
}

function GitOutput([string[]]$Arguments) {
    $output = & git @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "git $($Arguments -join ' ') failed"
    }
    return ($output | Out-String).Trim()
}

function GhJson([string[]]$Arguments) {
    $json = & gh @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "gh $($Arguments -join ' ') failed"
    }
    if (-not $json) {
        return $null
    }
    return ($json | Out-String | ConvertFrom-Json)
}

function Current-Version() {
    $text = Get-Content -Raw -Path "Cargo.toml"
    $match = [regex]::Match($text, '(?m)^version = "([^"]+)"')
    if (-not $match.Success) {
        Fail "Could not parse package version from Cargo.toml"
    }
    return $match.Groups[1].Value
}

function Validate-Tag([string]$ReleaseTag) {
    if ($ReleaseTag -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$') {
        Fail "Release tag must be v-prefixed semver, for example v0.2.8. Got: $ReleaseTag"
    }
}

function Assert-Clean-Tree() {
    $dirty = GitOutput @("status", "--porcelain")
    if ($dirty) {
        Write-Host $dirty
        Fail "Working tree is not clean. Commit or stash changes before deploying."
    }
}

function Resolve-Repo() {
    if ($Repo) {
        return $Repo
    }

    $repoName = & gh repo view --json nameWithOwner --jq ".nameWithOwner" 2>$null
    if (($LASTEXITCODE -eq 0) -and $repoName) {
        return $repoName.Trim()
    }

    $url = GitOutput @("remote", "get-url", $Remote)
    if ($url -match 'github\.com[:/]([^/]+/[^/.]+)(\.git)?$') {
        return $Matches[1]
    }

    Fail "Could not resolve GitHub repo. Pass -Repo owner/name."
}

function Find-Release-Run([string]$RepoName, [string]$ReleaseTag, [string]$EventName, [datetime]$StartedAfter) {
    $runs = GhJson @(
        "run", "list",
        "--repo", $RepoName,
        "--workflow", "Release",
        "--limit", "20",
        "--json", "databaseId,event,headBranch,createdAt,status,conclusion,displayTitle"
    )

    if (-not $runs) {
        return $null
    }

    $matches = @($runs | Where-Object {
        ([datetime]$_.createdAt -ge $StartedAfter.AddMinutes(-2)) -and
        ($_.event -eq $EventName) -and
        (($_.headBranch -eq $ReleaseTag) -or ($_.displayTitle -eq "Release"))
    })

    if ($matches.Count -gt 0) {
        return $matches[0]
    }

    return $null
}

Require-Command "git"
Require-Command "gh"
Require-Command "python"

if (-not (Test-Path "Cargo.toml")) {
    Fail "Run this script from the repository root."
}

if ($Tag -and $Version) {
    Fail "Pass either -Tag or -Version, not both."
}

if (-not $Tag) {
    if (-not $Version) {
        $Version = Current-Version
    }
    $Tag = if ($Version.StartsWith("v")) { $Version } else { "v$Version" }
}

Validate-Tag $Tag
$Version = $Tag.Substring(1)
$RepoName = Resolve-Repo
$startedAt = Get-Date

Write-Host "INFYNON release deploy"
Write-Host "Repo:    $RepoName"
Write-Host "Branch:  $Branch"
Write-Host "Tag:     $Tag"
Write-Host "Version: $Version"
if ($DryRun) {
    Write-Host "Mode:    dry run"
}

Run "git" @("fetch", $Remote, $Branch)

$currentBranch = GitOutput @("branch", "--show-current")
if ($currentBranch -ne $Branch) {
    Fail "Current branch is '$currentBranch'. Switch to '$Branch' before deploying."
}

Assert-Clean-Tree

Run "python" @("scripts/verify-release-versions.py", $Tag)
Run "git" @("diff", "--check")
Run "gh" @("auth", "status")

$localHead = GitOutput @("rev-parse", "HEAD")
$remoteHead = GitOutput @("rev-parse", "$Remote/$Branch")
if ($localHead -ne $remoteHead) {
    if ($DryRun) {
        Write-Host "Would push $Branch to $Remote."
    }
    else {
        Write-Host "Local $Branch differs from $Remote/$Branch. Pushing branch first."
        Run "git" @("push", $Remote, $Branch)
    }
}
else {
    Write-Host "$Branch is already pushed."
}

$remoteTagExists = Command-Succeeds "git" @("ls-remote", "--exit-code", "--tags", $Remote, "refs/tags/$Tag")
if ($remoteTagExists) {
    Write-Host "Remote tag $Tag already exists. Using workflow_dispatch so the current workflow can rerun the release safely."
    $Dispatch = $true
}
else {
    $localTagExists = Command-Succeeds "git" @("rev-parse", "-q", "--verify", "refs/tags/$Tag")
    if ($localTagExists) {
        $tagCommit = GitOutput @("rev-list", "-n", "1", $Tag)
        if ($tagCommit -ne $localHead) {
            Fail "Local tag $Tag exists but points at $tagCommit, not HEAD $localHead. Refusing to move a release tag."
        }
        Write-Host "Local tag $Tag already points at HEAD."
    }
    else {
        if ($DryRun) {
            Write-Host "Would create annotated tag $Tag at HEAD."
        }
        else {
            Run "git" @("tag", "-a", $Tag, "-m", "Release $Tag")
        }
    }

    if ($DryRun) {
        Write-Host "Would push tag $Tag to $Remote."
    }
    else {
        Run "git" @("push", $Remote, $Tag)
    }
}

if ($Dispatch) {
    if ($DryRun) {
        Write-Host "Would trigger Release workflow_dispatch for $Tag on $Branch."
    }
    else {
        Run "gh" @("workflow", "run", "Release", "--repo", $RepoName, "--ref", $Branch, "-f", "release_tag=$Tag")
        Write-Host "Triggered Release workflow_dispatch for $Tag."
    }
    $eventName = "workflow_dispatch"
}
else {
    Write-Host "Tag push triggered the Release workflow for $Tag."
    $eventName = "push"
}

if ($DryRun) {
    Write-Host "Dry run completed. No tag, push, or workflow dispatch was executed."
    exit 0
}

if ($NoWatch) {
    Write-Host "Deploy submitted. Skipping watch because -NoWatch was provided."
    exit 0
}

Write-Host "Waiting for Release workflow run..."
$run = $null
for ($i = 0; $i -lt 30; $i++) {
    Start-Sleep -Seconds 5
    $run = Find-Release-Run $RepoName $Tag $eventName $startedAt
    if ($run) {
        break
    }
}

if (-not $run) {
    Fail "Could not find the Release workflow run. Check GitHub Actions for $RepoName."
}

Write-Host "Watching run $($run.databaseId)..."
Run "gh" @("run", "watch", "$($run.databaseId)", "--repo", $RepoName, "--exit-status", "--interval", "$WatchIntervalSeconds")

Write-Host "Release workflow completed successfully for $Tag."
