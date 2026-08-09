mod cookies;
mod install;
mod state;
mod update;

use std::{
    collections::HashSet,
    env,
    fs,
    io::{self, IsTerminal},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result};
use cookies::PersistentCookieStore;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
    },
    DefaultTerminal, Frame,
};
use reqwest::{
    blocking::{Client, RequestBuilder, Response},
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    redirect::Policy,
};
use scraper::{ElementRef, Html, Selector};
use state::{
    now_unix, AppConfig, Bookmark, HistoryEntry, Paths, SessionData,
};
use url::{form_urlencoded, Url};

const BG: Color = Color::Rgb(10, 12, 16);
const PANEL: Color = Color::Rgb(15, 18, 24);
const TEXT: Color = Color::Rgb(226, 232, 240);
const MUTED: Color = Color::Rgb(113, 128, 150);
const ACCENT: Color = Color::Rgb(56, 189, 248);
const ACCENT_SOFT: Color = Color::Rgb(14, 116, 144);

#[derive(Debug, Clone)]
struct LinkTarget {
    label: String,
    url: Url,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormFieldKind {
    Text,
    Password,
    Hidden,
    Checkbox,
    Radio,
    Select,
    TextArea,
    Submit,
}

#[derive(Debug, Clone)]
struct FormField {
    name: String,
    label: String,
    value: String,
    kind: FormFieldKind,
    checked: bool,
    options: Vec<(String, String)>,
    selected: usize,
}

#[derive(Debug, Clone)]
struct PageForm {
    action: Url,
    method: String,
    fields: Vec<FormField>,
}

#[derive(Debug, Clone)]
struct Page {
    title: String,
    display_url: String,
    url: Option<Url>,
    lines: Vec<String>,
    links: Vec<LinkTarget>,
    forms: Vec<PageForm>,
    status_code: Option<u16>,
    raw_html: Option<String>,
    reader_mode: bool,
}

#[derive(Debug)]
struct Tab {
    pages: Vec<Page>,
    history_index: usize,
    scroll: u16,
    search_query: String,
    match_line: Option<usize>,
}

impl Tab {
    fn home() -> Self {
        Self {
            pages: vec![home_page()],
            history_index: 0,
            scroll: 0,
            search_query: String::new(),
            match_line: None,
        }
    }

    fn current_page(&self) -> &Page {
        &self.pages[self.history_index]
    }

    fn current_page_mut(&mut self) -> &mut Page {
        &mut self.pages[self.history_index]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Address,
    Search,
    Links,
    Bookmarks,
    History,
    Forms,
    FormInput { form: usize, field: usize },
    Help,
}

struct App {
    client: Client,
    cookies: Arc<PersistentCookieStore>,
    paths: Paths,
    config: AppConfig,
    bookmarks: Vec<Bookmark>,
    visit_history: Vec<HistoryEntry>,

    tabs: Vec<Tab>,
    active_tab: usize,
    mode: Mode,
    running: bool,
    viewport_height: u16,

    input: String,
    input_cursor: usize,
    link_state: ListState,
    bookmark_state: ListState,
    history_state: ListState,
    form_state: ListState,
    status: String,
}

impl App {
    fn new(initial: Option<String>) -> Result<Self> {
        let paths = Paths::discover()?;
        let config = state::load_config(&paths)?;
        let bookmarks = state::load_bookmarks(&paths).unwrap_or_default();
        let visit_history = state::load_history(&paths).unwrap_or_default();
        let cookies = Arc::new(PersistentCookieStore::load(paths.cookies.clone()));
        let client = build_client(&config, Arc::clone(&cookies))?;

        let mut app = Self {
            client,
            cookies,
            paths,
            config,
            bookmarks,
            visit_history,
            tabs: vec![Tab::home()],
            active_tab: 0,
            mode: Mode::Normal,
            running: true,
            viewport_height: 1,
            input: String::new(),
            input_cursor: 0,
            link_state: ListState::default(),
            bookmark_state: ListState::default(),
            history_state: ListState::default(),
            form_state: ListState::default(),
            status: "Готово".to_string(),
        };

        if let Some(value) = initial {
            if !value.trim().is_empty() {
                app.navigate_input(value.trim(), true);
                return Ok(app);
            }
        }

        if app.config.restore_session {
            let session = state::load_session(&app.paths).unwrap_or_default();
            if !session.tabs.is_empty() {
                app.restore_session(session);
                return Ok(app);
            }
        }

        if app.config.homepage != "about:home" && !app.config.homepage.trim().is_empty() {
            let homepage = app.config.homepage.clone();
            app.navigate_input(&homepage, true);
        }

        Ok(app)
    }

    fn restore_session(&mut self, session: SessionData) {
        let urls: Vec<String> = session.tabs.into_iter().take(12).collect();
        if urls.is_empty() {
            return;
        }

        self.tabs.clear();
        for url in urls {
            self.tabs.push(Tab::home());
            self.active_tab = self.tabs.len() - 1;
            if url != "about:home" {
                self.navigate_input(&url, true);
            }
        }
        self.active_tab = session.active_tab.min(self.tabs.len().saturating_sub(1));
        self.mode = Mode::Normal;
        self.reset_view();
        self.status = format!("Восстановлено вкладок: {}", self.tabs.len());
    }

