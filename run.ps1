$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host " NOX Browser " -ForegroundColor Cyan
Write-Host " ------------" -ForegroundColor DarkGray

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host ""
    Write-Host "Rust/Cargo не найден." -ForegroundColor Red
    Write-Host "Для разработки установи Rust. Для обычного запуска используй portable nox.exe." -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "Сборка release..." -ForegroundColor Cyan
cargo build --release

Write-Host ""
Write-Host "Запуск NOX..." -ForegroundColor Cyan
& ".\target\release\nox.exe" @args
