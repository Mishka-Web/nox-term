use std::{env, fs, path::{Path, PathBuf}};

#[cfg(windows)]
use std::process::Command;

use anyhow::{bail, Context, Result};

pub fn install_self() -> Result<()> {
    let current = env::current_exe().context("не удалось определить путь текущего NOX")?;
    let destination = install_destination()?;

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("не удалось создать {}", parent.display()))?;
    }

    if !same_path(&current, &destination) {
        fs::copy(&current, &destination).with_context(|| {
            format!(
                "не удалось установить NOX: {} -> {}",
                current.display(),
                destination.display()
            )
        })?;
    }

    #[cfg(unix)]
    make_executable(&destination)?;

    ensure_command_available(&destination)?;

    println!("NOX установлен: {}", destination.display());
    println!("Проверьте: nox --version");
    Ok(())
}

pub fn uninstall_self() -> Result<()> {
    let destination = install_destination()?;

    #[cfg(windows)]
    remove_windows_path(destination.parent().unwrap_or(Path::new("")))?;

    #[cfg(unix)]
    remove_unix_profile_marker(destination.parent().unwrap_or(Path::new("")))?;

    let current = env::current_exe().ok();
    if destination.exists() {
        if current.as_ref().is_some_and(|p| same_path(p, &destination)) {
            #[cfg(windows)]
            schedule_windows_delete(&destination)?;
            #[cfg(not(windows))]
            fs::remove_file(&destination).context("не удалось удалить установленный NOX")?;
        } else {
            fs::remove_file(&destination).context("не удалось удалить установленный NOX")?;
        }
    }

    println!("NOX удалён из пользовательской установки.");
    println!("В уже открытом терминале старая запись PATH может оставаться до перезапуска.");
    Ok(())
}

fn install_destination() -> Result<PathBuf> {
    #[cfg(windows)]
    {
        let local = env::var_os("LOCALAPPDATA").context("LOCALAPPDATA не определён")?;
        return Ok(PathBuf::from(local).join("Programs").join("NOX").join("nox.exe"));
    }

    #[cfg(not(windows))]
    {
        let home = env::var_os("HOME").context("HOME не определён")?;
        Ok(PathBuf::from(home).join(".local").join("bin").join("nox"))
    }
}

fn same_path(a: &Path, b: &Path) -> bool {
    let ca = fs::canonicalize(a).unwrap_or_else(|_| a.to_path_buf());
    let cb = fs::canonicalize(b).unwrap_or_else(|_| b.to_path_buf());
    ca == cb
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(permissions.mode() | 0o755);
    fs::set_permissions(path, permissions).context("не удалось выставить executable bit")
}

#[cfg(windows)]
fn ensure_command_available(destination: &Path) -> Result<()> {
    let install_dir = destination.parent().context("нет каталога установки")?;
    let escaped = install_dir.to_string_lossy().replace('\'', "''");
    let script = format!(
        "$d='{escaped}'; $p=[Environment]::GetEnvironmentVariable('Path','User'); \
         $items=@(); if($p){{$items=$p -split ';'}}; \
         if($items -notcontains $d){{ \
           $n=if([string]::IsNullOrWhiteSpace($p)){{$d}}else{{$p.TrimEnd(';')+';'+$d}}; \
           [Environment]::SetEnvironmentVariable('Path',$n,'User') \
         }}"
    );
    let status = Command::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .status()
        .context("не удалось добавить NOX в пользовательский PATH")?;
    if !status.success() {
        bail!("PowerShell не смог обновить пользовательский PATH");
    }
    println!("Каталог NOX добавлен в пользовательский PATH.");
    println!("Если команда `nox` ещё не видна в этом окне, откройте новый терминал.");
    Ok(())
}

#[cfg(not(windows))]
fn ensure_command_available(destination: &Path) -> Result<()> {
    let dir = destination.parent().context("нет каталога установки")?;
    if env::split_paths(&env::var_os("PATH").unwrap_or_default()).any(|p| p == dir) {
        return Ok(());
    }

    let home = PathBuf::from(env::var_os("HOME").context("HOME не определён")?);
    let marker_start = "# >>> NOX >>>";
    let marker_end = "# <<< NOX <<<";
    let block = format!(
        "\n{marker_start}\nexport PATH=\"{}:$PATH\"\n{marker_end}\n",
        dir.display()
    );

    let shell = env::var("SHELL").unwrap_or_default();
    let profile = if shell.ends_with("zsh") {
        home.join(".zshrc")
    } else if shell.ends_with("bash") {
        home.join(".bashrc")
    } else {
        home.join(".profile")
    };

    let existing = fs::read_to_string(&profile).unwrap_or_default();
    if !existing.contains(marker_start) {
        use std::io::Write;
        let mut file = fs::OpenOptions::new().create(true).append(true).open(&profile)
            .with_context(|| format!("не удалось изменить {}", profile.display()))?;
        file.write_all(block.as_bytes())?;
    }

    println!("{} добавлен в PATH через {}.", dir.display(), profile.display());
    println!("Откройте новый терминал или выполните: export PATH=\"{}:$PATH\"", dir.display());
    Ok(())
}

#[cfg(windows)]
fn remove_windows_path(dir: &Path) -> Result<()> {
    let escaped = dir.to_string_lossy().replace('\'', "''");
    let script = format!(
        "$d='{escaped}'; $p=[Environment]::GetEnvironmentVariable('Path','User'); \
         if($p){{ $n=(($p -split ';') | Where-Object {{ $_ -and $_ -ne $d }}) -join ';'; \
         [Environment]::SetEnvironmentVariable('Path',$n,'User') }}"
    );
    let _ = Command::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .status();
    Ok(())
}

#[cfg(not(windows))]
fn remove_unix_profile_marker(_dir: &Path) -> Result<()> {
    let home = PathBuf::from(env::var_os("HOME").context("HOME не определён")?);
    for profile in [home.join(".profile"), home.join(".bashrc"), home.join(".zshrc")] {
        let Ok(text) = fs::read_to_string(&profile) else { continue };
        let mut output = String::new();
        let mut skip = false;
        for line in text.lines() {
            if line.trim() == "# >>> NOX >>>" { skip = true; continue; }
            if line.trim() == "# <<< NOX <<<" { skip = false; continue; }
            if !skip { output.push_str(line); output.push('\n'); }
        }
        if output != text { let _ = fs::write(&profile, output); }
    }
    Ok(())
}

#[cfg(windows)]
fn schedule_windows_delete(path: &Path) -> Result<()> {
    let escaped = path.to_string_lossy().replace('\'', "''");
    let pid = std::process::id();
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue'; Wait-Process -Id {pid}; \
         for($i=0;$i -lt 30;$i++){{ try {{ Remove-Item -LiteralPath '{escaped}' -Force -ErrorAction Stop; exit 0 }} catch {{ Start-Sleep -Milliseconds 200 }} }}"
    );
    Command::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .spawn()
        .context("не удалось запланировать удаление NOX")?;
    Ok(())
}
