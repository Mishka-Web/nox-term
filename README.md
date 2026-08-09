# NOX Browser 0.7.4

**NOX** — portable terminal-first браузер на Rust: keyboard-first, без Chromium, с Reader Mode, поиском, вкладками, **HD Visual Mode** и **Terminal Layout Engine**.

NOX 0.7.4 умеет реконструировать структуру HTML-страницы как responsive terminal UI: `header`, `nav`, `main`, `aside`, `section`, card-like компоненты и `footer` раскладываются в панели, колонки и сетки в зависимости от ширины терминала. HD image renderer сохранён, но 0.7.4 делает его компактным и адаптивным: изображения больше не захватывают почти весь экран терминала.

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

Подробнее: [INSTALL.md](INSTALL.md). Перед релизом: [TESTING.md](TESTING.md).

## Что нового в 0.7.4 — Terminal Layout Engine

Переключение:

```text
L
```

`LAYOUT` пытается воспроизвести композицию сайта в терминальных ограничениях, а `FLOW` оставляет привычный линейный renderer.

Layout analyzer распознаёт:

- `header`, `nav`, `main`, `aside`, `footer`;
- `section` и `article`;
- card/feature/tile-like компоненты по DOM/class heuristics;
- короткие component groups и раскладывает их в 1/2/3 колонки;
- `main + aside` выводит рядом на широком терминале;
- на узком терминале автоматически выполняет responsive reflow в одну колонку;
- изображения оставляет на своих местах как media placeholders и дополнительно выводит compact adaptive media rail.

Пример концепции:

```text
╭─ HEADER ─────────────────────────────────────────────────────────╮
│ Brand                              Sign in   Docs   Download      │
╰──────────────────────────────────────────────────────────────────╯
╭─ NAV ────────────────────────────────────────────────────────────╮
│ Home   •   Guide   •   Blog   •   GitHub                         │
╰──────────────────────────────────────────────────────────────────╯

╭─ MAIN ─────────────────────────────────────╮  ╭─ ASIDE ─────────╮
│ █ Build software in Rust                   │  │ On this page    │
│ text...                                    │  │ • Install       │
│                                            │  │ • Examples      │
╰────────────────────────────────────────────╯  ╰──────────────────╯

  COMPONENT GRID · 3 columns
╭─ Fast ─────────╮  ╭─ Safe ─────────╮  ╭─ Portable ─────╮
│ ...            │  │ ...            │  │ ...            │
╰────────────────╯  ╰────────────────╯  ╰─────────────────╯
```

Это не Chromium и не pixel-perfect CSS engine: NOX не исполняет CSS/JavaScript один-в-один. Он строит **terminal-native representation** по DOM-семантике и layout heuristics, поэтому цель — максимально похожая структура и информационная иерархия, а не совпадение каждого CSS-пикселя.

Подробнее: [LAYOUT.md](LAYOUT.md).

## Что нового в 0.6.1 — HD Visual Content

### Visual Mode

Visual Mode включён по умолчанию. Переключение:

```text
V
```

В верхней панели NOX показывает `VISUAL` или `TEXT`.

В Visual Mode:

- `h1/h2/h3` получают отдельную визуальную иерархию;
- списки отображаются с акцентными bullets;
- blockquote оформляется отдельным блоком;
- `<pre>` / code blocks получают собственный фон;
- таблицы выделяются как структурированный контент;
- `<hr>` превращается в терминальный divider;
- `<figcaption>` отображается как подпись;
- изображения отображаются прямо внутри документа.

### Inline image previews

NOX находит HTML `<img>` и lazy-loading атрибуты (`data-src`, `data-lazy-src`, `data-original`, `srcset`), скачивает ограниченное количество изображений и строит true-color preview.

Пример представления:

```text
╭─ IMAGE Ferris
│ ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
│ ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
╰─ example.org
```

Каждый символ `▀` использует цвет верхнего и нижнего пикселя, поэтому одна терминальная строка кодирует два ряда пикселей.

Поддерживаемые bitmap-форматы в 0.6:

- PNG;
- JPEG;
- GIF (статическое preview);
- WebP.

SVG пока отображается как fallback-карточка, если декодировать его нельзя.

### HD image pipeline

