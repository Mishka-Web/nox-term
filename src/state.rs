use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub homepage: String,
    pub search_engine: String,
    pub restore_session: bool,
    pub reader_mode: bool,
    pub max_history: usize,
    pub user_agent: String,
    pub download_dir: Option<String>,
    pub visual_mode: bool,
    pub load_images: bool,
    pub max_images: usize,
    pub image_width: u32,
    pub image_max_bytes: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            homepage: "about:home".to_string(),
            search_engine: "https://lite.duckduckgo.com/lite/?q={query}".to_string(),
            restore_session: true,
            reader_mode: true,
            max_history: 1_000,
            user_agent: format!("NOX/{} terminal-browser", env!("CARGO_PKG_VERSION")),
            download_dir: None,
            visual_mode: true,
            load_images: true,
            max_images: 8,
            image_width: 48,
            image_max_bytes: 2_000_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub title: String,
    pub url: String,
    pub visited_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionData {
    pub tabs: Vec<String>,
    pub active_tab: usize,
}

#[derive(Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,
    pub config: PathBuf,
    pub bookmarks: PathBuf,
    pub history: PathBuf,
    pub session: PathBuf,
    pub cookies: PathBuf,
    pub downloads: PathBuf,
}

impl Paths {
    pub fn discover() -> Result<Self> {
        let root = app_data_dir()?;
        let downloads = default_download_dir().unwrap_or_else(|| root.join("downloads"));
        Ok(Self {
            config: root.join("config.toml"),
            bookmarks: root.join("bookmarks.json"),
            history: root.join("history.json"),
            session: root.join("session.json"),
            cookies: root.join("cookies.json"),
            downloads,
            root,
        })
    }

    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.root)
            .with_context(|| format!("не удалось создать {}", self.root.display()))?;
        Ok(())
    }
}

pub fn load_config(paths: &Paths) -> Result<AppConfig> {
    paths.ensure()?;
    if !paths.config.exists() {
        let config = AppConfig::default();
        save_config(paths, &config)?;
        return Ok(config);
    }

    let raw = fs::read_to_string(&paths.config)
        .with_context(|| format!("не удалось прочитать {}", paths.config.display()))?;
    let mut config = toml::from_str::<AppConfig>(&raw)
        .with_context(|| format!("ошибка в {}", paths.config.display()))?;

    let mut changed = false;

    // NOX 0.5 migrated the built-in DuckDuckGo provider to the Lite UI.
    // Preserve custom search engines, but upgrade the exact old default automatically.
    if config.search_engine == "https://html.duckduckgo.com/html/?q={query}" {
        config.search_engine = "https://lite.duckduckgo.com/lite/?q={query}".to_string();
        changed = true;
    }

    // NOX 0.6 added Visual Mode settings. Serde fills missing fields from Default;
    // write them back once so existing users can discover and tune them in config.toml.
    for key in ["visual_mode", "load_images", "max_images", "image_width", "image_max_bytes"] {
        if !raw.lines().any(|line| line.trim_start().starts_with(&format!("{key} ="))) {
            changed = true;
            break;
        }
    }

    if changed {
        save_config(paths, &config)?;
    }

    Ok(config)
}

pub fn save_config(paths: &Paths, config: &AppConfig) -> Result<()> {
    paths.ensure()?;
    let raw = toml::to_string_pretty(config).context("не удалось сериализовать config.toml")?;
    atomic_write(&paths.config, raw.as_bytes())
}

pub fn load_bookmarks(paths: &Paths) -> Result<Vec<Bookmark>> {
    load_json_or_default(&paths.bookmarks)
}

pub fn save_bookmarks(paths: &Paths, value: &[Bookmark]) -> Result<()> {
    save_json(&paths.bookmarks, value)
}

pub fn load_history(paths: &Paths) -> Result<Vec<HistoryEntry>> {
    load_json_or_default(&paths.history)
}

pub fn save_history(paths: &Paths, value: &[HistoryEntry]) -> Result<()> {
    save_json(&paths.history, value)
}

pub fn load_session(paths: &Paths) -> Result<SessionData> {
    load_json_or_default(&paths.session)
}

pub fn save_session(paths: &Paths, value: &SessionData) -> Result<()> {
    save_json(&paths.session, value)
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn app_data_dir() -> Result<PathBuf> {
    if let Ok(dir) = env::var("NOX_DATA_DIR") {
        let dir = dir.trim();
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }

    #[cfg(windows)]
    {
        if let Ok(local) = env::var("LOCALAPPDATA") {
            return Ok(PathBuf::from(local).join("NOX"));
        }
        if let Ok(home) = env::var("USERPROFILE") {
            return Ok(PathBuf::from(home).join("AppData").join("Local").join("NOX"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("NOX"));
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
            if !xdg.trim().is_empty() {
                return Ok(PathBuf::from(xdg).join("nox"));
            }
        }
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home).join(".config").join("nox"));
        }
    }

    Err(anyhow::anyhow!(
        "не удалось определить каталог данных NOX; задайте NOX_DATA_DIR"
    ))
}

fn default_download_dir() -> Option<PathBuf> {
    if let Ok(dir) = env::var("NOX_DOWNLOAD_DIR") {
        let dir = dir.trim();
        if !dir.is_empty() {
            return Some(PathBuf::from(dir));
        }
    }

    #[cfg(windows)]
    let home = env::var("USERPROFILE").ok();
    #[cfg(not(windows))]
    let home = env::var("HOME").ok();

    home.map(PathBuf::from).map(|home| home.join("Downloads"))
}

fn load_json_or_default<T>(path: &Path) -> Result<T>
where
    T: serde::de::DeserializeOwned + Default,
{
    if !path.exists() {
        return Ok(T::default());
    }
    let file = fs::File::open(path)
        .with_context(|| format!("не удалось открыть {}", path.display()))?;
    serde_json::from_reader(file)
        .with_context(|| format!("ошибка формата {}", path.display()))
}

fn save_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_vec_pretty(value).context("не удалось сериализовать JSON")?;
    atomic_write(path, &raw)
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, data)
        .with_context(|| format!("не удалось записать {}", tmp.display()))?;
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    fs::rename(&tmp, path)
        .with_context(|| format!("не удалось заменить {}", path.display()))?;
    Ok(())
}
