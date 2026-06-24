use crate::core_type::CoreType;
use semver::Version;
use std::path::{Path, PathBuf};

/// Parse a version tag (strip leading 'v') as semver::Version.
pub fn parse_version(tag: &str) -> Option<Version> {
    let stripped = tag.strip_prefix('v').unwrap_or(tag);
    Version::parse(stripped).ok()
}

/// Compare two versions. Returns true if `latest > current` per semver precedence.
pub fn is_newer(current: &Version, latest: &Version) -> bool {
    *latest > *current
}

/// Run `{core_type} version` and parse the first version-like token from stdout.
pub async fn get_current_version(core_type: CoreType, bin_dir: &Path) -> Option<String> {
    let exe = match core_type {
        CoreType::Xray => "xray",
        CoreType::SingBox => "sing-box",
        CoreType::Auto => return None,
    };

    // Try managed bin_dir first (inside per-core subdirectory), then PATH
    let bin_path = bin_dir.join(core_type.to_string()).join(exe);
    let cmd = if bin_path.is_file() {
        bin_path.to_string_lossy().to_string()
    } else {
        exe.to_string()
    };

    let output = tokio::process::Command::new(&cmd)
        .arg("version")
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Extract the first version-like token (e.g., "1.8.4" from "xray 1.8.4 ...")
    let version_str = stdout.split_whitespace().find(|token| {
        let stripped = token.strip_prefix('v').unwrap_or(token);
        stripped.chars().next().is_some_and(|c| c.is_ascii_digit())
            && stripped.chars().any(|c| c == '.')
    })?;

    let stripped = version_str.strip_prefix('v').unwrap_or(version_str);
    if Version::parse(stripped).is_ok() {
        Some(version_str.to_string())
    } else {
        None
    }
}

/// GitHub API endpoints and asset naming for each core type.
const XRAY_OWNER: &str = "XTLS";
const XRAY_REPO: &str = "Xray-core";
const SINGBOX_OWNER: &str = "SagerNet";
const SINGBOX_REPO: &str = "sing-box";

/// GET latest release from GitHub API and return the tag_name.
pub async fn get_latest_version(core_type: CoreType) -> Option<String> {
    let (owner, repo) = match core_type {
        CoreType::Xray => (XRAY_OWNER, XRAY_REPO),
        CoreType::SingBox => (SINGBOX_OWNER, SINGBOX_REPO),
        CoreType::Auto => return None,
    };

    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "xray-tui")
        .header("Accept", "application/json")
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let body: serde_json::Value = resp.json().await.ok()?;
    let tag_name = body.get("tag_name")?.as_str()?;
    Some(tag_name.to_string())
}

/// Download the release asset for the current platform.
pub async fn download_release(
    client: &reqwest::Client,
    core_type: CoreType,
    version: &str,
    dest_dir: &Path,
) -> Result<PathBuf, String> {
    let url = release_asset_url(core_type, version).ok_or_else(|| {
        format!(
            "unsupported platform: {}-{}",
            std::env::consts::ARCH,
            std::env::consts::OS
        )
    })?;

    let filename = url.rsplit('/').next().unwrap_or("archive.tar.gz");
    let dest = dest_dir.join(filename);

    std::fs::create_dir_all(dest_dir).map_err(|e| format!("failed to create temp dir: {e}"))?;

    // Stream download
    let resp = client
        .get(&url)
        .header("User-Agent", "xray-tui")
        .header("Accept", "application/octet-stream")
        .send()
        .await
        .map_err(|e| format!("download failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("download returned HTTP {}", resp.status()));
    }

    let total_size = resp.content_length();
    let mut file = tokio::fs::File::create(&dest)
        .await
        .map_err(|e| format!("failed to create file: {e}"))?;

    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("download stream error: {e}"))?;
        downloaded += chunk.len() as u64;
        if let Some(total) = total_size {
            // Progress could be reported via callback; for now we keep simple
            let _ = (downloaded, total);
        }
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(|e| format!("file write error: {e}"))?;
    }

    Ok(dest)
}

