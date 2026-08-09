# NOX Browser 0.4.1

**NOX** — минималистичный portable terminal browser на Rust: reader-first, keyboard-first, без Chromium и без обязательных runtime-зависимостей у конечного пользователя.

NOX 0.4 переводит проект из простого HTML-reader в полноценный терминальный браузер с вкладками, состоянием между запусками, формами, cookies и загрузками.

## Установка команды `nox`

После установки NOX запускается из любой директории просто как обычная CLI-команда:

```powershell
nox
nox github.com
nox --version
```

### Windows — локальная разработка

```powershell
.\scripts\dev-install.ps1
```

Скрипт соберёт release-бинарник, установит его в `%LOCALAPPDATA%\Programs\NOX` и автоматически добавит этот каталог в пользовательский `PATH`.

### Windows — GitHub Release

```powershell
irm https://raw.githubusercontent.com/Mishka-Web/nox-term/main/install.ps1 | iex
```

### Linux / macOS

```bash
curl -fsSL https://github.com/Mishka-Web/nox-term/releases/latest/download/install.sh | sh
```

Если portable-бинарник уже скачан вручную:

```bash
nox install
```

На Windows из каталога загрузки:

```powershell
.\nox.exe install
```

Подробности: [INSTALL.md](INSTALL.md).


## Что появилось в 0.4

- вкладки: `Ctrl+T`, `Ctrl+W`, `Ctrl+Tab`, `Alt+1..9`;
- отдельная Back/Forward history внутри каждой вкладки;
- глобальная история посещений (`H`);
- persistent bookmarks (`m`, `M`);
- восстановление вкладок после перезапуска;
- `config.toml`;
- cookies между запросами и persistent cookies для долгоживущих cookie-записей;
- Reader mode toggle (`R`);
- HTML tables;
- basic HTML forms: text/password/textarea/checkbox/radio/select/submit;
- GET и POST `application/x-www-form-urlencoded` forms;
- downloads из панели Links (`d`);
- JSON pretty-view;
- прежние `--dump`, self-update и portable release pipeline сохранены.

> NOX не исполняет JavaScript и не строит CSS layout. Это сознательная архитектура: быстрый текстовый браузер для терминала, SSH и developer workflows.

## Быстрый запуск для разработчика

```bash
cargo run
```

Или сразу открыть страницу:

```bash
cargo run -- github.com
```

Release build:

```bash
cargo build --release
```

## Основные клавиши

| Клавиша | Действие |
|---|---|
| `Ctrl+L` / `o` | URL или поисковый запрос |
| `Ctrl+T` | новая вкладка |
| `Ctrl+W` | закрыть вкладку |
| `Ctrl+Tab` | следующая вкладка |
| `Alt+1..9` | перейти на вкладку |
| `Tab` / `l` | ссылки |
| `Enter` | открыть выбранную ссылку |
| `d` в Links | скачать выбранную ссылку |
| `/` | поиск по странице |
| `n` | следующее совпадение |
| `b` / `f` | назад / вперёд |
| `r` | reload |
| `R` | Reader mode on/off |
| `m` | добавить/удалить текущую страницу из bookmarks |
| `M` | открыть bookmarks |
| `H` | открыть глобальную history |
| `F` | формы страницы |
| `j/k`, arrows | навигация/scroll |
| `?` | help |
| `q` | exit |

## Формы

На странице с `<form>` нажмите `F`.

NOX умеет:

- text input;
- password input;
- textarea;
- checkbox;
- radio;
- select;
- submit;
- GET forms;
- POST `application/x-www-form-urlencoded` forms.

В панели Forms:

```text
j/k      выбрать поле
Enter    редактировать поле / submit
Space    checkbox, radio или следующий select option
Esc      закрыть
```

Это не JavaScript form runtime: client-side JS validation и JS-generated forms не исполняются.

## Downloads

Откройте Links (`Tab`), выберите ссылку и нажмите:

```text
d
```

По умолчанию файл попадёт в пользовательский `Downloads`.

Каталог можно переопределить:

```toml
download_dir = "D:/Downloads/NOX"
```

или переменной окружения:

```text
NOX_DOWNLOAD_DIR
```

## Config

Узнать путь:

```bash
nox config --path
```

Пример `config.toml`:

```toml
homepage = "about:home"
search_engine = "https://html.duckduckgo.com/html/?q={query}"
restore_session = true
reader_mode = true
max_history = 1000
user_agent = "NOX/0.4 terminal-browser"
# download_dir = "D:/Downloads/NOX"
```

`{query}` обязателен для кастомного search engine. Если его нет, NOX использует DuckDuckGo HTML fallback.

## Где хранятся данные

Путь можно узнать:

```bash
nox data --path
```

Обычно:

### Windows

```text
%LOCALAPPDATA%\NOX\
```

### Linux

```text
~/.config/nox/
```

### macOS

```text
~/Library/Application Support/NOX/
```

В каталоге находятся:

```text
config.toml
bookmarks.json
history.json
session.json
cookies.json
```

Для полностью изолированного экземпляра можно задать:

```text
NOX_DATA_DIR=/custom/path
```

## Cookies

NOX передаёт cookies между запросами и сохраняет persistent cookie-записи в `cookies.json`.

Очистить:

```bash
nox cookies clear
```

Реализация 0.4 ориентирована на обычные RFC6265-style Domain/Path/Secure cookies. Полная browser-grade cookie policy (SameSite policy engine, Public Suffix enforcement и детальное expiration handling) остаётся задачей следующего security hardening этапа.

## Dump mode

```bash
nox --dump example.com
```

```bash
nox --dump example.com | less
```

Если stdout не является TTY, NOX автоматически работает как текстовая CLI-утилита.

## Self-update

```bash
nox update --check
nox update
```

Официальный release binary получает адрес `owner/repo` во время GitHub Actions build и проверяет SHA-256 перед заменой бинарника.

## Portable distribution

После публикации тега:

```bash
git tag v0.4.0
git push origin v0.4.0
```

GitHub Actions собирает release assets для Windows, Linux и macOS, x64/ARM64.

Подробнее: [PORTABLE.md](PORTABLE.md).

## Архитектурная цель

NOX не пытается заменить Chrome визуально. Направление проекта:

```text
terminal browser
+ reader
+ curl/HTTPie ergonomics
+ lightweight DevTools
```

Следующий логичный релиз — developer tooling: network inspector, headers, cookies viewer, source mode, proxy/SOCKS5 и API mode.
