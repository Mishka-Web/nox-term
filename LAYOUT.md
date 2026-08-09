# NOX Terminal Layout Engine

NOX 0.7.4 добавляет terminal-native layout renderer. Его задача — не эмулировать Chromium, а переносить структуру веб-страницы в ограничения character-cell terminal.

## Pipeline

```text
HTML DOM
  ↓
semantic region analyzer
  ↓
header / nav / main / aside / section / article / footer
  ↓
component heuristics (card / feature / tile)
  ↓
responsive terminal compositor
  ↓
1 / 2 / 3-column TUI + HD media rail
```

## Что учитывается

- semantic HTML regions;
- heading hierarchy;
- navigation links;
- short section/component density;
- наличие sidebar/aside;
- текущая ширина terminal viewport;
- HD images из Visual Mode.

## Responsive rules

- `< 76` columns: один столбец;
- `76..131`: component grid до двух колонок;
- `>= 132`: component grid до трёх колонок;
- `main + aside` располагаются рядом примерно от 92 columns;
- sidebar ограничивается разумной шириной, остальное получает main;
- при resize композиция рассчитывается заново на каждом render.

## Images

Внутри структурных блоков image marker занимает место в потоке, а bitmap preview рендерится ниже в MEDIA rail на полной ширине. Это сознательное решение: true-color `▀` preview теряет слишком много detail, если пытаться ужать его в sidebar/card column.

## Ограничения

0.7.4 пока не является CSS layout engine. External CSS, flex/grid computed styles, fonts, pseudo-elements, canvas и JavaScript layout не воспроизводятся pixel-perfect. NOX использует DOM semantics + class/component heuristics.

Следующий этап может добавить CSS-aware layer: загрузку stylesheets, parsing `display`, `grid-template-columns`, `flex-direction`, widths, margins/padding и media queries с переводом CSS px в terminal cells.