    fn persist(&self) {
        let _ = state::save_config(&self.paths, &self.config);
        let _ = state::save_bookmarks(&self.paths, &self.bookmarks);
        let _ = state::save_history(&self.paths, &self.visit_history);
        let session = SessionData {
            tabs: self
                .tabs
                .iter()
                .map(|tab| tab.current_page().display_url.clone())
                .collect(),
            active_tab: self.active_tab,
        };
        let _ = state::save_session(&self.paths, &session);
        self.cookies.save();
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key);
                    }
                }
            }
        }
        Ok(())
    }

    fn tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    fn tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    fn current_page(&self) -> &Page {
        self.tab().current_page()
    }

    fn current_page_mut(&mut self) -> &mut Page {
        self.tab_mut().current_page_mut()
    }

    fn current_url_text(&self) -> String {
        self.current_page().display_url.clone()
    }

    fn reset_view(&mut self) {
        let has_links = !self.current_page().links.is_empty();
        let has_forms = !flatten_form_fields(self.current_page()).is_empty();
        let tab = self.tab_mut();
        tab.scroll = 0;
        tab.match_line = None;
        self.link_state.select(has_links.then_some(0));
        self.form_state.select(has_forms.then_some(0));
    }

    fn navigate_input(&mut self, raw: &str, push_history: bool) {
        let target = match resolve_user_input(raw, &self.config) {
            Ok(url) => url,
            Err(err) => {
                self.status = format!("Ошибка адреса: {err}");
                return;
            }
        };
        self.navigate_url(target, push_history);
    }

    fn navigate_url(&mut self, target: Url, push_history: bool) {
        self.status = format!("Открываю {}", target.host_str().unwrap_or(target.as_str()));
        let reader_mode = self.config.reader_mode;
        let page = match fetch_page(&self.client, target.clone(), reader_mode) {
            Ok(page) => page,
            Err(err) => {
                self.status = format!("Ошибка: {err:#}");
                error_page(target, &err)
            }
        };
        self.install_page(page, push_history, true);
    }

    fn install_page(&mut self, page: Page, push_history: bool, record_visit: bool) {
        let status_code = page.status_code;
        let links = page.links.len();
        let forms = page.forms.len();
        let visit = page.url.as_ref().map(|url| HistoryEntry {
            title: page.title.clone(),
            url: url.as_str().to_string(),
            visited_at: now_unix(),
        });

        if push_history {
            let tab = self.tab_mut();
            tab.pages.truncate(tab.history_index + 1);
            tab.pages.push(page);
            tab.history_index = tab.pages.len() - 1;
        } else {
            let index = self.tab().history_index;
            self.tabs[self.active_tab].pages[index] = page;
        }

        if record_visit {
            if let Some(visit) = visit {
                self.visit_history.push(visit);
                let max = self.config.max_history.max(10);
                if self.visit_history.len() > max {
                    let remove = self.visit_history.len() - max;
                    self.visit_history.drain(0..remove);
                }
            }
        }

        self.mode = Mode::Normal;
        self.reset_view();
        self.status = match status_code {
            Some(code) => format!("HTTP {code} · {links} ссылок · {forms} форм"),
            None => format!("{links} ссылок · {forms} форм"),
        };
    }

    fn reload(&mut self) {
        let Some(url) = self.current_page().url.clone() else {
            self.status = "Домашнюю страницу перезагружать не нужно".to_string();
            return;
        };
        self.navigate_url(url, false);
    }

    fn go_back(&mut self) {
        if self.tab().history_index == 0 {
            self.status = "Это первая страница вкладки".to_string();
            return;
        }
        self.tab_mut().history_index -= 1;
        self.mode = Mode::Normal;
        self.reset_view();
        self.status = "Назад".to_string();
    }

    fn go_forward(&mut self) {
        if self.tab().history_index + 1 >= self.tab().pages.len() {
            self.status = "Вперёд идти некуда".to_string();
            return;
        }
        self.tab_mut().history_index += 1;
        self.mode = Mode::Normal;
        self.reset_view();
        self.status = "Вперёд".to_string();
    }

    fn new_tab(&mut self) {
        self.tabs.push(Tab::home());
        self.active_tab = self.tabs.len() - 1;
        self.mode = Mode::Normal;
        self.reset_view();
        self.status = format!("Новая вкладка · {} вкладок", self.tabs.len());
    }

    fn close_tab(&mut self) {
        if self.tabs.len() == 1 {
            self.tabs[0] = Tab::home();
            self.active_tab = 0;
        } else {
            self.tabs.remove(self.active_tab);
            self.active_tab = self.active_tab.min(self.tabs.len() - 1);
        }
        self.mode = Mode::Normal;
        self.reset_view();
        self.status = format!("{} вкладок", self.tabs.len());
    }

    fn next_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
            self.mode = Mode::Normal;
            self.reset_view();
        }
    }

    fn previous_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
            self.mode = Mode::Normal;
            self.reset_view();
        }
    }

    fn switch_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
            self.mode = Mode::Normal;
            self.reset_view();
        }
    }

    fn open_address(&mut self) {
        self.mode = Mode::Address;
        self.input = if self.current_page().url.is_some() {
            self.current_url_text()
        } else {
            String::new()
        };
        self.input_cursor = self.input.chars().count();
        self.status = "Введите URL или поисковый запрос".to_string();
    }

    fn open_search(&mut self) {
        self.mode = Mode::Search;
        self.input.clear();
        self.input_cursor = 0;
        self.status = "Поиск по текущей странице".to_string();
    }

    fn toggle_links(&mut self) {
        if self.current_page().links.is_empty() {
            self.status = "На странице нет ссылок".to_string();
            return;
        }
        self.mode = if self.mode == Mode::Links { Mode::Normal } else { Mode::Links };
        if self.mode == Mode::Links && self.link_state.selected().is_none() {
            self.link_state.select(Some(0));
        }
    }

    fn toggle_bookmark(&mut self) {
        let Some(url) = self.current_page().url.as_ref().map(|url| url.as_str().to_string()) else {
            self.status = "Домашнюю страницу нельзя добавить в закладки".to_string();
            return;
        };
        if let Some(index) = self.bookmarks.iter().position(|item| item.url == url) {
            self.bookmarks.remove(index);
            self.status = "Закладка удалена".to_string();
        } else {
            let title = self.current_page().title.clone();
            self.bookmarks.push(Bookmark {
                title,
                url,
                created_at: now_unix(),
            });
            self.status = "Добавлено в закладки".to_string();
        }
        let _ = state::save_bookmarks(&self.paths, &self.bookmarks);
    }

    fn open_bookmarks(&mut self) {
        self.mode = Mode::Bookmarks;
        self.bookmark_state.select((!self.bookmarks.is_empty()).then_some(0));
        self.status = format!("Закладок: {}", self.bookmarks.len());
    }

    fn open_history_panel(&mut self) {
        self.mode = Mode::History;
        let selected = self.visit_history.len().checked_sub(1);
        self.history_state.select(selected);
        self.status = format!("История: {} записей", self.visit_history.len());
    }

    fn toggle_reader_mode(&mut self) {
        let page = self.current_page().clone();
        let Some(html) = page.raw_html.as_deref() else {
            self.status = "Reader mode доступен только для HTML".to_string();
            return;
        };
        let Some(url) = page.url.clone() else {
            return;
        };
        let reader_mode = !page.reader_mode;
        let rebuilt = parse_html_page(url, page.status_code.unwrap_or(200), html, reader_mode);
        let index = self.tab().history_index;
        self.tabs[self.active_tab].pages[index] = rebuilt;
        self.reset_view();
        self.status = if reader_mode {
            "Reader mode включён".to_string()
        } else {
            "Reader mode выключен".to_string()
        };
    }

    fn open_forms(&mut self) {
        let count = flatten_form_fields(self.current_page()).len();
        if count == 0 {
            self.status = "На странице нет поддерживаемых полей формы".to_string();
            return;
        }
        self.mode = Mode::Forms;
        self.form_state.select(Some(0));
        self.status = "Enter редактировать/отправить · Space переключить".to_string();
    }

    fn selected_form_field(&self) -> Option<(usize, usize)> {
        let index = self.form_state.selected()?;
        flatten_form_fields(self.current_page()).get(index).copied()
    }

    fn activate_form_field(&mut self) {
        let Some((form_index, field_index)) = self.selected_form_field() else {
            return;
        };
        let field = self.current_page().forms[form_index].fields[field_index].clone();
        match field.kind {
            FormFieldKind::Text | FormFieldKind::Password | FormFieldKind::TextArea => {
                self.mode = Mode::FormInput { form: form_index, field: field_index };
                self.input = field.value;
                self.input_cursor = self.input.chars().count();
                self.status = format!("Редактирование: {}", field.label);
            }
            FormFieldKind::Checkbox | FormFieldKind::Radio | FormFieldKind::Select => {
                self.toggle_form_value(form_index, field_index);
            }
            FormFieldKind::Submit => self.submit_form(form_index, Some(field_index)),
            FormFieldKind::Hidden => {}
        }
    }

    fn toggle_form_value(&mut self, form_index: usize, field_index: usize) {
        let kind = self.current_page().forms[form_index].fields[field_index].kind;
        match kind {
            FormFieldKind::Checkbox => {
                let status = {
                    let field = &mut self.current_page_mut().forms[form_index].fields[field_index];
                    field.checked = !field.checked;
                    format!("{}: {}", field.label, if field.checked { "on" } else { "off" })
                };
                self.status = status;
            }
            FormFieldKind::Radio => {
                let name = self.current_page().forms[form_index].fields[field_index].name.clone();
                let fields = &mut self.current_page_mut().forms[form_index].fields;
                for (index, field) in fields.iter_mut().enumerate() {
                    if field.kind == FormFieldKind::Radio && field.name == name {
                        field.checked = index == field_index;
                    }
                }
                self.status = "Выбран radio-вариант".to_string();
            }
            FormFieldKind::Select => {
                let status = {
                    let field = &mut self.current_page_mut().forms[form_index].fields[field_index];
                    if field.options.is_empty() {
                        None
                    } else {
                        field.selected = (field.selected + 1) % field.options.len();
                        field.value = field.options[field.selected].1.clone();
                        Some(format!("{}: {}", field.label, field.options[field.selected].0))
                    }
                };
                if let Some(status) = status {
                    self.status = status;
                }
            }
            _ => {}
        }
    }

    fn submit_form(&mut self, form_index: usize, submit_field: Option<usize>) {
        let Some(form) = self.current_page().forms.get(form_index).cloned() else {
            return;
        };
        self.status = format!("Отправляю {} {}", form.method, form.action);
        let result = fetch_form(&self.client, &form, submit_field, self.config.reader_mode);
        match result {
            Ok(page) => self.install_page(page, true, true),
            Err(err) => self.status = format!("Ошибка формы: {err:#}"),
        }
    }

    fn max_scroll(&self) -> u16 {
        let total = self.current_page().lines.len();
        let visible = self.viewport_height.max(1) as usize;
        total.saturating_sub(visible).min(u16::MAX as usize) as u16
    }

    fn scroll_by(&mut self, delta: i32) {
        let max = self.max_scroll() as i32;
        let next = (self.tab().scroll as i32 + delta).clamp(0, max);
        self.tab_mut().scroll = next as u16;
    }

    fn page_step(&self) -> i32 {
        i32::from(self.viewport_height.saturating_sub(2).max(1))
    }

    fn move_list(state: &mut ListState, len: usize, delta: i32) {
        if len == 0 {
            state.select(None);
            return;
        }
        let current = state.selected().unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, len.saturating_sub(1) as i32) as usize;
        state.select(Some(next));
    }

    fn move_link(&mut self, delta: i32) {
        let len = self.current_page().links.len();
        Self::move_list(&mut self.link_state, len, delta);
        if let Some(index) = self.link_state.selected() {
            if let Some(link) = self.current_page().links.get(index) {
                self.status = link.url.as_str().to_string();
            }
        }
    }

    fn open_selected_link(&mut self) {
        let Some(index) = self.link_state.selected() else { return; };
        let Some(target) = self.current_page().links.get(index).map(|link| link.url.clone()) else { return; };
        self.navigate_url(target, true);
    }

    fn open_selected_bookmark(&mut self) {
        let Some(index) = self.bookmark_state.selected() else { return; };
        let Some(url) = self.bookmarks.get(index).map(|item| item.url.clone()) else { return; };
        self.navigate_input(&url, true);
    }

    fn open_selected_history(&mut self) {
        let Some(index) = self.history_state.selected() else { return; };
        let Some(url) = self.visit_history.get(index).map(|item| item.url.clone()) else { return; };
        self.navigate_input(&url, true);
    }

    fn download_selected_link(&mut self) {
        let Some(index) = self.link_state.selected() else { return; };
        let Some(url) = self.current_page().links.get(index).map(|link| link.url.clone()) else { return; };
        match download_url(&self.client, &url, &self.download_dir()) {
            Ok((path, bytes)) => self.status = format!("Скачано {} байт → {}", bytes, path.display()),
            Err(err) => self.status = format!("Ошибка загрузки: {err:#}"),
        }
    }

    fn download_dir(&self) -> PathBuf {
        self.config
            .download_dir
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.paths.downloads.clone())
    }

    fn commit_search(&mut self) {
        let query = self.input.trim().to_string();
        if query.is_empty() {
            self.mode = Mode::Normal;
            self.status = "Поиск отменён".to_string();
            return;
        }
        self.tab_mut().search_query = query;
        self.mode = Mode::Normal;
        self.find_next_from(0);
    }

    fn find_next(&mut self) {
        if self.tab().search_query.is_empty() {
            self.open_search();
            return;
        }
        let start = self.tab().match_line.map(|line| line + 1).unwrap_or(0);
        self.find_next_from(start);
    }

    fn find_next_from(&mut self, start: usize) {
        let query = self.tab().search_query.to_lowercase();
        let lines = &self.current_page().lines;
        let found = lines
            .iter()
            .enumerate()
            .skip(start)
            .find(|(_, line)| line.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .or_else(|| {
                lines
                    .iter()
                    .enumerate()
                    .take(start.min(lines.len()))
                    .find(|(_, line)| line.to_lowercase().contains(&query))
                    .map(|(index, _)| index)
            });

        match found {
            Some(index) => {
                let query = self.tab().search_query.clone();
                let tab = self.tab_mut();
                tab.match_line = Some(index);
                tab.scroll = index.min(u16::MAX as usize) as u16;
                self.status = format!("Найдено: «{query}»");
            }
            None => {
                let query = self.tab().search_query.clone();
                self.tab_mut().match_line = None;
                self.status = format!("«{query}» не найдено");
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.running = false;
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('t') {
            self.new_tab();
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('w') {
            self.close_tab();
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Tab {
            self.next_tab();
            return;
        }
        if key.code == KeyCode::BackTab
            || (key.modifiers.contains(KeyModifiers::CONTROL)
                && key.modifiers.contains(KeyModifiers::SHIFT)
                && key.code == KeyCode::Tab)
        {
            self.previous_tab();
            return;
        }
        if key.modifiers.contains(KeyModifiers::ALT) {
            if let KeyCode::Char(c @ '1'..='9') = key.code {
                self.switch_tab((c as u8 - b'1') as usize);
                return;
            }
        }

        match self.mode {
            Mode::Address | Mode::Search | Mode::FormInput { .. } => self.handle_input_key(key),
            Mode::Links => self.handle_links_key(key),
            Mode::Bookmarks => self.handle_bookmarks_key(key),
            Mode::History => self.handle_history_key(key),
            Mode::Forms => self.handle_forms_key(key),
            Mode::Help => self.handle_help_key(key),
            Mode::Normal => self.handle_normal_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('l') {
            self.open_address();
            return;
        }
        if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Left {
            self.go_back();
            return;
        }
        if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Right {
            self.go_forward();
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('o') => self.open_address(),
            KeyCode::Char('/') => self.open_search(),
            KeyCode::Tab | KeyCode::Char('l') => self.toggle_links(),
            KeyCode::Char('b') | KeyCode::Char('[') => self.go_back(),
            KeyCode::Char('f') | KeyCode::Char(']') => self.go_forward(),
            KeyCode::Char('r') => self.reload(),
            KeyCode::Char('R') => self.toggle_reader_mode(),
            KeyCode::Char('n') => self.find_next(),
            KeyCode::Char('m') => self.toggle_bookmark(),
            KeyCode::Char('M') => self.open_bookmarks(),
            KeyCode::Char('H') => self.open_history_panel(),
            KeyCode::Char('F') => self.open_forms(),
            KeyCode::Char('?') => self.mode = Mode::Help,
            KeyCode::Down | KeyCode::Char('j') => self.scroll_by(1),
            KeyCode::Up | KeyCode::Char('k') => self.scroll_by(-1),
            KeyCode::PageDown | KeyCode::Char(' ') => self.scroll_by(self.page_step()),
            KeyCode::PageUp => self.scroll_by(-self.page_step()),
            KeyCode::Home => self.tab_mut().scroll = 0,
            KeyCode::End => { let max = self.max_scroll(); self.tab_mut().scroll = max; },
            _ => {}
        }
    }

    fn handle_input_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = match self.mode {
                    Mode::FormInput { .. } => Mode::Forms,
                    _ => Mode::Normal,
                };
                self.status = "Ввод отменён".to_string();
            }
            KeyCode::Enter => {
                let value = self.input.trim_end().to_string();
                match self.mode {
                    Mode::Address => {
                        if !value.trim().is_empty() {
                            self.navigate_input(value.trim(), true);
                        } else {
                            self.mode = Mode::Normal;
                        }
                    }
                    Mode::Search => self.commit_search(),
                    Mode::FormInput { form, field } => {
                        if let Some(target) = self.current_page_mut().forms.get_mut(form).and_then(|form| form.fields.get_mut(field)) {
                            target.value = value;
                        }
                        self.mode = Mode::Forms;
                        self.status = "Значение формы сохранено".to_string();
                    }
                    _ => {}
                }
            }
            KeyCode::Left => self.input_cursor = self.input_cursor.saturating_sub(1),
            KeyCode::Right => self.input_cursor = (self.input_cursor + 1).min(self.input.chars().count()),
            KeyCode::Home => self.input_cursor = 0,
            KeyCode::End => self.input_cursor = self.input.chars().count(),
            KeyCode::Backspace => self.input_backspace(),
            KeyCode::Delete => self.input_delete(),
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) => self.input_insert(c),
            _ => {}
        }
    }

    fn handle_links_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Tab | KeyCode::Char('l') => self.mode = Mode::Normal,
            KeyCode::Char('q') => self.running = false,
            KeyCode::Down | KeyCode::Char('j') => self.move_link(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_link(-1),
            KeyCode::PageDown => self.move_link(8),
            KeyCode::PageUp => self.move_link(-8),
            KeyCode::Home => self.link_state.select(Some(0)),
            KeyCode::End => { let selected = self.current_page().links.len().checked_sub(1); self.link_state.select(selected); },
            KeyCode::Enter => self.open_selected_link(),
            KeyCode::Char('d') => self.download_selected_link(),
            KeyCode::Char('?') => self.mode = Mode::Help,
            _ => {}
        }
    }

    fn handle_bookmarks_key(&mut self, key: KeyEvent) {
        let len = self.bookmarks.len();
        match key.code {
            KeyCode::Esc | KeyCode::Char('M') => self.mode = Mode::Normal,
            KeyCode::Down | KeyCode::Char('j') => Self::move_list(&mut self.bookmark_state, len, 1),
            KeyCode::Up | KeyCode::Char('k') => Self::move_list(&mut self.bookmark_state, len, -1),
            KeyCode::Enter => self.open_selected_bookmark(),
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(index) = self.bookmark_state.selected() {
                    if index < self.bookmarks.len() {
                        self.bookmarks.remove(index);
                        let selected = if self.bookmarks.is_empty() {
                            None
                        } else {
                            Some(index.min(self.bookmarks.len() - 1))
                        };
                        self.bookmark_state.select(selected);
                        let _ = state::save_bookmarks(&self.paths, &self.bookmarks);
                        self.status = "Закладка удалена".to_string();
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_history_key(&mut self, key: KeyEvent) {
        let len = self.visit_history.len();
        match key.code {
            KeyCode::Esc | KeyCode::Char('H') => self.mode = Mode::Normal,
            KeyCode::Down | KeyCode::Char('j') => Self::move_list(&mut self.history_state, len, 1),
            KeyCode::Up | KeyCode::Char('k') => Self::move_list(&mut self.history_state, len, -1),
            KeyCode::Enter => self.open_selected_history(),
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(index) = self.history_state.selected() {
                    if index < self.visit_history.len() {
                        self.visit_history.remove(index);
                        self.history_state.select(self.visit_history.len().checked_sub(1));
                        let _ = state::save_history(&self.paths, &self.visit_history);
                    }
                }
            }
            KeyCode::Char('D') => {
                self.visit_history.clear();
                self.history_state.select(None);
                let _ = state::save_history(&self.paths, &self.visit_history);
                self.status = "История очищена".to_string();
            }
            _ => {}
        }
    }

    fn handle_forms_key(&mut self, key: KeyEvent) {
        let len = flatten_form_fields(self.current_page()).len();
        match key.code {
            KeyCode::Esc | KeyCode::Char('F') => self.mode = Mode::Normal,
            KeyCode::Down | KeyCode::Char('j') => Self::move_list(&mut self.form_state, len, 1),
            KeyCode::Up | KeyCode::Char('k') => Self::move_list(&mut self.form_state, len, -1),
            KeyCode::Enter => self.activate_form_field(),
            KeyCode::Char(' ') => {
                if let Some((form, field)) = self.selected_form_field() {
                    self.toggle_form_value(form, field);
                }
            }
            _ => {}
        }
    }

    fn handle_help_key(&mut self, key: KeyEvent) {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::Enter) {
            self.mode = Mode::Normal;
        }
    }

    fn input_insert(&mut self, c: char) {
        let index = byte_index_for_char(&self.input, self.input_cursor);
        self.input.insert(index, c);
        self.input_cursor += 1;
    }

    fn input_backspace(&mut self) {
        if self.input_cursor == 0 { return; }
        let start = byte_index_for_char(&self.input, self.input_cursor - 1);
        let end = byte_index_for_char(&self.input, self.input_cursor);
        self.input.replace_range(start..end, "");
        self.input_cursor -= 1;
    }

    fn input_delete(&mut self) {
        if self.input_cursor >= self.input.chars().count() { return; }
        let start = byte_index_for_char(&self.input, self.input_cursor);
        let end = byte_index_for_char(&self.input, self.input_cursor + 1);
        self.input.replace_range(start..end, "");
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Block::default().style(Style::default().bg(BG)), area);
        let [header, body, footer] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(4), Constraint::Length(1)])
            .areas(area);
        self.draw_header(frame, header);
        self.draw_body(frame, body);
        self.draw_footer(frame, footer);

        match self.mode {
            Mode::Help => self.draw_help(frame, area),
            Mode::Bookmarks => self.draw_bookmarks(frame, area),
            Mode::History => self.draw_history(frame, area),
            Mode::Forms | Mode::FormInput { .. } => self.draw_forms(frame, area),
            _ => {}
        }
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let [brand_area, tabs_area, input_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(3)])
            .areas(area);

        let brand = Line::from(vec![
            Span::styled(" NOX ", Style::default().fg(BG).bg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("  {}", self.current_page().title),
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("   {} cookies", self.cookies.count()),
                Style::default().fg(MUTED),
            ),
        ]);
        frame.render_widget(Paragraph::new(brand).style(Style::default().bg(BG)), brand_area);

        let mut tab_spans = Vec::new();
        for (index, tab) in self.tabs.iter().take(9).enumerate() {
            let mut title = tab.current_page().title.replace('\n', " ");
            if title.chars().count() > 16 {
                title = title.chars().take(15).collect::<String>() + "…";
            }
            let active = index == self.active_tab;
            tab_spans.push(Span::styled(
                format!(" {}:{} ", index + 1, title),
                if active {
                    Style::default().fg(BG).bg(ACCENT).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(MUTED).bg(PANEL)
                },
            ));
            tab_spans.push(Span::raw(" "));
        }
        frame.render_widget(Paragraph::new(Line::from(tab_spans)).style(Style::default().bg(BG)), tabs_area);

        let active_input = matches!(self.mode, Mode::Address | Mode::Search | Mode::FormInput { .. });
        let border_color = if active_input { ACCENT } else { ACCENT_SOFT };
        let title = match self.mode {
            Mode::Address => " ADDRESS · Enter открыть · Esc отмена ",
            Mode::Search => " FIND · Enter найти · Esc отмена ",
            Mode::FormInput { .. } => " FORM VALUE · Enter сохранить · Esc отмена ",
            _ => " ADDRESS · Ctrl+L / o ",
        };
        let content = match self.mode {
            Mode::Address | Mode::FormInput { .. } => input_with_cursor(&self.input, self.input_cursor),
            Mode::Search => format!("/ {}", input_with_cursor(&self.input, self.input_cursor)),
            _ => self.current_url_text(),
        };
        let block = Block::default()
            .title(Span::styled(title, Style::default().fg(border_color)))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(PANEL));
        frame.render_widget(Paragraph::new(content).style(Style::default().fg(TEXT).bg(PANEL)).block(block), input_area);
    }

    fn draw_body(&mut self, frame: &mut Frame, area: Rect) {
        if self.mode == Mode::Links && area.width >= 96 {
            let [reader, links] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(64), Constraint::Percentage(36)])
                .areas(area);
            self.draw_reader(frame, reader);
            self.draw_links(frame, links);
        } else {
            self.draw_reader(frame, area);
            if self.mode == Mode::Links {
                let popup = centered_rect(area, area.width.saturating_mul(4) / 5, area.height.saturating_mul(4) / 5);
                frame.render_widget(Clear, popup);
                self.draw_links(frame, popup);
            }
        }
    }

    fn draw_reader(&mut self, frame: &mut Frame, area: Rect) {
        self.viewport_height = area.height.saturating_sub(2).max(1);
        let match_line = self.tab().match_line;
        let mut rendered = Vec::with_capacity(self.current_page().lines.len());
        for (index, line) in self.current_page().lines.iter().enumerate() {
            rendered.push(style_reader_line(line, match_line == Some(index)));
        }
        if rendered.is_empty() {
            rendered.push(Line::from(Span::styled("Пустая страница", Style::default().fg(MUTED))));
        }
        let page = self.current_page();
        let mode = if page.reader_mode { "READER" } else { "DOCUMENT" };
        let title = match page.status_code {
            Some(code) => format!(" {mode} · HTTP {code} "),
            None => format!(" {mode} "),
        };
        let block = Block::default()
            .title(Span::styled(title, Style::default().fg(MUTED)))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(38, 45, 57)))
            .style(Style::default().bg(BG));
        let paragraph = Paragraph::new(rendered)
            .style(Style::default().fg(TEXT).bg(BG))
            .wrap(Wrap { trim: false })
            .scroll((self.tab().scroll, 0))
            .block(block);
        frame.render_widget(paragraph, area);
    }

    fn draw_links(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem<'_>> = self.current_page().links.iter().enumerate().map(|(index, link)| {
            let label = if link.label.trim().is_empty() {
                link.url.host_str().unwrap_or(link.url.as_str()).to_string()
            } else { link.label.clone() };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>3}  ", index + 1), Style::default().fg(MUTED)),
                Span::styled(label, Style::default().fg(TEXT)),
            ]))
        }).collect();
        let block = Block::default()
            .title(Span::styled(format!(" LINKS · {} · Enter open · d download ", items.len()), Style::default().fg(ACCENT)))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(ACCENT_SOFT))
            .style(Style::default().bg(PANEL));
        let list = List::new(items)
            .block(block)
            .highlight_symbol("› ")
            .highlight_style(Style::default().fg(Color::White).bg(Color::Rgb(20, 67, 86)).add_modifier(Modifier::BOLD));
        frame.render_stateful_widget(list, area, &mut self.link_state);
    }

    fn draw_bookmarks(&mut self, frame: &mut Frame, area: Rect) {
        let popup = centered_rect(area, area.width.saturating_mul(4) / 5, area.height.saturating_mul(4) / 5);
        frame.render_widget(Clear, popup);
        let items: Vec<ListItem<'_>> = self.bookmarks.iter().map(|bookmark| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{}  ", bookmark.title), Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
                Span::styled(bookmark.url.clone(), Style::default().fg(MUTED)),
            ]))
        }).collect();
        let list = List::new(items)
            .block(modal_block(" BOOKMARKS · Enter open · d delete "))
            .highlight_symbol("› ")
            .highlight_style(Style::default().fg(Color::White).bg(Color::Rgb(20, 67, 86)));
        frame.render_stateful_widget(list, popup, &mut self.bookmark_state);
    }

    fn draw_history(&mut self, frame: &mut Frame, area: Rect) {
        let popup = centered_rect(area, area.width.saturating_mul(4) / 5, area.height.saturating_mul(4) / 5);
        frame.render_widget(Clear, popup);
        let items: Vec<ListItem<'_>> = self.visit_history.iter().map(|entry| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{}  ", entry.title), Style::default().fg(TEXT)),
                Span::styled(entry.url.clone(), Style::default().fg(MUTED)),
            ]))
        }).collect();
        let list = List::new(items)
            .block(modal_block(" HISTORY · Enter open · d delete · D clear "))
            .highlight_symbol("› ")
            .highlight_style(Style::default().fg(Color::White).bg(Color::Rgb(20, 67, 86)));
        frame.render_stateful_widget(list, popup, &mut self.history_state);
    }

    fn draw_forms(&mut self, frame: &mut Frame, area: Rect) {
        let popup = centered_rect(area, area.width.saturating_mul(4) / 5, area.height.saturating_mul(4) / 5);
        frame.render_widget(Clear, popup);
        let refs = flatten_form_fields(self.current_page());
        let mut items = Vec::new();
        for (form_index, field_index) in refs {
            let field = &self.current_page().forms[form_index].fields[field_index];
            let marker = match field.kind {
                FormFieldKind::Checkbox | FormFieldKind::Radio if field.checked => "[x]",
                FormFieldKind::Checkbox | FormFieldKind::Radio => "[ ]",
                FormFieldKind::Select => "[▾]",
                FormFieldKind::Submit => "[→]",
                FormFieldKind::Password => "[••]",
                _ => "[ ]",
            };
            let value = match field.kind {
                FormFieldKind::Password => "•".repeat(field.value.chars().count().min(12)),
                FormFieldKind::Select => field.options.get(field.selected).map(|item| item.0.clone()).unwrap_or_default(),
                FormFieldKind::Submit => field.value.clone(),
                _ => field.value.clone(),
            };
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("{marker} "), Style::default().fg(ACCENT)),
                Span::styled(format!("{}  ", field.label), Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
                Span::styled(value, Style::default().fg(MUTED)),
            ])));
        }
        let list = List::new(items)
            .block(modal_block(" FORMS · Enter edit/submit · Space toggle "))
            .highlight_symbol("› ")
            .highlight_style(Style::default().fg(Color::White).bg(Color::Rgb(20, 67, 86)));
        frame.render_stateful_widget(list, popup, &mut self.form_state);
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(49)])
            .areas(area);
        frame.render_widget(
            Paragraph::new(format!(" {}", self.status)).style(Style::default().fg(MUTED).bg(BG)),
            left,
        );
        frame.render_widget(
            Paragraph::new("Ctrl+T tab · m bookmark · M/H lists · R reader · ? ")
                .alignment(Alignment::Right)
                .style(Style::default().fg(MUTED).bg(BG)),
            right,
        );
    }

    fn draw_help(&self, frame: &mut Frame, area: Rect) {
        let modal = centered_rect(area, 78, 31);
        frame.render_widget(Clear, modal);
        let lines = vec![
            Line::from(""),
            help_row("Ctrl+L / o", "адрес или веб-поиск"),
            help_row("Ctrl+T / Ctrl+W", "новая / закрыть вкладку"),
            help_row("Ctrl+Tab", "следующая вкладка"),
            help_row("Alt+1..9", "переключиться на вкладку"),
            help_row("Tab / l", "ссылки"),
            help_row("d (в Links)", "скачать выбранную ссылку"),
            help_row("/ / n", "поиск / следующее совпадение"),
            help_row("b / f", "назад / вперёд"),
            help_row("r / R", "reload / Reader mode"),
            help_row("m", "добавить/удалить закладку"),
            help_row("M", "закладки"),
            help_row("H", "общая история"),
            help_row("F", "формы текущей страницы"),
            help_row("j/k · PgUp/PgDn", "прокрутка"),
            help_row("?", "справка"),
            help_row("q / Ctrl+C", "выход"),
            Line::from(""),
            Line::from(Span::styled(
                "NOX 0.4 хранит config, bookmarks, history, session и cookies между запусками.",
                Style::default().fg(MUTED),
            )),
            Line::from(Span::styled(
                "JavaScript/CSS layout по-прежнему не исполняются — NOX остаётся reader-first TUI.",
                Style::default().fg(MUTED),
            )),
        ];
        frame.render_widget(
            Paragraph::new(lines).style(Style::default().fg(TEXT).bg(PANEL)).block(modal_block(" NOX 0.4 · KEYBOARD ")),
            modal,
        );
    }
}

