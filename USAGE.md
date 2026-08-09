# NOX 0.7 — использование

## Запуск

```bash
nox
nox example.com
nox "rust terminal browser"
```

## Terminal Layout

`L` переключает `LAYOUT ↔ FLOW`. В Layout Mode NOX реконструирует DOM как responsive terminal UI: header/nav/main/aside/footer, sections и component grids. На широких viewport появляются колонки, на узких выполняется reflow.

`:` → `layout` делает то же самое. Поиск `/` при активном совпадении использует линейное представление, чтобы позиционирование оставалось точным.

## Visual Mode

По умолчанию включён.

```text
V
```

Переключает:

```text
VISUAL ↔ TEXT
```

Visual Mode показывает rich-content и bitmap previews изображений.

## Проверка картинки напрямую

Откройте HTTP/HTTPS URL на PNG/JPEG/WebP/GIF:

```bash
nox https://example.org/photo.jpg
```

## Reader Mode

```text
R
```

Reader Mode и Visual Mode независимы.

## Навигация

```text
Ctrl+L / o   omnibox
s            web search
g            link hints
Tab / l      links
b / f        back / forward
r            reload
Ctrl+T       new tab
Ctrl+W       close tab
Ctrl+Tab     next tab
```

## Поиск

```text
/            find on page
n            next
N            previous
```

## Search aliases

```text
? query
!ddg query
!g query
!gh query
!w query
```

## Command Palette

```text
:
```

В том числе:

```text
visual
reader
open
search
find
hints
links
bookmarks
history
forms
```

## Config

```bash
nox config --path
```

Visual settings:

```toml
visual_mode = true
layout_mode = true
load_images = true
max_images = 8
image_width = 64
image_max_bytes = 2000000
```

Если страницы грузятся слишком долго из-за изображений:

```toml
max_images = 3
```

или:

```toml
load_images = false
```

## Dump

```bash
nox --dump example.com
```

Картинки выводятся текстом:

```text
[IMG] description -> https://...
```

## Doctor

```bash
nox doctor
```

0.6 также показывает текущие visual/image настройки.

## HD images (0.6.1)

`image_width = 64` используется по умолчанию. Допустимый диапазон — `24..160`. После изменения ширины выполните reload страницы, чтобы NOX заново декодировал preview с новым quality target.
