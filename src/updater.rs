use semver::Version;
use std::{
    env,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

const RELEASES_API: &str =
    "https://api.github.com/repos/alexsandrmarinchenko-bot/murg/releases?per_page=10";
const RELEASE_DOWNLOAD_PREFIX: &str =
    "https://github.com/alexsandrmarinchenko-bot/murg/releases/download/";
const MAX_API_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const MAX_BINARY_SIZE_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct AvailableUpdate {
    pub version: String,
    download_url: String,
    sha256: String,
}

pub fn check_for_update() -> Result<Option<AvailableUpdate>, String> {
    if env::consts::OS != "linux" || env::consts::ARCH != "x86_64" {
        return Ok(None);
    }

    let body = curl_text(RELEASES_API, "20")?;
    if body.len() > MAX_API_RESPONSE_BYTES {
        return Err("Ответ GitHub Releases слишком большой".to_owned());
    }

    let releases: serde_json::Value =
        serde_json::from_str(&body).map_err(|error| format!("Некорректный ответ GitHub: {error}"))?;
    let releases = releases
        .as_array()
        .ok_or_else(|| "GitHub вернул неожиданный формат списка релизов".to_owned())?;

    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|error| format!("Некорректная версия текущей сборки: {error}"))?;

    let latest_stable = releases.iter().find(|release| {
        release.get("draft").and_then(serde_json::Value::as_bool) == Some(false)
            && release
                .get("prerelease")
                .and_then(serde_json::Value::as_bool)
                == Some(false)
    });

    let Some(release) = latest_stable else {
        // No published release yet. The update button remains hidden.
        return Ok(None);
    };

    let tag = release
        .get("tag_name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "У релиза отсутствует tag_name".to_owned())?;
    let version_text = tag.strip_prefix('v').unwrap_or(tag);
    let version = Version::parse(version_text)
        .map_err(|error| format!("Версия релиза {tag:?} не соответствует SemVer: {error}"))?;

    if version <= current_version {
        return Ok(None);
    }

    let assets = release
        .get("assets")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "У релиза отсутствует список файлов".to_owned())?;

    let binary_asset = assets
        .iter()
        .find(|asset| {
            asset.get("name").and_then(serde_json::Value::as_str)
                == Some("playerdock-linux-x86_64")
        })
        .ok_or_else(|| "В новом релизе нет сборки для Linux x86_64".to_owned())?;

    let download_url = binary_asset
        .get("browser_download_url")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "У файла обновления отсутствует ссылка".to_owned())?
        .to_owned();

    validate_release_url(&download_url)?;

    let sha256 = if let Some(digest) = binary_asset
        .get("digest")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.strip_prefix("sha256:"))
    {
        digest.to_owned()
    } else {
        let checksum_asset = assets
            .iter()
            .find(|asset| {
                asset.get("name").and_then(serde_json::Value::as_str)
                    == Some("playerdock-linux-x86_64.sha256")
            })
            .ok_or_else(|| "В релизе нет SHA-256 контрольной суммы".to_owned())?;

        let checksum_url = checksum_asset
            .get("browser_download_url")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "У файла контрольной суммы отсутствует ссылка".to_owned())?;
        validate_release_url(checksum_url)?;

        let checksum_text = curl_text(checksum_url, "20")?;
        checksum_text
            .split_whitespace()
            .next()
            .filter(|value| value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit()))
            .ok_or_else(|| "Некорректная SHA-256 контрольная сумма".to_owned())?
            .to_owned()
    };

    if sha256.len() != 64 || !sha256.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("Некорректная SHA-256 контрольная сумма".to_owned());
    }

    Ok(Some(AvailableUpdate {
        version: version.to_string(),
        download_url,
        sha256: sha256.to_ascii_lowercase(),
    }))
}

fn validate_release_url(url: &str) -> Result<(), String> {
    if url.starts_with(RELEASE_DOWNLOAD_PREFIX) {
        Ok(())
    } else {
        Err("GitHub вернул ссылку на файл вне официального репозитория PlayerDock".to_owned())
    }
}

