param(
    [switch]$Debug
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

if ($Debug) {
    Write-Host "Building NOX (debug)..."
    cargo build
    $Binary = Join-Path $ProjectRoot "target\debug\nox.exe"
} else {
    Write-Host "Building NOX (release)..."
    cargo build --release
    $Binary = Join-Path $ProjectRoot "target\release\nox.exe"
}

if (-not (Test-Path $Binary)) {
    throw "NOX binary not found: $Binary"
}

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\NOX"
$Destination = Join-Path $InstallDir "nox.exe"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item -Force $Binary $Destination

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$Entries = if ($UserPath) { $UserPath -split ";" } else { @() }
if ($Entries -notcontains $InstallDir) {
    $NewPath = if ([string]::IsNullOrWhiteSpace($UserPath)) { $InstallDir } else { "$($UserPath.TrimEnd(';'));$InstallDir" }
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    Write-Host "Added to user PATH: $InstallDir"
}

# Make `nox` available immediately in this PowerShell process too.
if (($env:Path -split ";") -notcontains $InstallDir) {
    $env:Path = "$InstallDir;$env:Path"
}

Write-Host ""
Write-Host "Installed: $Destination"
& $Destination --version
Write-Host ""
Write-Host "Now try:"
Write-Host "  nox"
Write-Host "  nox example.com"
Write-Host "  nox --version"