fn build_client(config: &AppConfig, cookies: Arc<PersistentCookieStore>) -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(25))
        .redirect(Policy::limited(10))
        .user_agent(config.user_agent.clone())
        .cookie_provider(cookies)
        .build()
        .context("не удалось создать HTTP-клиент")
}

fn fetch_page(client: &Client, url: Url, reader_mode: bool) -> Result<Page> {
    if !matches!(url.scheme(), "http" | "https") {
        anyhow::bail!("поддерживаются только HTTP и HTTPS");
    }
    let response = client
        .get(url.clone())
        .send()
        .with_context(|| format!("не удалось загрузить {url}"))?;
    response_to_page(response, reader_mode)
}

fn fetch_form(client: &Client, form: &PageForm, submit_field: Option<usize>, reader_mode: bool) -> Result<Page> {
    let pairs = form_pairs(form, submit_field);
    let request: RequestBuilder = if form.method.eq_ignore_ascii_case("POST") {
        let body = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(pairs.iter().map(|(key, value)| (key.as_str(), value.as_str())))
            .finish();
        client
            .post(form.action.clone())
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
    } else {
        let mut target = form.action.clone();
        target.query_pairs_mut().extend_pairs(pairs.iter().map(|(key, value)| (key.as_str(), value.as_str())));
        client.get(target)
    };
    let response = request.send().context("не удалось отправить форму")?;
    response_to_page(response, reader_mode)
}

