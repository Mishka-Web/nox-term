use std::{
    cmp::Ordering,
    env, fs,
    path::{Path, PathBuf},
};

#[cfg(windows)]
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use reqwest::{blocking::Client, redirect::Policy};
use sha2::{Digest, Sha256};

const BUILT_REPOSITORY: Option<&str> = option_env!("NOX_GITHUB_REPOSITORY");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run(check_only: bool) -> Result<()> {
    let repository = repository()?;
    let client = update_client()?;

    println!("NOX {VERSION}");
    println!("Проверяю обновления: {repository}");

    let version_url = release_asset_url(&repository, "VERSION");
    let latest = download_text(&client, &version_url)?;
    let latest = latest.trim().trim_start_matches('v');

    match compare_versions(latest, VERSION)? {
        Ordering::Less | Ordering::Equal => {
            println!("У вас уже актуальная версия: {VERSION}");
            return Ok(());
        }
        Ordering::Greater => {}
    }

    println!("Доступна новая версия: {VERSION} -> {latest}");
    if check_only {
        return Ok(());
    }

    let asset = platform_asset_name()?;
    let checksum_url = release_asset_url(&repository, "SHA256SUMS");
    let checksums = download_text(&client, &checksum_url)?;
    let expected = checksum_for(&checksums, asset)
        .with_context(|| format!("в SHA256SUMS нет записи для {asset}"))?;

    println!("Скачиваю {asset}...");
    let binary_url = release_asset_url(&repository, asset);
    let binary = download_bytes(&client, &binary_url)?;

    let actual = sha256_hex(&binary);
    if !actual.eq_ignore_ascii_case(expected) {
        bail!(
            "проверка SHA-256 не пройдена: ожидался {expected}, получен {actual}. Обновление отменено"
        );
    }
    println!("SHA-256 проверен.");

    stage_and_replace(&binary, latest)?;
    Ok(())
}

fn repository() -> Result<String> {
    let runtime = env::var("NOX_GITHUB_REPOSITORY").ok();
    let value = runtime
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| BUILT_REPOSITORY.filter(|value| !value.trim().is_empty()))
        .context(
            "этот NOX собран без адреса GitHub-репозитория. Для dev-сборки задайте NOX_GITHUB_REPOSITORY=owner/repo; официальные GitHub Releases вшивают адрес автоматически",
        )?;

    let value = value.trim().trim_end_matches(".git");
    if value.starts_with("http://") || value.starts_with("https://") || value.matches('/').count() != 1 {
        bail!("NOX_GITHUB_REPOSITORY должен иметь вид owner/repo");
    }

    Ok(value.to_string())
}

fn release_asset_url(repository: &str, asset: &str) -> String {
    format!("https://github.com/{repository}/releases/latest/download/{asset}")
}

fn update_client() -> Result<Client> {
    Client::builder()
        .redirect(Policy::limited(10))
        .user_agent(format!("Nox/{VERSION} self-updater"))
        .build()
        .context("не удалось создать HTTP-клиент обновления")
}

fn download_text(client: &Client, url: &str) -> Result<String> {
    client
        .get(url)
        .send()
        .with_context(|| format!("не удалось скачать {url}"))?
        .error_for_status()
        .with_context(|| format!("сервер вернул ошибку для {url}"))?
        .text()
        .context("не удалось прочитать ответ сервера")
}

fn download_bytes(client: &Client, url: &str) -> Result<Vec<u8>> {
    let bytes = client
        .get(url)
        .send()
        .with_context(|| format!("не удалось скачать {url}"))?
        .error_for_status()
        .with_context(|| format!("сервер вернул ошибку для {url}"))?
        .bytes()
        .context("не удалось прочитать загруженный бинарник")?;

    Ok(bytes.to_vec())
}

fn parse_version(value: &str) -> Result<Vec<u64>> {
    let stable = value
        .trim()
        .trim_start_matches('v')
        .split(['-', '+'])
        .next()
        .unwrap_or_default();

    if stable.is_empty() {
        bail!("пустая версия");
    }

    stable
        .split('.')
        .map(|part| {
            part.parse::<u64>()
                .with_context(|| format!("некорректная версия: {value}"))
        })
        .collect()
}

fn compare_versions(left: &str, right: &str) -> Result<Ordering> {
    let left = parse_version(left)?;
    let right = parse_version(right)?;
    let len = left.len().max(right.len());

    for index in 0..len {
        let a = *left.get(index).unwrap_or(&0);
        let b = *right.get(index).unwrap_or(&0);
        match a.cmp(&b) {
            Ordering::Equal => continue,
            ordering => return Ok(ordering),
        }
    }

    Ok(Ordering::Equal)
}

