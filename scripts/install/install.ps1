$ErrorActionPreference = "Stop"

$Repository = "__NOX_REPOSITORY__"
$BaseUrl = "https://github.com/$Repository/releases/latest/download"
$InstallDir = if ($env:NOX_INSTALL_DIR) { $env:NOX_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "Programs\NOX" }

function Fail([string]$Message) {
    throw "NOX install error: $Message"
}

$arch = $env:PROCESSOR_ARCHITECTURE

if ([string]::IsNullOrWhiteSpace($arch)) {
    $arch = $env:PROCESSOR_ARCHITEW6432
}

switch ($arch.ToUpperInvariant()) {
    "AMD64" {
        $Asset = "nox-windows-x64.exe"
    }

    "ARM64" {
        $Asset = "nox-windows-arm64.exe"
    }

    default {
        Fail "unsupported Windows architecture: $arch"
    }
}

$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("nox-install-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

try {
    $BinaryPath = Join-Path $TempDir $Asset
    $ChecksumsPath = Join-Path $TempDir "SHA256SUMS"

    Write-Host "Installing NOX from $Repository"
    Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/$Asset" -OutFile $BinaryPath
    Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/SHA256SUMS" -OutFile $ChecksumsPath

    $ChecksumLine = Get-Content $ChecksumsPath | Where-Object {
        $_ -match ("\s\*?" + [regex]::Escape($Asset) + "$")
    } | Select-Object -First 1

    if (-not $ChecksumLine) {
        Fail "checksum for $Asset was not found"
    }

    $Expected = ($ChecksumLine -split "\s+")[0].ToLowerInvariant()
    $Actual = (Get-FileHash -Algorithm SHA256 -Path $BinaryPath).Hash.ToLowerInvariant()
    if ($Expected -ne $Actual) {
        Fail "SHA-256 verification failed"
    }
    Write-Host "SHA-256 verified"

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    $Destination = Join-Path $InstallDir "nox.exe"
    Move-Item -Force $BinaryPath $Destination
    Unblock-File -Path $Destination -ErrorAction SilentlyContinue

    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $PathEntries = @()
    if ($UserPath) { $PathEntries = $UserPath -split ";" }
    if ($PathEntries -notcontains $InstallDir) {
        $NewUserPath = if ([string]::IsNullOrWhiteSpace($UserPath)) { $InstallDir } else { "$UserPath;$InstallDir" }
        [Environment]::SetEnvironmentVariable("Path", $NewUserPath, "User")
        Write-Host "Added $InstallDir to the user PATH"
    }

    if (($env:Path -split ";") -notcontains $InstallDir) {
        $env:Path = "$InstallDir;$env:Path"
    }

    $Version = & $Destination --version
    Write-Host "Installed $Version to $Destination"

    $Resolved = Get-Command nox -ErrorAction SilentlyContinue
    if ($Resolved) {
        Write-Host "Command ready: nox"
        Write-Host "Try: nox --version"
        Write-Host "Try: nox example.com"
    }
    else {
        Write-Host "NOX was installed, but this PowerShell session did not refresh PATH."
        Write-Host "Open a new terminal and run: nox"
    }
}
finally {
    Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
}