fn response_to_page(response: Response, reader_mode: bool) -> Result<Page> {
    let status = response.status().as_u16();
    let final_url = response.url().clone();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    if content_type.contains("application/json") || content_type.contains("+json") {
        let body = response.text().context("не удалось прочитать JSON")?;
        let pretty = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|value| serde_json::to_string_pretty(&value).ok())
            .unwrap_or(body);
        return Ok(Page {
            title: final_url.host_str().unwrap_or("JSON").to_string(),
            display_url: final_url.as_str().to_string(),
            url: Some(final_url),
            lines: pretty.lines().map(|line| format!("    {line}")).collect(),
            links: Vec::new(),
            forms: Vec::new(),
            status_code: Some(status),
            raw_html: None,
            reader_mode: false,
        });
    }

    if !content_type.is_empty()
        && !content_type.contains("text/html")
        && !content_type.contains("application/xhtml+xml")
        && !content_type.contains("text/plain")
    {
        return Ok(Page {
            title: "Неподдерживаемый документ".to_string(),
            display_url: final_url.as_str().to_string(),
            url: Some(final_url),
            lines: vec![
                "Этот ресурс не является HTML, текстом или JSON.".to_string(),
                String::new(),
                format!("Content-Type: {content_type}"),
                String::new(),
                "Откройте панель ссылок и нажмите d, чтобы скачать бинарный ресурс.".to_string(),
            ],
            links: Vec::new(),
            forms: Vec::new(),
            status_code: Some(status),
            raw_html: None,
            reader_mode: false,
        });
    }

    let body = response.text().context("не удалось прочитать тело ответа")?;
    if content_type.contains("text/plain") {
        let mut lines: Vec<String> = body.lines().map(ToString::to_string).collect();
        if lines.is_empty() { lines.push(String::new()); }
        return Ok(Page {
            title: final_url.host_str().unwrap_or("Текстовый документ").to_string(),
            display_url: final_url.as_str().to_string(),
            url: Some(final_url),
            lines,
            links: Vec::new(),
            forms: Vec::new(),
            status_code: Some(status),
            raw_html: None,
            reader_mode: false,
        });
    }

    Ok(parse_html_page(final_url, status, &body, reader_mode))
}

