# Changelog

## 0.7.4

- Clean Clippy pass: removed five non-functional style warnings.
- Simplified Windows install destination expression, link-hint match guard, command palette selection check, and integer ceiling division.
- No behavior changes relative to 0.7.3.

## 0.7.3

- исправлен semantic layout extractor: короткие валидные `<section>`/`<article>` больше не отбрасываются из-за слишком высокого fingerprint threshold;
- regression-test `terminal_layout_extracts_semantic_regions` теперь проверяет реальное поведение extractor, а не обходится ослаблением assertion;
- сохранены scroll/focus fixes из 0.7.2 и compact adaptive images из 0.7.1.

## 0.7.2

- исправлен scroll regression: высота документа теперь считается по реально перенесённым terminal rows, а не по количеству логических `Line`;
- `↑/↓`, `j/k`, PageUp/PageDown и End снова работают на длинных Flow/Visual страницах после URL navigation и поиска;
- после успешной навигации NOX централизованно возвращает focus в `Mode::Normal` и очищает stale input/hints state;
- обработка клавиатуры принимает `KeyEventKind::Repeat`, поэтому удерживание стрелок и `j/k` даёт непрерывную прокрутку;
- поиск по странице после Enter также гарантированно возвращает document focus;
- сохранены Terminal Layout Engine и компактные adaptive image previews из 0.7.1.

## 0.7.1

- исправлен Rust borrow-checker `E0506` в `draw_reader`: immutable borrow страницы теперь завершается до обновления `rendered_height`;
- изображения стали компактнее: default HD buffer `96` → `64`;
- фактическая ширина preview адаптируется к терминалу (~46–52% content area, максимум 72 колонки);
- высокие изображения ограничены примерно 36 строками терминала без искажения aspect ratio;
- metadata изображения теперь показывает source/buffer/view размеры;
- config migration `701` аккуратно переводит только прежний default `96` → `64`, сохраняя пользовательские значения.

## 0.7.0

- добавлен Terminal Layout Engine и горячая клавиша `L` (`LAYOUT ↔ FLOW`);
- HTML DOM анализируется как `header/nav/main/aside/section/article/footer`;
- `main + aside` получают responsive двухколоночную композицию на широком терминале;
- короткие sections/card/feature/tile компоненты автоматически раскладываются в 1/2/3-column grid;
- layout автоматически reflow-ится в одну колонку на узких viewport;
- добавлены terminal-native panel borders, region labels и отдельная визуальная иерархия;
- HD image previews сохранены и выводятся в full-width MEDIA rail, чтобы колонки не уничтожали детализацию изображения;
- `:layout` добавлен в Command Palette;
- `layout_mode = true` и `config_revision = 700` добавлены в config;
- `nox doctor` показывает состояние layout renderer;
- добавлены unit tests semantic region extraction и default layout config;
- добавлен `LAYOUT.md`.

## 0.6.1

- HD image renderer: default preview width увеличен с 48 до 96 px;
- максимальная настраиваемая ширина preview увеличена до 160 px;
- удалено квадратное ограничение preview, portrait/landscape сохраняют aspect ratio;
- resize изображений выполняется через Lanczos3;
- при адаптации под более узкий terminal viewport используется bilinear sampling;
- image metadata показывает исходное разрешение и HD preview resolution;
- добавлен `config_revision = 601` и одноразовая миграция старого default `image_width = 48` → `96`;
- добавлены unit tests для aspect-ratio fitting;
- сохранён idempotent Windows installer fix.

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
