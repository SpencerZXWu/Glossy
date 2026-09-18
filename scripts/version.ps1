<#
.SYNOPSIS
    Reads, checks and sets the Glossy version number.

.DESCRIPTION
    src-tauri/tauri.conf.json is the single source of truth for the version: the
    number Tauri stamps into the executable and the installer comes from there, so
    every other place it is written has to agree with it. Those places are:

        src-tauri/tauri.conf.json   the version Tauri ships (authoritative)
        package.json                npm metadata
        package-lock.json           the same number twice: root and packages[""]
        src-tauri/Cargo.toml        the [package] version
        src-tauri/Cargo.lock        the entry cargo keeps in step with Cargo.toml

    -Check (the default) prints all of them and exits with 1 when any of them
    disagrees; that is what CI and scripts/release.ps1 call. -Set writes one
    version into all of them, so a release is one command instead of five edits.

.PARAMETER Check
    Compare the locations and fail on drift. This is the default.

.PARAMETER Get
    Check first, then print the authoritative version and nothing else, for
    scripts: $version = & scripts\version.ps1 -Get

.PARAMETER Set
    Write this version (x.y.z) into every location and verify the result.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/version.ps1
    powershell -ExecutionPolicy Bypass -File scripts/version.ps1 -Set 0.1.1
#>
[CmdletBinding(DefaultParameterSetName = 'Check')]
param(
    [Parameter(ParameterSetName = 'Check')]
    [switch]$Check,

    [Parameter(ParameterSetName = 'Get')]
    [switch]$Get,

    [Parameter(ParameterSetName = 'Set', Mandatory = $true)]
    [string]$Set
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$utf8 = New-Object System.Text.UTF8Encoding($false)

# Every place the version appears. Each pattern captures what comes before the
# number, the number itself and what comes after it, so a replacement rewrites
# only the digits and the rest of the file is written back byte for byte.
$locations = @(
    [pscustomobject]@{
        Name    = 'src-tauri/tauri.conf.json'
        Path    = 'src-tauri\tauri.conf.json'
        Detail  = 'what Tauri ships'
        Pattern = '(?m)^( *"version" *: *")([^"]+)(")'
    }
    [pscustomobject]@{
        Name    = 'package.json'
        Path    = 'package.json'
        Detail  = 'npm package'
        Pattern = '(?m)^( *"version" *: *")([^"]+)(")'
    }
    [pscustomobject]@{
        Name    = 'package-lock.json'
        Path    = 'package-lock.json'
        Detail  = 'npm lockfile root'
        Pattern = '(?m)^( *"version" *: *")([^"]+)(")'
    }
    [pscustomobject]@{
        Name    = 'package-lock.json'
        Path    = 'package-lock.json'
        Detail  = 'npm lockfile packages[""]'
        Pattern = '("packages" *: *\{\s*"" *: *\{[^}]*?"version" *: *")([^"]+)(")'
    }
    [pscustomobject]@{
        Name    = 'src-tauri/Cargo.toml'
        Path    = 'src-tauri\Cargo.toml'
        Detail  = 'crate'
        Pattern = '(?m)^(version\s*=\s*")([^"]+)(")'
    }
    [pscustomobject]@{
        Name    = 'src-tauri/Cargo.lock'
        Path    = 'src-tauri\Cargo.lock'
        Detail  = 'lockfile entry for the glossy crate'
        Pattern = '(?m)^(name = "glossy"\r?\nversion = ")([^"]+)(")'
    }
)

# Reads one row per location, in the order above. The first row is the
# authoritative version.
function Get-VersionRows {
    $texts = @{}

    foreach ($location in $locations) {
        if (-not $texts.ContainsKey($location.Path)) {
            $full = Join-Path $root $location.Path
            if (-not (Test-Path -LiteralPath $full)) {
                throw "Missing file: $($location.Name). Run this script from the repository."
            }
            $texts[$location.Path] = [IO.File]::ReadAllText($full)
        }

        $match = [regex]::Match($texts[$location.Path], $location.Pattern)
        if (-not $match.Success) {
            throw "No version found in $($location.Name) ($($location.Detail))."
        }

        [pscustomobject]@{
            Name    = $location.Name
            Detail  = $location.Detail
            Version = $match.Groups[2].Value
        }
    }
}

function Get-VersionReport {
    param([object[]]$Rows, [string]$Expected)

    $width = 0
    foreach ($row in $Rows) {
        if ($row.Name.Length -gt $width) { $width = $row.Name.Length }
    }

    $lines = @()
    $drift = $false
    foreach ($row in $Rows) {
        $agrees = $row.Version -eq $Expected
        if (-not $agrees) { $drift = $true }
        $mark = if ($agrees) { '   ' } else { ' ! ' }
        $lines += ('{0}{1}  {2,-10}  {3}' -f $mark, $row.Name.PadRight($width), $row.Version, $row.Detail)
    }

    return [pscustomobject]@{ Lines = $lines; Drift = $drift }
}

$rows = @(Get-VersionRows)
$expected = $rows[0].Version

if ($PSCmdlet.ParameterSetName -eq 'Set') {
    if ($Set -notmatch '^\d+\.\d+\.\d+$') {
        throw "-Set expects a version like 0.1.1, got '$Set'."
    }

    $version = $Set
    $paths = @($locations | ForEach-Object { $_.Path } | Select-Object -Unique)

    foreach ($path in $paths) {
        $full = Join-Path $root $path
        $text = [IO.File]::ReadAllText($full)

        foreach ($location in @($locations | Where-Object { $_.Path -eq $path })) {
            # An instance Regex, because the static overload that takes a
            # MatchEvaluator has no replacement count and would rewrite every
            # match in the file.
            $regex = [regex]::new($location.Pattern)
            $text = $regex.Replace($text, {
                param($match)
                $match.Groups[1].Value + $version + $match.Groups[3].Value
            }, 1)
        }

        [IO.File]::WriteAllText($full, $text, $utf8)
    }

    Write-Host "Wrote $version to $($paths.Count) files." -ForegroundColor Cyan
    $rows = @(Get-VersionRows)
    $expected = $version
}

$report = Get-VersionReport -Rows $rows -Expected $expected

if ($PSCmdlet.ParameterSetName -eq 'Get') {
    if ($report.Drift) {
        foreach ($line in $report.Lines) { Write-Host $line }
        Write-Host "Version drift: the files above disagree with $($rows[0].Name) ($expected)." -ForegroundColor Red
        exit 1
    }
    Write-Output $expected
    exit 0
}

Write-Host ''
foreach ($line in $report.Lines) { Write-Host $line }
Write-Host ''

if ($report.Drift) {
    Write-Host "$($rows[0].Name) says $expected; the rows marked ! disagree." -ForegroundColor Red
    Write-Host "Sync them with: powershell -ExecutionPolicy Bypass -File scripts/version.ps1 -Set $expected" -ForegroundColor Yellow
    exit 1
}

Write-Host "All $($rows.Count) locations are on $expected." -ForegroundColor Green
exit 0