fn parse_html_page(url: Url, status: u16, html: &str, reader_mode: bool) -> Page {
    let document = Html::parse_document(html);
    let title_selector = Selector::parse("title").expect("valid selector");
    let link_selector = Selector::parse("a[href]").expect("valid selector");
    let body_selector = Selector::parse("body").expect("valid selector");
    let article_selector = Selector::parse("article").expect("valid selector");
    let main_selector = Selector::parse("main").expect("valid selector");

    let title = document
        .select(&title_selector)
        .next()
        .map(|node| normalize_ws(&node.text().collect::<Vec<_>>().join(" ")))
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| url.host_str().unwrap_or("Без названия").to_string());

    let root = if reader_mode {
        document
            .select(&article_selector)
            .next()
            .or_else(|| document.select(&main_selector).next())
            .or_else(|| document.select(&body_selector).next())
    } else {
        document.select(&body_selector).next()
    };

    let mut lines = root.map(collect_content_lines).unwrap_or_default();
    if lines.is_empty() {
        if let Some(body) = document.select(&body_selector).next() {
            let text = normalize_ws(&body.text().collect::<Vec<_>>().join(" "));
            if !text.is_empty() { lines.push(text); }
        }
    }
    if lines.is_empty() {
        lines.push("Страница не содержит отображаемого текста.".to_string());
    }

    let mut links = Vec::new();
    let mut seen = HashSet::new();
    for element in document.select(&link_selector) {
        let Some(href) = element.value().attr("href") else { continue; };
        if href.starts_with('#') || href.starts_with("javascript:") || href.starts_with("mailto:") || href.starts_with("tel:") {
            continue;
        }
        let Ok(target) = url.join(href) else { continue; };
        if !matches!(target.scheme(), "http" | "https") { continue; }
        if !seen.insert(target.as_str().to_string()) { continue; }
        let label = normalize_ws(&element.text().collect::<Vec<_>>().join(" "));
        links.push(LinkTarget { label, url: target });
        if links.len() >= 500 { break; }
    }

    let forms = extract_forms(&document, &url);
    if !forms.is_empty() {
        push_blank(&mut lines);
        lines.push(format!("### Forms: {} · press F", forms.len()));
    }

    Page {
        title,
        display_url: url.as_str().to_string(),
        url: Some(url),
        lines,
        links,
        forms,
        status_code: Some(status),
        raw_html: Some(html.to_string()),
        reader_mode,
    }
}

