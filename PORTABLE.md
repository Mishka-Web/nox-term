# NOX Portable Distribution

NOX 0.7 рассчитан на две модели использования:

1. установка одной командой;
2. запуск одиночного portable-бинарника без установки.

Пользователю не нужны Rust, Cargo, Node.js или Python.

## Release assets

Каждый release содержит:

```text
nox-windows-x64.exe
nox-windows-arm64.exe
nox-linux-x64
nox-linux-arm64
nox-macos-x64
nox-macos-arm64

nox-windows-x64.zip
nox-windows-arm64.zip
nox-linux-x64.tar.gz
nox-linux-arm64.tar.gz
nox-macos-x64.tar.gz
nox-macos-arm64.tar.gz

install.ps1
install.sh
VERSION
SHA256SUMS
```

Чистые бинарники используются install scripts и self-updater. Архивы удобны для ручной загрузки.

## Installation

Windows:

```powershell
irm https://raw.githubusercontent.com/Mishka-Web/nox-term/main/install.ps1 | iex
```

Linux/macOS:

```bash
curl -fsSL https://github.com/Mishka-Web/nox-term/releases/latest/download/install.sh | sh
```

## Self-update

```bash
nox update --check
nox update
```

Официальная release-сборка получает переменную `NOX_GITHUB_REPOSITORY=${{ github.repository }}` во время компиляции. Она становится compile-time настройкой и позволяет бинарнику находить собственный release feed.

Для локальной dev-сборки источник можно задать во время запуска:

```bash
NOX_GITHUB_REPOSITORY="Mishka-Web/nox-term" nox update --check
```

или во время сборки:

```bash
NOX_GITHUB_REPOSITORY="Mishka-Web/nox-term" cargo build --release
```

## Integrity

Release workflow создаёт `SHA256SUMS` после подготовки всех release assets.

Install scripts и `nox update` скачивают checksum list независимо от бинарника и не устанавливают файл, если SHA-256 не совпадает.

## Linux portability

Linux release targets:

```text
x86_64-unknown-linux-musl
aarch64-unknown-linux-musl
```

Это уменьшает зависимость от конкретной версии glibc. HTTPS всё ещё ожидает доступный набор доверенных CA-сертификатов в системе.

## Platform matrix

| ОС | Архитектура | Release target |
|---|---|---|
| Windows | x64 | `x86_64-pc-windows-msvc` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` |
| Linux | x64 | `x86_64-unknown-linux-musl` |
| Linux | ARM64 | `aarch64-unknown-linux-musl` |
| macOS | Intel | `x86_64-apple-darwin` |
| macOS | Apple Silicon | `aarch64-apple-darwin` |

## Version release procedure

1. Изменить `version` в `Cargo.toml`.
2. Commit/push.
3. Создать совпадающий тег.

```bash
git tag v0.7.4
git push origin v0.7.4
```

Release workflow специально завершится ошибкой, если тег и Cargo version отличаются.
