# NOX 0.5 — smoke test

Перед публикацией релиза выполните проверки на локальной машине.

## 1. Компиляция

```powershell
cargo check
cargo test
cargo clippy
```

## 2. Запуск TUI

```powershell
cargo run -- example.com
```

Проверьте прокрутку `j/k`, `b/f`, `r`, `R`, `Tab`, `Ctrl+T`, `Ctrl+W`.

## 3. Omnibox и web search

В TUI нажмите `s`, введите:

```text
rust ratatui terminal
```

Ожидается открытие результатов поиска.

Затем через `Ctrl+L` проверьте:

```text
? rust ratatui
!ddg rust tui
!g rust tui
!gh ratatui
!w Rust
```

Обычный URL по-прежнему должен открываться напрямую:

```text
github.com
```

## 4. Link Hints

На странице со ссылками нажмите `g`.

Ожидается список вида:

```text
[ 1] ...
[ 2] ...
[ 3] ...
```

Введите номер и `Enter`. Должна открыться соответствующая ссылка.

## 5. Command Palette

Нажмите `:`.

Проверьте фильтрацию, например введите:

```text
rea
```

Выберите `reader` и нажмите `Enter`.

Также проверьте `search`, `hints`, `new-tab`, `history`, `bookmarks`.

## 6. Find

Нажмите `/`, введите слово с текущей страницы и `Enter`.

```text
n   следующее
N   предыдущее
```

Footer/status должен показывать позицию вида `2/7`.

## 7. Links

Нажмите `Tab` или `l`.

```text
Enter   открыть
 t      открыть в новой вкладке
 d      скачать
```

## 8. CLI search

```powershell
cargo run -- search rust ratatui
```

В non-interactive режиме:

```powershell
cargo run -- --dump "? rust ratatui"
```

## 9. Doctor

```powershell
cargo run -- doctor
```

Ожидаются проверки TTY, data/config, downloads, cookies, HTTPS и executable path.

## 10. Release build

```powershell
cargo build --release
.\target\release\nox.exe --version
.\target\release\nox.exe doctor
.\target\release\nox.exe example.com
```

Ожидаемая версия:

```text
nox 0.5.0
```

## 11. Перед тегом

```powershell
git status
git add .
git commit -m "feat: NOX 0.5 navigation and search"
git push origin main
```

После успешного CI:

```powershell
git tag v0.5.0
git push origin v0.5.0
```
