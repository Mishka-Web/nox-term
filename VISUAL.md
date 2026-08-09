# NOX 0.6.1 — HD Visual Mode

## Идея

NOX остаётся TUI-приложением. Картинки не открываются во внешнем окне: bitmap адаптивно уменьшается до HD preview и кодируется в terminal cells. В 0.6.1 исходная пропорция сохраняется, а старое квадратное ограничение 48×48 удалено.

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
image_width = 64
image_max_bytes = 2000000
```

`max_images` ограничивается внутренним пределом 24.

`image_width` clamp-ится в диапазоне 24..160 preview pixels для внутреннего HD buffer. Resize выполняется через Lanczos3. Фактический размер на экране вычисляется адаптивно: примерно 46–52% ширины content area, с мягким потолком 72 колонок и максимум 72 пиксельными строками (около 36 строк терминала) для высоких изображений. При финальном уменьшении используется bilinear sampling.

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

## Что даёт HD pipeline

- старый renderer: максимум около 48 px по каждой стороне;
- новый default: 96 px по ширине;
- landscape и portrait больше не принудительно вписываются в квадрат;
- исходные dimensions сохраняются в metadata;
- прозрачность композитится на NOX background;
- Lanczos3 сохраняет мелкие границы лучше простого thumbnail;
- bilinear sampling уменьшает ступеньки при resize под текущую ширину терминала.

Для очень широкого терминала можно попробовать:

```toml
image_width = 120
```

или максимум:

```toml
image_width = 160
```