/// Extract archive, verify binary, install all files to bin_dir.
pub async fn install_binary(
    archive: &Path,
    core_type: CoreType,
    bin_dir: &Path,
) -> Result<(), String> {
    let suffix = match core_type {
        CoreType::Xray => "xray",
        CoreType::SingBox => "sing-box",
        CoreType::Auto => return Err("cannot install for Auto core type".into()),
    };
    let temp_dir = std::env::temp_dir().join(format!("xray-tui-install-{suffix}"));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("failed to create temp dir: {e}"))?;

    // 1. Extract archive to temp dir
    crate::bin_manager::extract_archive(archive, core_type, &temp_dir)
        .map_err(|e| format!("extraction failed: {e}"))?;

    // 2. Find extracted binary and verify it runs
    let exe_name = match core_type {
        CoreType::Xray => "xray",
        CoreType::SingBox => "sing-box",
        CoreType::Auto => return Err("cannot install for Auto core type".into()),
    };

    // The binary may be in a subdirectory (e.g., sing-box-1.9.0-linux-amd64/)
    let binary = find_binary_in_dir(&temp_dir, exe_name)?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(&binary).map_err(|e| format!("cannot stat binary: {e}"))?;
        let mut perms = meta.permissions();
        perms.set_mode(perms.mode() | 0o111);
        std::fs::set_permissions(&binary, perms)
            .map_err(|e| format!("cannot set executable bit: {e}"))?;
    }

    // Verify binary runs
    let output = tokio::process::Command::new(&binary)
        .arg("version")
        .output()
        .await
        .map_err(|e| format!("failed to execute extracted binary: {e}"))?;

    if !output.status.success() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err("extracted binary returned non-zero exit code".into());
    }

    let version_output = String::from_utf8_lossy(&output.stdout);
    if version_output.is_empty() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err("extracted binary produced no version output".into());
    }

    // 3. Install to bin_dir
    let core_bin_dir = bin_dir.join(match core_type {
        CoreType::Xray => "xray",
        CoreType::SingBox => "sing-box",
        CoreType::Auto => return Err("cannot install for Auto core type".into()),
    });

    // Remove old .bak if present
    let bak_dir = bin_dir.join(format!(
        "{}.bak",
        core_bin_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    ));
    let _ = std::fs::remove_dir_all(&bak_dir);

    // Rename existing directory to .bak
    if core_bin_dir.exists() {
        std::fs::rename(&core_bin_dir, &bak_dir)
            .map_err(|e| format!("failed to back up existing binary: {e}"))?;
    }

    // Copy all extracted files
    std::fs::create_dir_all(&core_bin_dir).map_err(|e| format!("failed to create bin dir: {e}"))?;

    if let Err(e) = copy_recursively(&temp_dir, &core_bin_dir) {
        // Restore from backup
        let _ = std::fs::remove_dir_all(&core_bin_dir);
        if bak_dir.exists() {
            let _ = std::fs::rename(&bak_dir, &core_bin_dir);
        }
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(format!("failed to copy files: {e}"));
    }

    // Remove .bak on success
    let _ = std::fs::remove_dir_all(&bak_dir);
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}

/// Build the GitHub release asset URL for the current OS/arch.
pub fn release_asset_url(core_type: CoreType, version: &str) -> Option<String> {
    let arch = match std::env::consts::ARCH {
        "x86_64" => match core_type {
            CoreType::Xray => "64",
            CoreType::SingBox => "amd64",
            CoreType::Auto => return None,
        },
        "aarch64" => match core_type {
            CoreType::Xray => "arm64",
            CoreType::SingBox => "arm64",
            CoreType::Auto => return None,
        },
        _ => return None,
    };

    let os = std::env::consts::OS;
    if os != "linux" {
        return None;
    }

    let url = match core_type {
        CoreType::Xray => format!(
            "https://github.com/XTLS/Xray-core/releases/download/v{version}/Xray-linux-{arch}.zip",
            version = version.strip_prefix('v').unwrap_or(version),
        ),
        CoreType::SingBox => format!(
            "https://github.com/SagerNet/sing-box/releases/download/v{version}/sing-box-{version}-linux-{arch}.tar.gz",
            version = version.strip_prefix('v').unwrap_or(version),
        ),
        CoreType::Auto => return None,
    };

    Some(url)
}

// ── Internal helpers ──────────────────────────────────────────────────────