fn collect_content_lines(root: ElementRef<'_>) -> Vec<String> {
    let selector = Selector::parse("h1,h2,h3,h4,h5,h6,p,li,pre,blockquote,table").expect("valid selector");
    let mut lines = Vec::new();
    for element in root.select(&selector) {
        let tag = element.value().name();
        if tag == "pre" {
            let raw = element.text().collect::<Vec<_>>().join("");
            let raw = raw.trim_matches('\n');
            if raw.is_empty() { continue; }
            for line in raw.lines() { lines.push(format!("    {}", line.trim_end())); }
            push_blank(&mut lines);
            continue;
        }
        if tag == "table" {
            render_table(element, &mut lines);
            continue;
        }
        let text = normalize_ws(&element.text().collect::<Vec<_>>().join(" "));
        if text.is_empty() { continue; }
        match tag {
            "h1" => { push_blank(&mut lines); lines.push(format!("# {text}")); push_blank(&mut lines); }
            "h2" => { push_blank(&mut lines); lines.push(format!("## {text}")); push_blank(&mut lines); }
            "h3" | "h4" | "h5" | "h6" => { push_blank(&mut lines); lines.push(format!("### {text}")); }
            "li" => lines.push(format!("• {text}")),
            "blockquote" => { lines.push(format!("│ {text}")); push_blank(&mut lines); }
            "p" => { lines.push(text); push_blank(&mut lines); }
            _ => lines.push(text),
        }
    }
    while lines.last().is_some_and(|line| line.is_empty()) { lines.pop(); }
    lines
}

fn render_table(table: ElementRef<'_>, lines: &mut Vec<String>) {
    let row_selector = Selector::parse("tr").expect("valid selector");
    let cell_selector = Selector::parse("th,td").expect("valid selector");
    push_blank(lines);
    for row in table.select(&row_selector).take(50) {
        let cells: Vec<String> = row
            .select(&cell_selector)
            .map(|cell| normalize_ws(&cell.text().collect::<Vec<_>>().join(" ")))
            .filter(|cell| !cell.is_empty())
            .collect();
        if !cells.is_empty() {
            lines.push(format!("│ {} │", cells.join(" │ ")));
        }
    }
    push_blank(lines);
}

