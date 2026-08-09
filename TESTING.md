# NOX 0.7.4 — smoke test

## Scroll / focus regression (0.7.4)

1. Open a long page with `Ctrl+L` or `o`, enter a URL and press Enter.
2. Verify `Down`, `Up`, `j`, `k`, `PageDown`, `PageUp`, `Home`, and `End`.
3. Press `/`, search for a word in a long paragraph, press Enter, then scroll again with arrows and `j/k`.
4. Open `:` command palette, run `open` or `search`, navigate, and verify document scrolling again.
5. Hold `Down` or `j`: key-repeat should continuously scroll instead of moving only once.

Expected: navigation/overlays never leave the document in a stale input mode, and long wrapped paragraphs contribute their real terminal-row height to the scroll range.

## 1. Toolchain

```powershell
cargo check
cargo test
cargo clippy
```

`cargo check` должен обновить `Cargo.lock`, поскольку 0.6 добавляет crate `image`.

## 2. Version

```powershell
cargo run -- --version
```

Ожидается:

```text
nox 0.7.4
```

## 3. Home

```powershell
cargo run
```

Проверить:

- header содержит `VISUAL` и `LAYOUT`;
- `V` переключает `VISUAL ↔ TEXT`;
- `L` переключает `LAYOUT ↔ FLOW`;
- `?` открывает help с Visual Mode.

## 4. Rich content

Открыть страницу со статьёй:

```text
o
https://en.wikipedia.org/wiki/Terminal_emulator
Enter
```

Проверить:

- заголовки имеют разный стиль;
- списки выглядят структурированно;
- таблицы/цитаты/code blocks не сливаются с обычным текстом;
- `R` меняет Reader/Document;
- `V` меняет Visual/Text.


## 5. Terminal Layout Engine

Открыть сначала современный landing page:

```powershell
cargo run -- https://www.rust-lang.org/
```

Проверить:

- в header NOX отображается `LAYOUT`;
- semantic header/nav/main/footer выводятся отдельными панелями;
- если найден `aside` и окно достаточно широкое, `MAIN` и `ASIDE` стоят рядом;
- feature/card sections на широком терминале могут стать 2/3-column grid;
- при уменьшении ширины окна layout автоматически reflow-ится;
- `L` возвращает старый flow renderer без reload;
- `:` → `layout` выполняет тот же toggle;
- `/` временно использует flow representation для точного jump к найденной строке.

Проверить длинную статью:

```powershell
cargo run -- https://en.wikipedia.org/wiki/Rust_(programming_language)
```

Убедиться, что sections не превращаются в бессмысленную сетку: длинные секции должны идти вертикально.

## 6. Inline images

На странице с bitmap-картинками должны появиться блоки:

```text
╭─ IMAGE ...
│ <color preview>
╰─ host
```

Если конкретный ресурс является SVG или блокирует hotlink, NOX должен показать fallback `preview unavailable`, а не падать.

## 7. Direct image document

Проверить прямой URL на PNG/JPEG/WebP/GIF.

Ожидается один визуальный image document с HTTP status и preview.

## 8. Limits

Открыть:

```powershell
cargo run -- config --path
```

Проверить в `config.toml`:

```toml
visual_mode = true
load_images = true
max_images = 8
image_width = 64
image_max_bytes = 2000000
```

Поставить временно:

```toml
load_images = false
```

Перезапустить страницу: должен быть image fallback без скачивания preview.

Вернуть `true`.

## 8. Search + images

На длинной странице с картинками:

```text
/
```

Найти слово ниже изображения. Скролл должен прыгнуть ближе к реальному совпадению, учитывая высоту image preview.

Проверить `n` и `N`.

## 9. Navigation regression

Проверить:

```text
Ctrl+L
s
g
:
Tab
Ctrl+T
Ctrl+W
b
f
r
R
m
M
H
F
```

## 10. Dump

```powershell
cargo run -- --dump https://en.wikipedia.org/wiki/Terminal_emulator
```

Не должно быть управляющих `NOXIMG` markers. Вместо них:

```text
[IMG] ... -> https://...
```

## 11. Doctor

```powershell
cargo run -- doctor
```

Проверить строку `Visual:`.

## 12. Release build

```powershell
cargo build --release
.\target\release\nox.exe --version
.\target\release\nox.exe doctor
```

## 13. Commit

После того как `Cargo.lock` обновлён локально:

```powershell
git add .
git commit -m "feat: NOX 0.7 terminal layout engine"
git push origin main
```

После зелёного CI:

```powershell
git tag v0.7.4
git push origin v0.7.4
```

## HD image checks (0.6.1)

1. `nox config --path` и проверьте, что после миграции старого 0.6-конфига `image_width = 64`.
2. Откройте страницу с большой landscape-картинкой: header должен показать `original · HD preview`.
3. Откройте portrait image URL: изображение не должно сжиматься в квадрат.
4. Сузьте окно терминала: preview должен плавно уменьшиться без crash и заметной nearest-neighbor pixelation.
5. Попробуйте `image_width = 120`, затем reload страницы.
6. `cargo test` должен включать HD aspect-ratio tests.

## Deterministic local layout demo

From repository root:

```powershell
python -m http.server 8080
```

In another terminal:

```powershell
cargo run -- http://127.0.0.1:8080/examples/layout-demo.html
```

Open the same URL in a normal browser for side-by-side comparison. Resize the terminal across ~70, ~100 and ~140 columns and verify 1/2/3-column reflow.
