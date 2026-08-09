$ErrorActionPreference = "Stop"

$Repository = "Mishka-Web/nox-term"
$ReleaseInstaller = "https://github.com/$Repository/releases/latest/download/install.ps1"
$TempFile = Join-Path ([System.IO.Path]::GetTempPath()) ("nox-bootstrap-" + [Guid]::NewGuid().ToString("N") + ".ps1")

try {
    Write-Host "Downloading NOX installer..."
    Invoke-WebRequest -UseBasicParsing -Uri $ReleaseInstaller -OutFile $TempFile

    # GitHub release assets are served as binary downloads. Reading the file back
    # explicitly avoids Invoke-RestMethod | Invoke-Expression edge cases in
    # Windows PowerShell while keeping the public install command one-line.
    $InstallerSource = [System.IO.File]::ReadAllText($TempFile, [System.Text.Encoding]::UTF8)
    $InstallerBlock = [ScriptBlock]::Create($InstallerSource)
    & $InstallerBlock
}
finally {
    Remove-Item -Force $TempFile -ErrorAction SilentlyContinue
}