fn checksum_for<'a>(contents: &'a str, asset: &str) -> Option<&'a str> {
    contents.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?.trim_start_matches('*');
        (name == asset).then_some(hash)
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

fn platform_asset_name() -> Result<&'static str> {
    match (env::consts::OS, env::consts::ARCH) {
        ("windows", "x86_64") => Ok("nox-windows-x64.exe"),
        ("windows", "aarch64") => Ok("nox-windows-arm64.exe"),
        ("linux", "x86_64") => Ok("nox-linux-x64"),
        ("linux", "aarch64") => Ok("nox-linux-arm64"),
        ("macos", "x86_64") => Ok("nox-macos-x64"),
        ("macos", "aarch64") => Ok("nox-macos-arm64"),
        (os, arch) => bail!("автообновление пока не поддерживает {os}/{arch}"),
    }
}

fn stage_and_replace(binary: &[u8], latest: &str) -> Result<()> {
    let current = env::current_exe().context("не удалось определить путь текущего NOX")?;
    let parent = current
        .parent()
        .context("не удалось определить каталог текущего NOX")?;
    let staged = staged_path(parent);

    fs::write(&staged, binary).with_context(|| {
        format!(
            "не удалось записать обновление рядом с {}. Установите NOX в каталог, доступный текущему пользователю",
            current.display()
        )
    })?;

    set_executable_permissions(&staged, &current)?;

    #[cfg(windows)]
    {
        schedule_windows_replace(&staged, &current)?;
        println!("NOX {latest} скачан и проверен.");
        println!("Обновление будет применено сразу после завершения этой команды.");
        Ok(())
    }

    #[cfg(not(windows))]
    {
        fs::rename(&staged, &current).with_context(|| {
            format!(
                "не удалось заменить {}. Проверьте права на каталог",
                current.display()
            )
        })?;
        println!("NOX обновлён до {latest}.");
        println!("Проверьте: nox --version");
        Ok(())
    }
}

fn staged_path(parent: &Path) -> PathBuf {
    #[cfg(windows)]
    let name = format!(".nox-update-{}.exe", std::process::id());
    #[cfg(not(windows))]
    let name = format!(".nox-update-{}", std::process::id());

    parent.join(name)
}

#[cfg(unix)]
fn set_executable_permissions(staged: &Path, current: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mode = fs::metadata(current)
        .map(|metadata| metadata.permissions().mode())
        .unwrap_or(0o755);
    fs::set_permissions(staged, fs::Permissions::from_mode(mode | 0o111))
        .context("не удалось выставить права на новый бинарник")
}

#[cfg(windows)]
fn set_executable_permissions(_staged: &Path, _current: &Path) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
fn schedule_windows_replace(staged: &Path, current: &Path) -> Result<()> {
    let staged = ps_quote(staged);
    let current = ps_quote(current);
    let pid = std::process::id();
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue'; Wait-Process -Id {pid}; \
         for($i=0;$i -lt 30;$i++){{ try {{ Move-Item -LiteralPath '{staged}' -Destination '{current}' -Force -ErrorAction Stop; exit 0 }} catch {{ Start-Sleep -Milliseconds 200 }} }}; exit 1"
    );

    Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("не удалось запустить процесс, который применит обновление NOX")?;

    Ok(())
}

#[cfg(windows)]
fn ps_quote(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_numerically() {
        assert_eq!(compare_versions("0.10.0", "0.9.9").unwrap(), Ordering::Greater);
        assert_eq!(compare_versions("v1.2.0", "1.2").unwrap(), Ordering::Equal);
        assert_eq!(compare_versions("1.2.3", "1.3.0").unwrap(), Ordering::Less);
    }

    #[test]
    fn reads_checksum_line() {
        let text = "abc123  nox-linux-x64\ndef456 *nox-macos-arm64\n";
        assert_eq!(checksum_for(text, "nox-linux-x64"), Some("abc123"));
        assert_eq!(checksum_for(text, "nox-macos-arm64"), Some("def456"));
    }

    #[test]
    fn hashes_bytes() {
        assert_eq!(
            sha256_hex(b"nox"),
            "b076d0643d8294e911d8e3e67c6bdf2a7668dcafa8558a4d36b7442783d4b055"
        );
    }
}
