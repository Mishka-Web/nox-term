# NOX 0.5 — использование

## Запуск

```bash
nox
nox example.com
```

## Omnibox

Откройте `Ctrl+L` или `o`. Быстрый веб-поиск: `s`.

URL:

```text
github.com
https://example.com
localhost:5173
```

Обычный текст автоматически становится поисковым запросом:

```text
rust terminal browser
```

Принудительный поиск:

```text
? rust terminal browser
```

Shortcuts:

```text
!ddg query   DuckDuckGo
!d query     DuckDuckGo
!g query     Google
!gh query    GitHub
!w query     Wikipedia
```

CLI:

```bash
nox search rust ratatui
```

## Command Palette

Нажмите `:`.

Начните вводить название команды, используйте `↑/↓`, затем `Enter`.

Доступны:

```text
open, search, find, hints, links,
new-tab, close-tab, reload, back, forward,
reader, bookmarks, history, forms,
home, help, quit
```

## Link Hints

Нажмите `g`.

Введите номер ссылки и `Enter`.

Пример:

```text
[ 1] Documentation
[ 2] GitHub
[ 3] Download
```

Введите `2` + `Enter`.

## Links

`Tab` или `l`:

```text
j/k       выбрать
Enter     открыть
 t        открыть в новой вкладке
 d        скачать
Esc       закрыть
```

## Поиск по странице

```text
/     начать поиск
n     следующее совпадение
N     предыдущее совпадение
```

NOX показывает номер текущего совпадения, например `2/7`.

## Вкладки

```text
Ctrl+T       новая
Ctrl+W       закрыть
Ctrl+Tab     следующая
Shift+Tab    предыдущая
Alt+1..9     выбрать по номеру
```

## Reader Mode

```text
R
```

## Закладки и история

```text
m     добавить/удалить bookmark
M     bookmarks
H     history
```

## Формы

```text
F
```

Поддерживаются text/password/textarea/checkbox/radio/select/submit и GET/POST urlencoded forms.

## Dump mode

```bash
nox --dump example.com
nox --dump example.com > page.txt
```

## Диагностика

```bash
nox doctor
```

## Обновление

```bash
nox update --check
nox update
```

## Справка

```bash
nox --help
```

В TUI:

```text
?
```
