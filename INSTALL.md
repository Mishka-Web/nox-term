# Установка NOX как системной команды

После установки NOX можно запускать из любой директории одной командой:

```bash
nox
```

или:

```bash
nox github.com
```

## Windows

Для опубликованного GitHub Release:

```powershell
irm https://github.com/Mishka-Web/nox-term/releases/latest/download/install.ps1 | iex
```

Установщик:

1. определяет x64 или ARM64;
2. скачивает подходящий `nox.exe`;
3. проверяет SHA-256;
4. устанавливает бинарник в `%LOCALAPPDATA%\Programs\NOX\nox.exe`;
5. автоматически добавляет каталог NOX в пользовательский `PATH`;
6. делает `nox` доступным в текущем PowerShell и во всех новых терминалах.

Проверка:

```powershell
nox --version
nox example.com
```

### Локальная установка из исходников

Во время разработки:

```powershell
.\scripts\dev-install.ps1
```

Скрипт соберёт release-версию, установит её и добавит в `PATH`.

После этого:

```powershell
nox
```

## Linux / macOS

```bash
curl -fsSL https://github.com/Mishka-Web/nox-term/releases/latest/download/install.sh | sh
```

Установщик старается выбрать пользовательский каталог, который уже находится в `PATH`. Если это невозможно, NOX устанавливается в `~/.local/bin`, а путь автоматически добавляется в профиль shell.

После открытия нового терминала:

```bash
nox --version
nox example.com
```

## Если бинарник скачан вручную

Windows:

```powershell
.\nox.exe install
```

Linux/macOS:

```bash
./nox install
```

NOX установит копию самого себя в стандартный пользовательский каталог и зарегистрирует команду `nox`.

Удаление:

```bash
nox uninstall
```
