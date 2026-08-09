use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
};

use reqwest::{
    cookie::CookieStore,
    header::HeaderValue,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredCookie {
    name: String,
    value: String,
    domain: String,
    path: String,
    secure: bool,
    host_only: bool,
    persistent: bool,
}

#[derive(Debug)]
pub struct PersistentCookieStore {
    path: PathBuf,
    cookies: Mutex<Vec<StoredCookie>>,
}

impl PersistentCookieStore {
    pub fn load(path: PathBuf) -> Self {
        let cookies = fs::File::open(&path)
            .ok()
            .and_then(|file| serde_json::from_reader::<_, Vec<StoredCookie>>(file).ok())
            .unwrap_or_default();
        Self {
            path,
            cookies: Mutex::new(cookies),
        }
    }

    pub fn save(&self) {
        let Ok(cookies) = self.cookies.lock() else {
            return;
        };
        let persistent: Vec<&StoredCookie> = cookies.iter().filter(|cookie| cookie.persistent).collect();
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(file) = fs::File::create(&self.path) {
            let _ = serde_json::to_writer_pretty(file, &persistent);
        }
    }

    pub fn clear(&self) -> usize {
        let Ok(mut cookies) = self.cookies.lock() else {
            return 0;
        };
        let count = cookies.len();
        cookies.clear();
        let _ = fs::remove_file(&self.path);
        count
    }

    pub fn count(&self) -> usize {
        self.cookies.lock().map(|cookies| cookies.len()).unwrap_or(0)
    }
}

impl Drop for PersistentCookieStore {
    fn drop(&mut self) {
        self.save();
    }
}

impl CookieStore for PersistentCookieStore {
    fn set_cookies(
        &self,
        cookie_headers: &mut dyn Iterator<Item = &HeaderValue>,
        url: &Url,
    ) {
        let Some(host) = url.host_str().map(|host| host.to_ascii_lowercase()) else {
            return;
        };
        let default_path = default_cookie_path(url);

        let Ok(mut store) = self.cookies.lock() else {
            return;
        };

        for header in cookie_headers {
            let Ok(raw) = header.to_str() else {
                continue;
            };
            let mut parts = raw.split(';');
            let Some(pair) = parts.next() else {
                continue;
            };
            let Some((name, value)) = pair.split_once('=') else {
                continue;
            };
            let name = name.trim();
            if name.is_empty() {
                continue;
            }

            let mut cookie = StoredCookie {
                name: name.to_string(),
                value: value.trim().to_string(),
                domain: host.clone(),
                path: default_path.clone(),
                secure: false,
                host_only: true,
                persistent: false,
            };
            let mut delete = false;

            for attr in parts {
                let attr = attr.trim();
                let (key, value) = attr
                    .split_once('=')
                    .map(|(k, v)| (k.trim().to_ascii_lowercase(), Some(v.trim())))
                    .unwrap_or_else(|| (attr.to_ascii_lowercase(), None));

                match key.as_str() {
                    "domain" => {
                        if let Some(value) = value {
                            let domain = value.trim_start_matches('.').to_ascii_lowercase();
                            if domain_matches(&host, &domain, false) {
                                cookie.domain = domain;
                                cookie.host_only = false;
                            }
                        }
                    }
                    "path" => {
                        if let Some(value) = value {
                            if value.starts_with('/') {
                                cookie.path = value.to_string();
                            }
                        }
                    }
                    "secure" => cookie.secure = true,
                    "expires" => cookie.persistent = true,
                    "max-age" => {
                        cookie.persistent = true;
                        if value.and_then(|v| v.parse::<i64>().ok()).is_some_and(|age| age <= 0) {
                            delete = true;
                        }
                    }
                    _ => {}
                }
            }

            store.retain(|existing| {
                !(existing.name == cookie.name
                    && existing.domain == cookie.domain
                    && existing.path == cookie.path)
            });

            if !delete {
                store.push(cookie);
            }
        }
    }

    fn cookies(&self, url: &Url) -> Option<HeaderValue> {
        let host = url.host_str()?.to_ascii_lowercase();
        let request_path = if url.path().is_empty() { "/" } else { url.path() };
        let secure_request = url.scheme() == "https";
        let store = self.cookies.lock().ok()?;

        let value = store
            .iter()
            .filter(|cookie| !cookie.secure || secure_request)
            .filter(|cookie| domain_matches(&host, &cookie.domain, cookie.host_only))
            .filter(|cookie| path_matches(request_path, &cookie.path))
            .map(|cookie| format!("{}={}", cookie.name, cookie.value))
            .collect::<Vec<_>>()
            .join("; ");

        if value.is_empty() {
            None
        } else {
            HeaderValue::from_str(&value).ok()
        }
    }
}

fn domain_matches(host: &str, domain: &str, host_only: bool) -> bool {
    if host_only {
        return host.eq_ignore_ascii_case(domain);
    }
    host.eq_ignore_ascii_case(domain)
        || host
            .strip_suffix(domain)
            .is_some_and(|prefix| prefix.ends_with('.'))
}

fn path_matches(request_path: &str, cookie_path: &str) -> bool {
    if request_path == cookie_path {
        return true;
    }
    if !request_path.starts_with(cookie_path) {
        return false;
    }
    cookie_path.ends_with('/')
        || request_path
            .as_bytes()
            .get(cookie_path.len())
            .is_some_and(|byte| *byte == b'/')
}

fn default_cookie_path(url: &Url) -> String {
    let path = url.path();
    if !path.starts_with('/') || path == "/" {
        return "/".to_string();
    }
    match path.rfind('/') {
        Some(0) | None => "/".to_string(),
        Some(index) => path[..index].to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_rules_work() {
        assert!(domain_matches("sub.example.com", "example.com", false));
        assert!(!domain_matches("badexample.com", "example.com", false));
        assert!(!domain_matches("sub.example.com", "example.com", true));
    }

    #[test]
    fn path_rules_work() {
        assert!(path_matches("/docs/page", "/docs"));
        assert!(!path_matches("/docs2", "/docs"));
    }
}
