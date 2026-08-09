param(
    [string]$Repository = $env:NOX_GITHUB_REPOSITORY
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($Repository)) {
    try {
        $remote = (git config --get remote.origin.url 2>$null).Trim()
        if ($remote -match 'github\.com[:/](?<repo>[^/]+/[^/]+?)(?:\.git)?$') {
            $Repository = $Matches.repo
        }
    } catch {}
}

if (-not [string]::IsNullOrWhiteSpace($Repository)) {
    $env:NOX_GITHUB_REPOSITORY = $Repository
    Write-Host "Self-update repository: $Repository"
} else {
    Write-Warning "NOX_GITHUB_REPOSITORY is unknown. The binary will work, but 'nox update' will require NOX_GITHUB_REPOSITORY=owner/repo."
}

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
if ($arch -eq "X64") {
    $target = "x86_64-pc-windows-msvc"
    $package = "nox-windows-x64"
} elseif ($arch -eq "Arm64") {
    $target = "aarch64-pc-windows-msvc"
    $package = "nox-windows-arm64"
} else {
    throw "Unsupported Windows architecture: $arch"
}

rustup target add $target
cargo test --release --target $target
cargo build --release --target $target

$dest = Join-Path "dist" $package
New-Item -ItemType Directory -Force -Path $dest | Out-Null
Copy-Item "target/$target/release/nox.exe" "$dest/nox.exe" -Force
Copy-Item README.md, PORTABLE.md, LICENSE -Destination $dest -Force

Write-Host "Portable build: $dest/nox.exe"
