<#
.SYNOPSIS
    Builds the Windows installer and stages everything needed for a GitHub release.

.DESCRIPTION
    Checks the version numbers, then runs a release build, copies the NSIS
    installer into release/v<version>/ and writes SHA256SUMS.txt next to it.

    The version comes from scripts/version.ps1, which reads the authoritative
    number out of src-tauri/tauri.conf.json and refuses to continue when
    package.json, package-lock.json, Cargo.toml or Cargo.lock disagree.

    RELEASE_NOTES.md is extracted from the matching section of CHANGELOG.md, unless the
    file already exists - hand-written notes are never overwritten without -ForceNotes.

.PARAMETER SkipBuild
    Skip the build and re-stage whatever is already in src-tauri/target/release/bundle.

.PARAMETER ForceNotes
    Regenerate RELEASE_NOTES.md from CHANGELOG.md even if the file already exists.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/release.ps1
    powershell -ExecutionPolicy Bypass -File scripts/release.ps1 -SkipBuild
#>
[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [switch]$ForceNotes
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot

function Get-ChangelogSection {
    param([string]$Version)

    $path = Join-Path $root 'CHANGELOG.md'
    $lines = @(Get-Content -LiteralPath $path -Encoding UTF8)
    $pattern = '^## \[' + [regex]::Escape($Version) + '\]'
    $start = -1
    $end = $lines.Count

    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($start -lt 0) {
            if ($lines[$i] -match $pattern) { $start = $i }
            continue
        }
        if ($lines[$i] -match '^## \[') { $end = $i; break }
    }

    if ($start -lt 0) { return $null }
    return (($lines[($start + 1)..($end - 1)]) -join [Environment]::NewLine).Trim()
}

function Stop-RunningApp {
    $procs = @(Get-Process -Name glossy -ErrorAction SilentlyContinue)
    if ($procs.Count -eq 0) { return }

    $ids = ($procs | ForEach-Object { $_.Id }) -join ', '
    Write-Host "Stopping running Glossy ($ids) so the linker can replace the binary." -ForegroundColor Yellow
    foreach ($p in $procs) { Stop-Process -Id $p.Id -Force }
    Start-Sleep -Seconds 1
}

$version = & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'version.ps1') -Get
if ($LASTEXITCODE -ne 0 -or -not $version) {
    throw 'The version numbers do not agree; run scripts\version.ps1 to see which file is off.'
}
$version = ([string]$version).Trim()

Write-Host "Version: $version" -ForegroundColor Cyan

$stage = Join-Path $root ('release\v' + $version)
New-Item -ItemType Directory -Path $stage -Force | Out-Null

if (-not $SkipBuild) {
    Stop-RunningApp

    Write-Host 'Building the release bundle...' -ForegroundColor Cyan
    $build = 'cd /d "' + $root + '" && set PATH=%USERPROFILE%\.cargo\bin;%PATH% && npx tauri build'
    & $env:ComSpec /c $build
    if ($LASTEXITCODE -ne 0) { throw "The release build failed with exit code $LASTEXITCODE." }
}

$bundleDir = Join-Path $root 'src-tauri\target\release\bundle\nsis'
# Only what this version built: an installer left behind by an earlier build sits
# in the same directory and must not be staged into the new release folder.
$installers = @(Get-ChildItem -LiteralPath $bundleDir -Filter '*-setup.exe' -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -like "Glossy_$($version)_*" })
if ($installers.Count -eq 0) {
    throw "No installer for $version found in $bundleDir. Run without -SkipBuild first."
}

$staged = @()
foreach ($installer in $installers) {
    Copy-Item -LiteralPath $installer.FullName -Destination $stage -Force
    $staged += (Join-Path $stage $installer.Name)
    Write-Host "Staged $($installer.Name)" -ForegroundColor Green
}

$sums = foreach ($file in $staged) {
    $hash = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $(Split-Path -Leaf $file)"
}
$eol = [Environment]::NewLine
[IO.File]::WriteAllText((Join-Path $stage 'SHA256SUMS.txt'), (($sums -join $eol) + $eol), (New-Object Text.UTF8Encoding($false)))

$notesPath = Join-Path $stage 'RELEASE_NOTES.md'
if ((Test-Path -LiteralPath $notesPath) -and -not $ForceNotes) {
    Write-Host 'Keeping the existing RELEASE_NOTES.md (-ForceNotes would regenerate it).' -ForegroundColor Yellow
} else {
    $section = Get-ChangelogSection -Version $version
    if (-not $section) {
        Write-Warning "CHANGELOG.md has no '## [$version]' section - write RELEASE_NOTES.md by hand."
    } else {
        [IO.File]::WriteAllText($notesPath, ($section + $eol), (New-Object Text.UTF8Encoding($false)))
        Write-Host 'Wrote RELEASE_NOTES.md from CHANGELOG.md' -ForegroundColor Green
    }
}

Write-Host ''
Write-Host "Ready in $stage" -ForegroundColor Cyan
Get-ChildItem -LiteralPath $stage -File | ForEach-Object {
    '  {0,-34} {1,10:N0} bytes' -f $_.Name, $_.Length
}
Write-Host ''
Write-Host 'Upload: create the tag, then the release, paste RELEASE_NOTES.md into the'
Write-Host 'description and attach the installer together with SHA256SUMS.txt.'