fn curl_text(url: &str, timeout: &str) -> Result<String, String> {
    let output = Command::new("curl")
        .args([
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--max-time",
            timeout,
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "User-Agent: PlayerDock",
            url,
        ])
        .output()
        .map_err(|error| format!("Не удалось запустить curl: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "Не удалось проверить GitHub Releases: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| format!("GitHub вернул некорректный UTF-8: {error}"))
}

fn ensure_writable_directory(directory: &Path) -> Result<(), String> {
    let probe = directory.join(format!(".playerdock-write-test-{}", std::process::id()));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .map_err(|error| {
            format!(
                "Каталог приложения недоступен для записи ({directory:?}): {error}. \
                 Установи PlayerDock в пользовательский каталог, например на /mnt/Storage."
            )
        })?;
    drop(file);
    fs::remove_file(&probe).map_err(|error| error.to_string())
}

fn download_asset(url: &str, destination: &Path) -> Result<(), String> {
    let status = Command::new("curl")
        .args([
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--max-time",
            "180",
            "--header",
            "User-Agent: PlayerDock",
            "--output",
        ])
        .arg(destination)
        .arg(url)
        .status()
        .map_err(|error| format!("Не удалось запустить curl: {error}"))?;

    if !status.success() {
        let _ = fs::remove_file(destination);
        return Err("Не удалось скачать обновление с GitHub".to_owned());
    }

    let size = fs::metadata(destination)
        .map_err(|error| error.to_string())?
        .len();
    if size == 0 || size > MAX_BINARY_SIZE_BYTES {
        let _ = fs::remove_file(destination);
        return Err("Размер файла обновления недопустим".to_owned());
    }

    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .map_err(|error| format!("Не удалось запустить sha256sum: {error}"))?;

    if !output.status.success() {
        return Err("Не удалось проверить SHA-256 скачанного файла".to_owned());
    }

    let hash = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();

    if hash.len() != 64 || !hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("sha256sum вернул некорректную контрольную сумму".to_owned());
    }

    Ok(hash)
}

/// Downloads and verifies the release binary, then starts a helper process.
/// The helper waits for this UI process to exit before atomically replacing it.
pub fn download_and_launch_update() -> Result<String, String> {
    let update = check_for_update()?
        .ok_or_else(|| "Опубликованной новой версии сейчас нет".to_owned())?;

    let executable =
        env::current_exe().map_err(|error| format!("Не удалось определить файл приложения: {error}"))?;
    let executable = fs::canonicalize(&executable)
        .map_err(|error| format!("Не удалось определить путь приложения: {error}"))?;
    let directory = executable
        .parent()
        .ok_or_else(|| "Не удалось определить каталог приложения".to_owned())?;

    ensure_writable_directory(directory)?;

    let staged = directory.join(format!(
        ".playerdock-update-{}-{}",
        update.version,
        std::process::id()
    ));
    if staged.exists() {
        fs::remove_file(&staged).map_err(|error| error.to_string())?;
    }

    download_asset(&update.download_url, &staged)?;

    let actual_hash = sha256_file(&staged)?;
    if actual_hash != update.sha256 {
        let _ = fs::remove_file(&staged);
        return Err("Контрольная сумма обновления не совпала. Файл удалён".to_owned());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&staged)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&staged, permissions)
            .map_err(|error| error.to_string())?;
    }

    let helper = Command::new(&executable)
        .arg("--apply-update")
        .arg(std::process::id().to_string())
        .arg(&staged)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    if let Err(error) = helper {
        let _ = fs::remove_file(&staged);
        return Err(format!("Не удалось запустить помощник обновления: {error}"));
    }

    Ok(update.version)
}

/// Entry point used only by the short-lived update helper process.
pub fn apply_staged_update(parent_pid: u32, staged_path: &Path) -> Result<(), String> {
    let target = env::current_exe()
        .map_err(|error| error.to_string())?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let directory = target
        .parent()
        .ok_or_else(|| "Не удалось определить каталог приложения".to_owned())?;

    let staged = staged_path
        .canonicalize()
        .map_err(|error| format!("Скачанный файл обновления не найден: {error}"))?;

    if staged.parent() != Some(directory)
        || !staged
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(".playerdock-update-"))
    {
        return Err("Путь временного файла обновления не прошёл проверку".to_owned());
    }

    wait_for_parent_exit(parent_pid, &target)?;

    let backup = directory.join(".playerdock.previous");
    if backup.exists() {
        fs::remove_file(&backup)
            .map_err(|error| format!("Не удалось очистить старую резервную копию: {error}"))?;
    }

    fs::rename(&target, &backup)
        .map_err(|error| format!("Не удалось сохранить предыдущую версию: {error}"))?;

    if let Err(error) = fs::rename(&staged, &target) {
        let _ = fs::rename(&backup, &target);
        return Err(format!("Не удалось установить новую версию: {error}"));
    }

    match Command::new(&target).spawn() {
        Ok(_child) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(&target);
            let _ = fs::rename(&backup, &target);
            Err(format!(
                "Новая версия не запустилась; предыдущая версия восстановлена: {error}"
            ))
        }
    }
}

fn wait_for_parent_exit(parent_pid: u32, target: &Path) -> Result<(), String> {
    let process_executable = PathBuf::from(format!("/proc/{parent_pid}/exe"));

    for _ in 0..600 {
        match fs::read_link(&process_executable) {
            Ok(path) if path == target => thread::sleep(Duration::from_millis(200)),
            _ => return Ok(()),
        }
    }

    Err("Приложение не завершилось вовремя; обновление отменено".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_only_playerdock_release_assets() {
        assert!(validate_release_url(
            "https://github.com/alexsandrmarinchenko-bot/murg/releases/download/v0.2.0/playerdock-linux-x86_64"
        )
        .is_ok());

        assert!(validate_release_url("https://example.com/playerdock").is_err());
        assert!(validate_release_url(
            "https://github.com/another-owner/murg/releases/download/v0.2.0/playerdock-linux-x86_64"
        )
        .is_err());
    }
}
