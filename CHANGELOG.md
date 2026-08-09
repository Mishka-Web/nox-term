# Changelog

## 0.6.0

- добавлен Visual Mode (`V`, `:visual`) с сохранением настройки в `config.toml`;
- добавлен inline true-color renderer изображений через Unicode half-block `▀`;
- добавлено декодирование PNG/JPEG/GIF/WebP через Rust `image`;
- добавлена обработка прямых image URLs как визуальных документов;
- HTML parser теперь распознаёт `img`, lazy image attributes, `figcaption` и `hr`;
- добавлены rich styles для headings, lists, blockquotes, code blocks и tables;
- добавлены лимиты `max_images`, `image_width`, `image_max_bytes`;
- поиск по странице учитывает высоту inline image previews при jump;
- `--dump` преобразует image markers в читаемые `[IMG] alt -> URL`;
- `nox doctor` показывает visual/image configuration;
- добавлен `VISUAL.md`;
- версия проекта поднята до 0.6.0.

## 0.5.0

- новый omnibox с явным `? query` и search aliases `!ddg`, `!g`, `!gh`, `!w`;
- default search переведён на DuckDuckGo Lite non-JavaScript endpoint;
- существующий конфиг со старым встроенным DuckDuckGo HTML endpoint автоматически мигрирует на Lite;
- CLI-команда `nox search <query>`;
- Command Palette по `:` с фильтрацией команд;
- Link Hints по `g` для быстрого открытия пронумерованных ссылок;
- `t` в Links открывает ссылку в новой вкладке;
- поиск по странице теперь показывает `current/total`;
- `N` ищет предыдущее совпадение, `n` — следующее;
- контекстный footer и процент прокрутки;
- новый `nox doctor` для диагностики TTY/config/data/downloads/cookies/HTTPS;
- обновлены home/help экраны;
- версия проекта поднята до 0.5.0.


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
