$ErrorActionPreference = "Stop"

$Repository = "Mishka-Web/nox-term"
$ReleaseInstaller = "https://github.com/$Repository/releases/latest/download/install.ps1"
$TempFile = Join-Path ([System.IO.Path]::GetTempPath()) ("nox-bootstrap-" + [Guid]::NewGuid().ToString("N") + ".ps1")

function Get-PowerShellExecutable {
    $candidate = if ($PSVersionTable.PSEdition -eq "Core") {
        Join-Path $PSHOME "pwsh.exe"
    }
    else {
        Join-Path $PSHOME "powershell.exe"
    }

    if (Test-Path $candidate) {
        return $candidate
    }

    $command = Get-Command pwsh.exe -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    $command = Get-Command powershell.exe -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    throw "NOX install error: PowerShell executable was not found."
}

try {
    Write-Host "Downloading NOX installer..."
    Invoke-WebRequest -UseBasicParsing -Uri $ReleaseInstaller -OutFile $TempFile

    # Execute the downloaded release installer as a real .ps1 file.
    # This intentionally mirrors the installation path that works reliably
    # in Windows PowerShell 5.1 and PowerShell 7.
    $PowerShellExe = Get-PowerShellExecutable
    & $PowerShellExe -NoProfile -ExecutionPolicy Bypass -File $TempFile

    if ($LASTEXITCODE -ne 0) {
        throw "NOX install error: installer exited with code $LASTEXITCODE."
    }

    # The installer writes the directory to the persistent User PATH, but a
    # child process cannot modify the environment of its parent. Refresh PATH
    # here so `nox` is immediately available in the same terminal session.
    $MachinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $env:Path = @($MachinePath, $UserPath) -join ";"

    $Nox = Get-Command nox -ErrorAction SilentlyContinue
    if ($Nox) {
        Write-Host ""
        Write-Host "NOX is ready."
        & nox --version
        Write-Host "Run: nox"
    }
    else {
        Write-Host ""
        Write-Host "NOX was installed successfully."
        Write-Host "Open a new terminal and run: nox"
    }
}
finally {
    Remove-Item -Force $TempFile -ErrorAction SilentlyContinue
}