/// Recursively copy a directory.
fn copy_recursively(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if file_type.is_dir() {
            copy_recursively(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Find a binary in a directory tree (searching recursively).
fn find_binary_in_dir(dir: &Path, exe_name: &str) -> Result<PathBuf, String> {
    let mut found = None;
    let mut entries: Vec<_> = vec![dir.to_path_buf()];

    while let Some(current) = entries.pop() {
        if let Ok(read_dir) = std::fs::read_dir(&current) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    entries.push(path);
                } else if path.file_name().and_then(|n| n.to_str()) == Some(exe_name) {
                    found = Some(path);
                    break;
                }
            }
        }
        if found.is_some() {
            break;
        }
    }

    found.ok_or_else(|| format!("{exe_name} not found in extracted archive"))
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("v1.8.4"), Some(Version::new(1, 8, 4)));
        assert_eq!(parse_version("1.8.4"), Some(Version::new(1, 8, 4)));
        assert!(parse_version("v1.9.0-rc1").is_some());
        assert_eq!(parse_version("invalid"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer(&Version::new(1, 8, 4), &Version::new(1, 8, 10)));
        assert!(!is_newer(&Version::new(1, 8, 10), &Version::new(1, 8, 4)));
        assert!(!is_newer(&Version::new(1, 8, 4), &Version::new(1, 8, 4)));
        // Pre-release: 1.9.0-rc1 < 1.9.0, so 1.9.0 IS newer than rc1
        assert!(is_newer(
            &Version::parse("1.9.0-rc1").unwrap(),
            &Version::new(1, 9, 0)
        ));
        assert!(!is_newer(
            &Version::new(1, 9, 0),
            &Version::parse("1.9.0-rc1").unwrap()
        ));
    }

    #[test]
    fn test_release_asset_url() {
        let xray_url = release_asset_url(CoreType::Xray, "v1.8.10").unwrap();
        assert!(xray_url.contains("Xray-linux-64.zip"));

        let singbox_url = release_asset_url(CoreType::SingBox, "v1.10.3").unwrap();
        assert!(singbox_url.contains("sing-box-1.10.3-linux-amd64.tar.gz"));
    }

    #[test]
    #[cfg(unix)]
    fn get_current_version_finds_managed_binary() {
        let tmp = std::env::temp_dir().join("xray-tui-test-updater-managed");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp.join("xray")).unwrap();

        // Create a fake xray binary that outputs version info
        let binary = tmp.join("xray").join("xray");
        std::fs::write(&binary, "#!/bin/sh\necho \"xray 1.8.4\"\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let version = rt.block_on(get_current_version(CoreType::Xray, &tmp));
        assert_eq!(version, Some("1.8.4".to_string()));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    #[cfg(unix)]
    fn get_current_version_with_singbox_managed_binary() {
        let tmp = std::env::temp_dir().join("xray-tui-test-updater-singbox");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp.join("sing-box")).unwrap();

        let binary = tmp.join("sing-box").join("sing-box");
        std::fs::write(&binary, "#!/bin/sh\necho \"sing-box 1.10.3\"\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let version = rt.block_on(get_current_version(CoreType::SingBox, &tmp));
        assert_eq!(version, Some("1.10.3".to_string()));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    #[cfg(unix)]
    fn find_binary_and_get_current_version_combined() {
        let tmp = std::env::temp_dir().join("xray-tui-test-updater-combined");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp.join("xray")).unwrap();

        let binary = tmp.join("xray").join("xray");
        std::fs::write(&binary, "#!/bin/sh\necho \"xray 2.0.0\"\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();

        // find_binary should find the managed binary
        let found = crate::bin_manager::find_binary(CoreType::Xray, &tmp);
        assert!(found.is_some(), "find_binary should find managed xray");
        assert_eq!(
            found.unwrap(),
            binary,
            "find_binary should return the managed path"
        );

        // get_current_version should parse the version from the same binary
        let rt = tokio::runtime::Runtime::new().unwrap();
        let version = rt.block_on(get_current_version(CoreType::Xray, &tmp));
        assert_eq!(version, Some("2.0.0".to_string()));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn get_current_version_returns_none_when_binary_missing() {
        let tmp = std::env::temp_dir().join("xray-tui-test-updater-missing");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let version = rt.block_on(get_current_version(CoreType::Xray, &tmp));
        // Should be None since no binary exists in managed dir and likely not in PATH
        assert!(version.is_none(), "should return None when no binary found");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
