# NOX 0.6 — Visual Mode

## Идея

NOX остаётся TUI-приложением. Картинки не открываются во внешнем окне: bitmap уменьшается до безопасного preview и кодируется в terminal cells.

Используется символ `▀`:

- foreground = верхний пиксель;
- background = нижний пиксель;
- один terminal cell = два вертикальных пикселя.

Это делает backend независимым от Kitty/Sixel/iTerm2 и позволяет работать в обычных true-color терминалах и через SSH.

## HTML

Парсер распознаёт:

```text
img
figcaption
hr
h1..h6
p
li
pre
blockquote
table
```

Для `<img>` проверяются:

```text
data-src
data-lazy-src
data-original
src
srcset
```

Поддерживаются только HTTP/HTTPS image URLs. `data:` изображения в 0.6 пропускаются.

## Форматы

Bitmap preview:

- PNG
- JPEG
- GIF
- WebP

SVG пока не rasterize-ится.

## Конфигурация

```toml
visual_mode = true
load_images = true
max_images = 8
image_width = 48
image_max_bytes = 2000000
```

`max_images` ограничивается внутренним пределом 24.

`image_width` дополнительно clamp-ится в диапазоне 12..80 preview pixels.

`image_max_bytes` clamp-ится в диапазоне 64 KiB..8 MiB.

## Переключение

```text
V
```

Visual Mode сохраняется в `config.toml`.

Command Palette:

```text
:visual
```

## Reader Mode

`R` и `V` независимы:

- `R` определяет, какую часть HTML читать (`article/main/body`);
- `V` определяет, как найденный контент визуализировать.

Поэтому можно использовать:

```text
Reader + Visual
Reader + Text
Document + Visual
Document + Text
```
