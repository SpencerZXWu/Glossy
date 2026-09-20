<#
.SYNOPSIS
    Builds the Windows installer and stages everything needed for a GitHub release.

.DESCRIPTION
    Checks the version numbers, then runs a release build, copies the NSIS
    installer into release/v<version>/ and writes SHA256SUMS.txt next to it.

    The version comes from scripts/version.ps1, which reads the authoritative
    number out of src-tauri/tauri.conf.json and refuses to continue when
    package.json, package-lock.json, Cargo.toml or Cargo.lock disagree.

    RELEASE_NOTES.md is written in three languages behind anchors - English, Chinese
    and Spanish - so the release description can be read without leaving the page and
    switched with the links at the top. The English section comes from the matching
    section of CHANGELOG.md, which stays English; the other two are filled in by hand
    and a placeholder left behind is called out before the release is published.

    An existing RELEASE_NOTES.md is never overwritten without -ForceNotes.

    With -Publish the script also pushes the tag and creates the GitHub release through
    the GitHub CLI, so a release is one command from a clean tree.

.PARAMETER SkipBuild
    Skip the build and re-stage whatever is already in src-tauri/target/release/bundle.

.PARAMETER ForceNotes
    Regenerate RELEASE_NOTES.md from CHANGELOG.md even if the file already exists. The
    translations are reset to their placeholders, so translations already written by
    hand are lost.

.PARAMETER Publish
    After staging, push the v<version> tag and create the GitHub release with the
    GitHub CLI instead of printing the manual steps. Refuses to publish while
    RELEASE_NOTES.md still holds an untranslated section.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/release.ps1
    powershell -ExecutionPolicy Bypass -File scripts/release.ps1 -SkipBuild
    powershell -ExecutionPolicy Bypass -File scripts/release.ps1 -SkipBuild -Publish
#>
[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [switch]$ForceNotes,
    [switch]$Publish
)

$ErrorActionPreference = 'Stop'

# Windows PowerShell 5.1 decodes a BOM-less script as ANSI, which would silently turn
# the Chinese and Spanish labels in New-NotesScaffold into mojibake inside the notes.
$self = [IO.File]::ReadAllBytes($MyInvocation.MyCommand.Path)
if ($self.Length -lt 3 -or $self[0] -ne 0xEF -or $self[1] -ne 0xBB -or $self[2] -ne 0xBF) {
    throw 'scripts\release.ps1 has to stay saved as UTF-8 with a BOM; re-save it that way.'
}

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

function New-NotesScaffold {
    <#
        Builds the trilingual skeleton: a switcher line, then every language behind
        its own anchor, in reading order. The English body comes from CHANGELOG.md;
        the translations are left as placeholders that release.ps1 reports.
    #>
    param([string]$Section)

    $todo = '<!-- TODO: translate the English section above, then delete this comment. -->'
    $parts = @(
        '[English](#en) · [中文](#zh-cn) · [Español](#es)',
        '',
        '<a id="en"></a>',
        '',
        '## English',
        '',
        $Section,
        '',
        '<a id="zh-cn"></a>',
        '',
        '## 中文',
        '',
        $todo,
        '',
        '<a id="es"></a>',
        '',
        '## Español',
        '',
        $todo
    )
    return ($parts -join [Environment]::NewLine)
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

# The GNU toolchain links WebView2Loader.dll dynamically, so the installer has to
# carry it next to glossy.exe. build.rs stages the file and bundle.resources ships
# it; catch a missing or unslotted resource here rather than in a user's error box.
$loader = Join-Path $root 'src-tauri\resources\WebView2Loader.dll'
if (-not (Test-Path -LiteralPath $loader)) {
    throw 'src-tauri\resources\WebView2Loader.dll was not staged by build.rs; the installer would not start.'
}

$nsi = Join-Path $root 'src-tauri\target\release\nsis\x64\installer.nsi'
if ((Test-Path -LiteralPath $nsi) -and
    -not (Select-String -LiteralPath $nsi -SimpleMatch 'oname=WebView2Loader.dll' -Quiet)) {
    throw 'The generated installer.nsi does not install WebView2Loader.dll; check bundle.resources in tauri.conf.json.'
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
        $notes = (New-NotesScaffold -Section $section) + $eol
        [IO.File]::WriteAllText($notesPath, $notes, (New-Object Text.UTF8Encoding($false)))
        Write-Host 'Wrote the trilingual RELEASE_NOTES.md skeleton' -ForegroundColor Green
        Write-Host '  translate the English section for the other two languages before publishing.' -ForegroundColor Yellow
    }
}

