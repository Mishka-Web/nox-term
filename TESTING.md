# NOX 0.6 — smoke test

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
nox 0.6.0
```

## 3. Home

```powershell
cargo run
```

Проверить:

- header содержит `VISUAL`;
- `V` переключает `VISUAL ↔ TEXT`;
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

## 5. Inline images

На странице с bitmap-картинками должны появиться блоки:

```text
╭─ IMAGE ...
│ <color preview>
╰─ host
```

Если конкретный ресурс является SVG или блокирует hotlink, NOX должен показать fallback `preview unavailable`, а не падать.

## 6. Direct image document

Проверить прямой URL на PNG/JPEG/WebP/GIF.

Ожидается один визуальный image document с HTTP status и preview.

## 7. Limits

Открыть:

```powershell
cargo run -- config --path
```

Проверить в `config.toml`:

```toml
visual_mode = true
load_images = true
max_images = 8
image_width = 48
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
git commit -m "feat: NOX 0.6 visual content and inline images"
git push origin main
```

После зелёного CI:

```powershell
git tag v0.6.0
git push origin v0.6.0
```