fn extract_forms(document: &Html, base: &Url) -> Vec<PageForm> {
    let form_selector = Selector::parse("form").expect("valid selector");
    let field_selector = Selector::parse("input,textarea,select,button").expect("valid selector");
    let option_selector = Selector::parse("option").expect("valid selector");
    let mut forms = Vec::new();

    for form in document.select(&form_selector).take(20) {
        let action_raw = form.value().attr("action").unwrap_or(base.as_str());
        let action = base.join(action_raw).unwrap_or_else(|_| base.clone());
        if !matches!(action.scheme(), "http" | "https") { continue; }
        let method = form.value().attr("method").unwrap_or("GET").to_ascii_uppercase();
        let method = if method == "POST" { "POST" } else { "GET" }.to_string();
        let mut fields = Vec::new();

        for element in form.select(&field_selector).take(100) {
            let tag = element.value().name();
            let name = element.value().attr("name").unwrap_or("").to_string();
            let placeholder = element.value().attr("placeholder").unwrap_or("");
            let aria = element.value().attr("aria-label").unwrap_or("");
            let mut label = if !aria.is_empty() { aria.to_string() } else if !placeholder.is_empty() { placeholder.to_string() } else if !name.is_empty() { name.clone() } else { tag.to_string() };

            let mut field = if tag == "textarea" {
                FormField {
                    name,
                    label,
                    value: element.text().collect::<Vec<_>>().join(""),
                    kind: FormFieldKind::TextArea,
                    checked: false,
                    options: Vec::new(),
                    selected: 0,
                }
            } else if tag == "select" {
                let options: Vec<(String, String)> = element.select(&option_selector).map(|option| {
                    let text = normalize_ws(&option.text().collect::<Vec<_>>().join(" "));
                    let value = option.value().attr("value").map(str::to_string).unwrap_or_else(|| text.clone());
                    (text, value)
                }).collect();
                let selected = element.select(&option_selector).position(|option| option.value().attr("selected").is_some()).unwrap_or(0).min(options.len().saturating_sub(1));
                let value = options.get(selected).map(|item| item.1.clone()).unwrap_or_default();
                FormField { name, label, value, kind: FormFieldKind::Select, checked: false, options, selected }
            } else {
                let kind_name = if tag == "button" { element.value().attr("type").unwrap_or("submit") } else { element.value().attr("type").unwrap_or("text") };
                let kind = match kind_name.to_ascii_lowercase().as_str() {
                    "password" => FormFieldKind::Password,
                    "hidden" => FormFieldKind::Hidden,
                    "checkbox" => FormFieldKind::Checkbox,
                    "radio" => FormFieldKind::Radio,
                    "submit" | "image" => FormFieldKind::Submit,
                    _ => FormFieldKind::Text,
                };
                let value = element.value().attr("value").unwrap_or(if kind == FormFieldKind::Submit { "Submit" } else { "" }).to_string();
                if label == "button" || label == "input" { label = value.clone(); }
                FormField {
                    name,
                    label,
                    value,
                    kind,
                    checked: element.value().attr("checked").is_some(),
                    options: Vec::new(),
                    selected: 0,
                }
            };
            if field.kind == FormFieldKind::Checkbox && field.value.is_empty() { field.value = "on".to_string(); }
            fields.push(field);
        }

        if !fields.is_empty() { forms.push(PageForm { action, method, fields }); }
    }
    forms
}

fn form_pairs(form: &PageForm, submit_field: Option<usize>) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for (index, field) in form.fields.iter().enumerate() {
        if field.name.is_empty() { continue; }
        let include = match field.kind {
            FormFieldKind::Checkbox | FormFieldKind::Radio => field.checked,
            FormFieldKind::Submit => submit_field == Some(index),
            _ => true,
        };
        if include {
            let value = if field.kind == FormFieldKind::Select {
                field.options.get(field.selected).map(|item| item.1.clone()).unwrap_or_else(|| field.value.clone())
            } else { field.value.clone() };
            pairs.push((field.name.clone(), value));
        }
    }
    pairs
}

fn flatten_form_fields(page: &Page) -> Vec<(usize, usize)> {
    let mut refs = Vec::new();
    for (form_index, form) in page.forms.iter().enumerate() {
        for (field_index, field) in form.fields.iter().enumerate() {
            if field.kind != FormFieldKind::Hidden {
                refs.push((form_index, field_index));
            }
        }
    }
    refs
}

fn download_url(client: &Client, url: &Url, dir: &Path) -> Result<(PathBuf, usize)> {
    let response = client.get(url.clone()).send().with_context(|| format!("не удалось скачать {url}"))?;
    let disposition = response.headers().get(CONTENT_DISPOSITION).and_then(|value| value.to_str().ok()).unwrap_or("");
    let filename = disposition
        .split(';')
        .map(str::trim)
        .find_map(|part| part.strip_prefix("filename=").map(|value| value.trim_matches('"').to_string()))
        .filter(|value| !value.is_empty())
        .or_else(|| url.path_segments().and_then(|mut segments| segments.next_back()).filter(|value| !value.is_empty()).map(ToString::to_string))
        .unwrap_or_else(|| "download.bin".to_string());
    let filename = sanitize_filename(&filename);
    fs::create_dir_all(dir).with_context(|| format!("не удалось создать {}", dir.display()))?;
    let mut path = dir.join(&filename);
    let mut suffix = 1;
    while path.exists() {
        let stem = Path::new(&filename).file_stem().and_then(|value| value.to_str()).unwrap_or("download");
        let ext = Path::new(&filename).extension().and_then(|value| value.to_str());
        let candidate = match ext {
            Some(ext) => format!("{stem}-{suffix}.{ext}"),
            None => format!("{stem}-{suffix}"),
        };
        path = dir.join(candidate);
        suffix += 1;
    }
    let bytes = response.bytes().context("не удалось прочитать загружаемый файл")?;
    fs::write(&path, &bytes).with_context(|| format!("не удалось записать {}", path.display()))?;
    Ok((path, bytes.len()))
}

fn sanitize_filename(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|c| if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c })
        .collect();
    let cleaned = cleaned.trim_matches(|c| c == ' ' || c == '.');
    if cleaned.is_empty() { "download.bin".to_string() } else { cleaned.to_string() }
}

fn resolve_user_input(raw: &str, config: &AppConfig) -> Result<Url> {
    let raw = raw.trim();
    if raw.is_empty() { anyhow::bail!("пустой адрес"); }
    if raw == "about:home" { anyhow::bail!("about:home используется только внутри NOX"); }
    if let Ok(url) = Url::parse(raw) {
        if matches!(url.scheme(), "http" | "https") { return Ok(url); }
    }
    let local_host = !raw.chars().any(char::is_whitespace)
        && (raw.starts_with("localhost") || raw.starts_with("127.0.0.1") || raw.starts_with("[::1]"));
    if local_host { return Url::parse(&format!("http://{raw}")).context("некорректный локальный адрес"); }
    let looks_like_host = !raw.chars().any(char::is_whitespace) && (raw.contains('.') || raw.contains(':'));
    if looks_like_host { return Url::parse(&format!("https://{raw}")).context("некорректный адрес"); }

    let encoded = form_urlencoded::byte_serialize(raw.as_bytes()).collect::<String>();
    let template = if config.search_engine.contains("{query}") {
        config.search_engine.clone()
    } else {
        "https://html.duckduckgo.com/html/?q={query}".to_string()
    };
    Url::parse(&template.replace("{query}", &encoded)).context("некорректный search_engine в config.toml")
}

fn home_page() -> Page {
    Page {
        title: "New tab".to_string(),
        display_url: "about:home".to_string(),
        url: None,
        status_code: None,
        links: Vec::new(),
        forms: Vec::new(),
        raw_html: None,
        reader_mode: true,
        lines: vec![
            "# NOX 0.4".to_string(),
            String::new(),
            "Portable reader-first terminal browser.".to_string(),
            String::new(),
            "Ctrl+L — адрес или поиск     Ctrl+T — новая вкладка".to_string(),
            "Tab — ссылки                m / M — bookmark / bookmarks".to_string(),
            "H — история                 F — формы".to_string(),
            "R — Reader mode             ? — все клавиши".to_string(),
            String::new(),
            "Tabs. History. Bookmarks. Cookies. Forms. Downloads. Sessions.".to_string(),
        ],
    }
}

fn error_page(target: Url, err: &anyhow::Error) -> Page {
    Page {
        title: "Ошибка загрузки".to_string(),
        display_url: target.as_str().to_string(),
        url: Some(target),
        status_code: None,
        links: Vec::new(),
        forms: Vec::new(),
        raw_html: None,
        reader_mode: true,
        lines: vec![
            "# Не удалось открыть страницу".to_string(),
            String::new(),
            format!("{err:#}"),
            String::new(),
            "Проверьте адрес, подключение или повторите клавишей r.".to_string(),
        ],
    }
}

