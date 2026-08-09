# NOX Browser 0.5.0

**NOX** — минималистичный terminal-first браузер на Rust: keyboard-first, portable, без Chromium и без обязательных runtime-зависимостей у конечного пользователя.

NOX ориентирован на чтение документации и статей, SSH/VPS, быстрый веб-поиск, работу со ссылками, HTML-формами, JSON и developer workflows.

## Установка

### Windows

```powershell
irm https://raw.githubusercontent.com/Mishka-Web/nox-term/main/install.ps1 | iex
```

После установки:

```powershell
nox
nox github.com
nox --version
```

### Linux / macOS

```bash
curl -fsSL https://github.com/Mishka-Web/nox-term/releases/latest/download/install.sh | sh
```

Если portable-бинарник уже скачан:

```bash
nox install
```

На Windows:

```powershell
.\nox.exe install
```

Подробнее: [INSTALL.md](INSTALL.md). Перед релизом: [TESTING.md](TESTING.md).

## Что нового в 0.5

### Omnibox и веб-поиск

`Ctrl+L` или `o` открывает omnibox. Для мгновенного веб-поиска нажмите `s`. Он понимает URL и обычный поисковый запрос.

```text
github.com
rust terminal browser
? rust ratatui
!gh ratatui
!w Rust
!g rust terminal browser
!ddg terminal browser
```

Доступные shortcuts:

| Shortcut | Поиск |
|---|---|
| `? query` | поисковик из `config.toml` |
| `!ddg query` / `!d query` | DuckDuckGo Lite |
| `!g query` | Google |
| `!gh query` | GitHub |
| `!w query` | Wikipedia |

Из CLI можно явно запустить поиск:

```bash
nox search rust ratatui
```

### Command Palette

Нажмите:

```text
:
```

Доступны команды с фильтрацией по мере ввода:

```text
open
search
find
hints
links
new-tab
close-tab
reload
back
forward
reader
bookmarks
history
forms
home
help
quit
```

Используйте `↑/↓` и `Enter`.

### Link Hints

Нажмите:

```text
g
```

NOX покажет пронумерованные ссылки. Введите номер и нажмите `Enter`.

Это быстрый keyboard-first способ открыть ссылку без перехода в отдельную панель Links.

### Улучшенный поиск по странице

```text
/     новый поиск
n     следующее совпадение
N     предыдущее совпадение
```

Статус показывает позицию:

```text
«rust» · 2/7 · n/N следующее/предыдущее
```

### NOX Doctor

Диагностика установки и окружения:

```bash
nox doctor
```

Проверяются:

- TTY;
- каталог пользовательских данных;
- `config.toml`;
- каталог Downloads;
- cookie store;
- HTTP/HTTPS-клиент;
- доступ к `https://example.com`;
- путь текущего executable.

## Возможности браузера

- HTTP / HTTPS;
- несколько вкладок;
- Back / Forward history для каждой вкладки;
- глобальная история;
- persistent bookmarks;
- восстановление сессии;
- cookies;
- Reader Mode;
- HTML tables;
- basic HTML forms;
- GET / POST forms;
- downloads;
- JSON pretty-view;
- `--dump` для stdout/pipes;
- self-update;
- portable releases Windows/Linux/macOS x64/ARM64.

> NOX намеренно не исполняет полноценный JavaScript runtime и не строит CSS layout. Это terminal-first браузер, а не Chromium в терминале.

## Основные клавиши

| Клавиша | Действие |
|---|---|
| `Ctrl+L` / `o` | omnibox: URL или поиск |
| `s` | быстрый веб-поиск |
| `:` | Command Palette |
| `g` | Link Hints |
| `/` | поиск по странице |
| `n` / `N` | следующее / предыдущее совпадение |
| `Ctrl+T` | новая вкладка |
| `Ctrl+W` | закрыть вкладку |
| `Ctrl+Tab` | следующая вкладка |
| `Alt+1..9` | перейти на вкладку |
| `Tab` / `l` | панель Links |
| `Enter` в Links | открыть ссылку |
| `t` в Links | открыть ссылку в новой вкладке |
| `d` в Links | скачать ссылку |
| `b` / `f` | назад / вперёд |
| `r` | reload |
| `R` | Reader Mode |
| `m` / `M` | bookmark / список bookmarks |
| `H` | история |
| `F` | формы |
| `j/k`, arrows | навигация / scroll |
| `?` | help |
| `q` / `Ctrl+C` | exit |

## CLI

```bash
nox
nox example.com
nox rust terminal browser
nox search rust ratatui
nox --dump example.com
nox doctor
nox config --path
nox data --path
nox cookies clear
nox update --check
nox update
nox --version
```

## Config

Путь:

```bash
nox config --path
```

Пример:

```toml
homepage = "about:home"
search_engine = "https://lite.duckduckgo.com/lite/?q={query}"
restore_session = true
reader_mode = true
max_history = 1000
user_agent = "NOX/0.5 terminal-browser"
# download_dir = "D:/Downloads/NOX"
```

`{query}` — место для URL-encoded поискового запроса. При обновлении со старого встроенного DuckDuckGo HTML-шаблона NOX автоматически мигрирует его на Lite; пользовательские поисковики не меняются.

## Данные пользователя

```bash
nox data --path
```

NOX хранит данные отдельно от бинарника:

```text
config.toml
bookmarks.json
history.json
session.json
cookies.json
```

## Сборка из исходников

```bash
cargo check
cargo test
cargo clippy
cargo run
```

Сразу открыть сайт:

```bash
cargo run -- example.com
```

Release build:

```bash
cargo build --release
```

## Release

Перед тегом версия в `Cargo.toml` должна совпадать с тегом.

```bash
git tag v0.5.0
git push origin v0.5.0
```

GitHub Actions собирает portable assets для Windows, Linux и macOS, x64/ARM64.

## Roadmap

Следующее крупное направление — **Developer Tools**:

- Network Inspector;
- request/response headers;
- View Source;
- cookie viewer;
- JSON tree;
- API mode;
- HTTP methods/body/headers;
- proxy / SOCKS5.

## License

MIT.
