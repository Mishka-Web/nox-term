# Changelog


## 0.4.2

- Исправлен однострочный установщик Windows PowerShell.
- Добавлен корневой `install.ps1` bootstrap для `raw.githubusercontent.com`.
- Bootstrap скачивает GitHub Release installer во временный файл и запускает его отдельным PowerShell-процессом.
- После установки bootstrap перечитывает User/Machine PATH, поэтому `nox` становится доступен в текущем PowerShell без перезапуска терминала.
- Определение Windows-архитектуры в release installer больше не зависит только от `RuntimeInformation.OSArchitecture`.

## 0.4.1

- добавлена установка NOX как глобальной пользовательской команды через `nox install`;
- добавлена команда `nox uninstall`;
- добавлен `scripts/dev-install.ps1` для локальной Windows-разработки;
- Windows installer автоматически добавляет NOX в User PATH и обновляет PATH текущего PowerShell;
- Linux/macOS installer автоматически настраивает shell PATH при необходимости;
- добавлен `INSTALL.md`;
- устранены предупреждения `unused import`, `dead_code` и `clippy::collapsible_match`.

## 0.4.0

### Browser essentials

- multi-tab TUI;
- per-tab navigation history;
- persistent global history;
- persistent bookmarks;
- session restore;
- user `config.toml`;
- persistent cookie store;
- Reader mode toggle;
- HTML table rendering;
- basic GET/POST forms;
- link downloads;
- JSON pretty view;
- data/config/cookie management CLI commands.

### Distribution

- keeps portable Windows/Linux/macOS x64/ARM64 builds;
- keeps install scripts;
- keeps SHA-256 release verification;
- keeps `nox update` self-update flow.