fn style_reader_line(line: &str, is_match: bool) -> Line<'static> {
    let base_bg = if is_match { Color::Rgb(45, 43, 20) } else { BG };
    if let Some(text) = line.strip_prefix("# ") {
        return Line::from(Span::styled(text.to_string(), Style::default().fg(Color::White).bg(base_bg).add_modifier(Modifier::BOLD)));
    }
    if let Some(text) = line.strip_prefix("## ") {
        return Line::from(Span::styled(text.to_string(), Style::default().fg(ACCENT).bg(base_bg).add_modifier(Modifier::BOLD)));
    }
    if let Some(text) = line.strip_prefix("### ") {
        return Line::from(Span::styled(text.to_string(), Style::default().fg(Color::Rgb(125, 211, 252)).bg(base_bg).add_modifier(Modifier::BOLD)));
    }
    if let Some(text) = line.strip_prefix("• ") {
        return Line::from(vec![Span::styled("• ", Style::default().fg(ACCENT).bg(base_bg)), Span::styled(text.to_string(), Style::default().fg(TEXT).bg(base_bg))]);
    }
    if let Some(text) = line.strip_prefix("│ ") {
        return Line::from(vec![Span::styled("│ ", Style::default().fg(ACCENT).bg(base_bg)), Span::styled(text.to_string(), Style::default().fg(MUTED).bg(base_bg).add_modifier(Modifier::ITALIC))]);
    }
    if line.starts_with("    ") {
        return Line::from(Span::styled(line.to_string(), Style::default().fg(Color::Rgb(186, 230, 253)).bg(Color::Rgb(12, 22, 30))));
    }
    if line.starts_with('│') && line.ends_with('│') {
        return Line::from(Span::styled(line.to_string(), Style::default().fg(Color::Rgb(165, 243, 252)).bg(base_bg)));
    }
    Line::from(Span::styled(line.to_string(), Style::default().fg(TEXT).bg(base_bg)))
}

fn help_row<'a>(key: &'a str, description: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("  {:<20}", key), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled(description, Style::default().fg(TEXT)),
    ])
}

fn modal_block(title: &'static str) -> Block<'static> {
    Block::default()
        .title(Span::styled(title, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(PANEL))
}

fn input_with_cursor(input: &str, cursor: usize) -> String {
    let index = byte_index_for_char(input, cursor);
    let mut rendered = String::with_capacity(input.len() + 3);
    rendered.push_str(&input[..index]);
    rendered.push('▏');
    rendered.push_str(&input[index..]);
    rendered
}

fn byte_index_for_char(value: &str, char_index: usize) -> usize {
    value.char_indices().nth(char_index).map(|(index, _)| index).unwrap_or(value.len())
}

fn normalize_ws(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn push_blank(lines: &mut Vec<String>) {
    if lines.last().is_some_and(|line| !line.is_empty()) { lines.push(String::new()); }
}

fn centered_rect(area: Rect, requested_width: u16, requested_height: u16) -> Rect {
    let width = requested_width.max(1).min(area.width.max(1));
    let height = requested_height.max(1).min(area.height.max(1));
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

fn dump_page(raw: &str) -> Result<()> {
    let paths = Paths::discover()?;
    let config = state::load_config(&paths)?;
    let cookies = Arc::new(PersistentCookieStore::load(paths.cookies.clone()));
    let client = build_client(&config, Arc::clone(&cookies))?;
    let target = resolve_user_input(raw, &config)?;
    let page = fetch_page(&client, target, config.reader_mode)?;
    println!("# {}", page.title);
    println!("{}", page.display_url);
    println!();
    for line in page.lines { println!("{line}"); }
    if !page.links.is_empty() {
        println!();
        println!("--- LINKS ({}) ---", page.links.len());
        for (index, link) in page.links.iter().enumerate() {
            let label = if link.label.trim().is_empty() { link.url.as_str() } else { &link.label };
            println!("{:>3}. {} -> {}", index + 1, label, link.url);
        }
    }
    cookies.save();
    Ok(())
}

fn print_cli_help() {
    println!(
        "NOX {} — portable reader-first terminal browser\n\n\
         USAGE:\n  nox [URL | search query]\n  nox --dump <URL | search query>\n  nox install\n  nox uninstall\n  nox update [--check]\n  nox config --path\n  nox data --path\n  nox cookies clear\n  nox --version\n\n\
         TUI ESSENTIALS:\n  Ctrl+T/Ctrl+W   tabs\n  Ctrl+L          address/search\n  m / M           toggle bookmark / bookmarks\n  H               history\n  F               forms\n  R               reader mode\n  Tab             links\n\n\
         EXAMPLES:\n  nox example.com\n  nox rust terminal browser\n  nox --dump https://example.com\n  nox install\n  nox update --check",
        env!("CARGO_PKG_VERSION")
    );
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        print_cli_help();
        return Ok(());
    }
    if matches!(args.first().map(String::as_str), Some("--version" | "-V")) {
        println!("nox {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.first().map(String::as_str) == Some("install") {
        return install::install_self();
    }
    if args.first().map(String::as_str) == Some("uninstall") {
        return install::uninstall_self();
    }
    if matches!(args.first().map(String::as_str), Some("update" | "self-update")) {
        let check_only = args.iter().skip(1).any(|arg| arg == "--check" || arg == "-c");
        return update::run(check_only);
    }
    if args.first().map(String::as_str) == Some("config") && args.get(1).map(String::as_str) == Some("--path") {
        println!("{}", Paths::discover()?.config.display());
        return Ok(());
    }
    if args.first().map(String::as_str) == Some("data") && args.get(1).map(String::as_str) == Some("--path") {
        println!("{}", Paths::discover()?.root.display());
        return Ok(());
    }
    if args.first().map(String::as_str) == Some("cookies") && args.get(1).map(String::as_str) == Some("clear") {
        let paths = Paths::discover()?;
        let store = PersistentCookieStore::load(paths.cookies);
        println!("Удалено cookies: {}", store.clear());
        return Ok(());
    }

    let explicit_dump = matches!(args.first().map(String::as_str), Some("--dump" | "-d"));
    let value_args = if explicit_dump { &args[1..] } else { &args[..] };
    let initial = (!value_args.is_empty()).then(|| value_args.join(" "));
    let interactive_terminal = io::stdin().is_terminal() && io::stdout().is_terminal();
    if explicit_dump || !interactive_terminal {
        let Some(value) = initial else { print_cli_help(); return Ok(()); };
        return dump_page(&value);
    }

    let mut app = App::new(initial)?;
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();
    app.persist();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AppConfig { AppConfig::default() }

    #[test]
    fn url_without_scheme_gets_https() {
        let url = resolve_user_input("example.com", &config()).unwrap();
        assert_eq!(url.as_str(), "https://example.com/");
    }

    #[test]
    fn localhost_defaults_to_http() {
        let url = resolve_user_input("localhost:5173", &config()).unwrap();
        assert_eq!(url.as_str(), "http://localhost:5173/");
    }

    #[test]
    fn plain_text_becomes_search() {
        let url = resolve_user_input("rust terminal browser", &config()).unwrap();
        assert_eq!(url.host_str(), Some("html.duckduckgo.com"));
        assert!(url.query().unwrap_or_default().contains("rust+terminal+browser") || url.query().unwrap_or_default().contains("rust%20terminal%20browser"));
    }

    #[test]
    fn html_tables_render() {
        let page = parse_html_page(
            Url::parse("https://example.com").unwrap(),
            200,
            "<html><body><table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table></body></html>",
            false,
        );
        assert!(page.lines.iter().any(|line| line.contains("A") && line.contains("B")));
    }

    #[test]
    fn forms_are_extracted() {
        let page = parse_html_page(
            Url::parse("https://example.com/search").unwrap(),
            200,
            "<html><body><form action='/q'><input name='q' placeholder='Search'><button type='submit'>Go</button></form></body></html>",
            true,
        );
        assert_eq!(page.forms.len(), 1);
        assert_eq!(page.forms[0].fields.len(), 2);
    }

    #[test]
    fn filename_is_sanitized() {
        assert_eq!(sanitize_filename("a:b?.zip"), "a_b_.zip");
    }
}