В 0.6.1 bitmap больше не зажимается в старый квадрат `48×48`. NOX сохраняет aspect ratio, использует ширину preview до `96` по умолчанию (`24..160` настраиваемо), допускает более высокие portrait previews и применяет `Lanczos3`. Если терминал уже preview, цвета выбираются через bilinear sampling вместо грубого nearest-neighbor.

Заголовок изображения показывает исходное и терминальное разрешение, например:

```text
IMAGE Ferris  512×512 · HD 96×96
```

### Direct image URLs

Можно открыть изображение непосредственно:

```bash
nox https://example.org/image.png
```

NOX создаст визуальный документ с preview картинки.

### Безопасные лимиты изображений

По умолчанию:

```toml
config_revision = 701
visual_mode = true
layout_mode = true
load_images = true
max_images = 8
image_width = 64
image_max_bytes = 2000000
```

Это ограничивает количество картинок на страницу, размер preview и максимальный объём одного изображения.

## Navigation & Search

`Ctrl+L` или `o` открывает omnibox. `s` — быстрый веб-поиск.

```text
github.com
rust terminal browser
? rust ratatui
!gh ratatui
!w Rust
!g rust terminal browser
!ddg terminal browser
```

Search aliases:

| Shortcut | Поиск |
|---|---|
| `? query` | поисковик из `config.toml` |
| `!ddg query` / `!d query` | DuckDuckGo Lite |
| `!g query` | Google |
| `!gh query` | GitHub |
| `!w query` | Wikipedia |

### Command Palette

Нажмите `:`. Доступны команды:

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
visual
bookmarks
history
forms
home
help
quit
```

### Link Hints

Нажмите `g`, введите номер ссылки и `Enter`.

### Поиск по странице

```text
/     новый поиск
n     следующее совпадение
N     предыдущее совпадение
```

В 0.6 поиск учитывает дополнительную высоту inline-картинок при переходе к совпадению.

## Возможности браузера

- HTTP / HTTPS;
- вкладки;
- Back / Forward;
- history;
- bookmarks;
- session restore;
- persistent cookies;
- Reader Mode;
- Visual Mode;
- inline terminal images;
- HTML tables;
- basic HTML forms GET/POST;
- downloads;
- JSON pretty-view;
- `--dump`;
- self-update;
- portable Windows/Linux/macOS x64/ARM64 releases.

> NOX не исполняет полноценный JavaScript runtime и не строит CSS layout. Это terminal-first renderer, а не Chromium внутри консоли.

## Основные клавиши

| Клавиша | Действие |
|---|---|
| `Ctrl+L` / `o` | omnibox |
| `s` | web search |
| `:` | Command Palette |
| `g` | Link Hints |
| `/` | find |
| `n` / `N` | next / previous match |
| `V` | Visual Mode / Text Mode |
| `L` | Terminal Layout / Flow |
| `R` | Reader Mode |
| `Ctrl+T` / `Ctrl+W` | новая / закрыть вкладку |
| `Ctrl+Tab` | следующая вкладка |
| `Alt+1..9` | перейти на вкладку |
| `Tab` / `l` | Links |
| `Enter` в Links | открыть |
| `t` в Links | открыть в новой вкладке |
| `d` в Links | скачать |
| `b` / `f` | назад / вперёд |
| `r` | reload |
| `m` / `M` | bookmark / bookmarks |
| `H` | history |
| `F` | forms |
| `j/k`, arrows | scroll/navigation |
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

`--dump` не выводит внутренние image markers: вместо них печатается строка `[IMG] alt -> URL`.

## Config

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
user_agent = "NOX/0.6 terminal-browser"
visual_mode = true
load_images = true
max_images = 8
image_width = 64
image_max_bytes = 2000000
# download_dir = "D:/Downloads/NOX"
```

## Сборка

После обновления до 0.6 сначала обновите lockfile новой image-зависимостью:

```bash
cargo check
cargo test
cargo clippy
```

Затем:

```bash
cargo run -- example.com
cargo build --release
```

## Release

После зелёных локальных проверок и CI:

```bash
git tag v0.7.4
git push origin v0.7.4
```

## Roadmap

Следующий большой слой можно строить поверх Visual Mode:

- image cache;
- protocol-native rendering (Kitty/Sixel/iTerm2) как optional backend;
- View Source;
- Network Inspector;
- headers/cookies inspector;
- JSON tree;
- API client;
- proxy / SOCKS5.

## License

MIT.
