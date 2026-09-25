<#
    Puts a shortcut that runs scripts\dev.cmd on the desktop, so the dev build
    can be opened with a double click instead of a command line. The shortcut
    points at this checkout and carries the application icon; delete it and run
    this script again to get it back.

    powershell -ExecutionPolicy Bypass -File scripts\make-shortcut.ps1 [-Name "Glossy dev"]
#>
param(
    [string] $Name = 'Glossy dev'
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$path = Join-Path ([Environment]::GetFolderPath('Desktop')) "$Name.lnk"

$shell = New-Object -ComObject WScript.Shell
$link = $shell.CreateShortcut($path)
$link.TargetPath = Join-Path $env:SystemRoot 'System32\cmd.exe'
$link.Arguments = '/c ""' + (Join-Path $PSScriptRoot 'dev.cmd') + '""'
$link.WorkingDirectory = $root
$link.IconLocation = Join-Path $root 'src-tauri\icons\icon.ico'
$link.Description = "Run Glossy from $root, as a dev build"

$link.Save()
Write-Output $path