if ((Test-Path -LiteralPath $notesPath) -and
    (Select-String -LiteralPath $notesPath -SimpleMatch 'TODO: translate' -Quiet)) {
    Write-Warning 'RELEASE_NOTES.md still has an untranslated section; fill it in before publishing.'
}

Write-Host ''
Write-Host "Ready in $stage" -ForegroundColor Cyan
Get-ChildItem -LiteralPath $stage -File | ForEach-Object {
    '  {0,-34} {1,10:N0} bytes' -f $_.Name, $_.Length
}
Write-Host ''

function Publish-Release {
    param([string]$Version, [string]$Stage)

    $gh = Get-Command gh -ErrorAction SilentlyContinue |
        Select-Object -First 1 -ExpandProperty Source
    if (-not $gh) {
        # A fresh install is not on the PATH of the shell that started before it.
        $fallback = 'C:\Program Files\GitHub CLI\gh.exe'
        if (Test-Path -LiteralPath $fallback) { $gh = $fallback }
    }
    if (-not $gh) {
        throw 'The GitHub CLI (gh) was not found. Install it with: winget install --id GitHub.cli'
    }

    $notesPath = Join-Path $Stage 'RELEASE_NOTES.md'
    if (-not (Test-Path -LiteralPath $notesPath)) {
        throw "$notesPath is missing; the release description would be empty."
    }
    if (Select-String -LiteralPath $notesPath -SimpleMatch 'TODO: translate' -Quiet) {
        throw 'RELEASE_NOTES.md still has an untranslated section; all three languages have to be filled in before publishing.'
    }

    # gh and git report progress and failures on stderr, and PowerShell 5.1 turns
    # redirected native stderr into an error record, which is fatal under the
    # $ErrorActionPreference at the top of this script. They are therefore run with
    # Continue and judged by $LASTEXITCODE alone.
    $saved = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'

    & $gh auth status --hostname github.com *> $null
    if ($LASTEXITCODE -ne 0) {
        $ErrorActionPreference = $saved
        throw "gh is not signed in. Run: & '$gh' auth login --hostname github.com --git-protocol https --web"
    }

    $tag = "v$Version"
    Write-Host "Publishing $tag" -ForegroundColor Cyan

    Push-Location $root
    try {
        & git rev-parse -q --verify "refs/tags/$tag" *> $null
        if ($LASTEXITCODE -ne 0) {
            & git tag -a $tag -m "Glossy $Version"
            if ($LASTEXITCODE -ne 0) {
                $ErrorActionPreference = $saved
                throw "git tag $tag failed."
            }
        }

        & git push origin $tag
        if ($LASTEXITCODE -ne 0) {
            $ErrorActionPreference = $saved
            throw "git push origin $tag failed. Pushing from this machine needs the proxy: `$env:HTTPS_PROXY='http://127.0.0.1:7897'"
        }
    } finally {
        Pop-Location
    }

    # The notes are the description, never an asset, so every other staged file goes up.
    $assets = @(Get-ChildItem -LiteralPath $Stage -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -ne 'RELEASE_NOTES.md' } |
        ForEach-Object { $_.FullName })
    if ($assets.Count -eq 0) {
        $ErrorActionPreference = $saved
        throw "Nothing to attach in $Stage."
    }

    $ghArgs = @('release', 'create', $tag,
        '--title', "Glossy $Version",
        '--notes-file', $notesPath,
        '--verify-tag') + $assets
    & $gh @ghArgs
    $code = $LASTEXITCODE
    $ErrorActionPreference = $saved
    if ($code -ne 0) { throw "gh release create $tag failed with exit code $code." }

    Write-Host "Published release $tag with $($assets.Count) asset(s)." -ForegroundColor Green
}

if ($Publish) {
    Publish-Release -Version $version -Stage $stage
} else {
    Write-Host 'Publish: re-run with -Publish, or do it by hand - create the tag vX.Y.Z,'
    Write-Host 'then the release titled "Glossy X.Y.Z" (the tag keeps the v, the title does'
    Write-Host 'not), paste RELEASE_NOTES.md into the description and attach the installer'
    Write-Host 'together with SHA256SUMS.txt.'
    Write-Host 'The notes switch language through the links at the top; keep all three translated.'
}
