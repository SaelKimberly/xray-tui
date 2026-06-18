use crate::core_type::CoreType;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CoreBinInfo {
    pub exe_names: &'static [&'static str],
    pub args_template: &'static str,
}

pub fn get_core_info(core_type: CoreType) -> Option<CoreBinInfo> {
    match core_type {
        CoreType::Xray => Some(CoreBinInfo {
            exe_names: &["xray"],
            args_template: "run -c {0}",
        }),
        CoreType::SingBox => Some(CoreBinInfo {
            exe_names: &["sing-box-client", "sing-box"],
            args_template: "run -c {0}",
        }),
        CoreType::Auto => None,
    }
}

/// Find a core binary by checking the managed directory first, then PATH.
pub fn find_binary(core_type: CoreType, bin_dir: &Path) -> Option<PathBuf> {
    let info = get_core_info(core_type)?;

    // 1. Check managed binary directory
    for exe in info.exe_names {
        let managed = bin_dir.join(exe);
        if managed.is_file() {
            // Check if executable (on Unix, file must be executable)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = managed.metadata()
                    && meta.permissions().mode() & 0o111 != 0
                {
                    return Some(managed);
                }
            }
            #[cfg(not(unix))]
            {
                return Some(managed);
            }
        }
    }

    // 2. Check PATH via `which` command
    for exe in info.exe_names {
        if let Ok(path) = std::process::Command::new("which").arg(exe).output()
            && path.status.success()
        {
            let path_str = String::from_utf8_lossy(&path.stdout);
            let trimmed = path_str.trim();
            if !trimmed.is_empty() {
                let p = PathBuf::from(trimmed);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }

    None
}

/// Extract a core binary archive (zip or tar.gz) into the target directory.
pub fn extract_archive(
    archive: &Path,
    _core_type: CoreType,
    target_dir: &Path,
) -> Result<(), BinError> {
    let ext = archive.extension().and_then(|e| e.to_str()).unwrap_or("");

    let stem = archive.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    // Create target dir
    std::fs::create_dir_all(target_dir)?;

    match ext {
        "zip" => extract_zip(archive, target_dir)?,
        "gz" | "tgz" if stem.ends_with(".tar") || ext == "tgz" => {
            extract_tar_gz(archive, target_dir)?
        }
        _ => {
            return Err(BinError::ExtractionFailed(format!(
                "Unsupported archive format: .{ext}"
            )));
        }
    }

    Ok(())
}

fn extract_zip(archive: &Path, target_dir: &Path) -> Result<(), BinError> {
    let file = std::fs::File::open(archive)?;
    let mut archive_zip = zip::ZipArchive::new(file)?;

    for i in 0..archive_zip.len() {
        let mut entry = archive_zip.by_index(i)?;
        if let Some(name) = entry.enclosed_name() {
            let target_path = target_dir.join(name);
            if entry.is_dir() {
                std::fs::create_dir_all(&target_path)?;
            } else {
                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&target_path)?;
                std::io::copy(&mut entry, &mut outfile)?;
            }
        }
    }

    Ok(())
}

fn extract_tar_gz(archive: &Path, target_dir: &Path) -> Result<(), BinError> {
    let file = std::fs::File::open(archive)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive_tar = tar::Archive::new(decoder);

    for entry in archive_tar.entries()? {
        let mut entry = entry?;
        if let Some(path) = entry.path()?.parent() {
            std::fs::create_dir_all(target_dir.join(path))?;
        }
        entry.unpack_in(target_dir)?;
    }

    Ok(())
}

/// Find archives in the current directory and extract them to the bin dir.
pub fn find_and_extract_archives(bin_dir: &Path) -> Result<(), BinError> {
    let cwd = std::env::current_dir()?;

    // Xray zip
    for entry in std::fs::read_dir(&cwd)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if name.starts_with("xray") && name.ends_with(".zip") {
            let xray_dir = bin_dir.join("xray");
            extract_archive(&path, CoreType::Xray, &xray_dir)?;
        }

        if name.starts_with("sing-box") && name.ends_with(".tar.gz") {
            let singbox_dir = bin_dir.join("sing-box");
            extract_archive(&path, CoreType::SingBox, &singbox_dir)?;
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum BinError {
    #[error("Binary not found: {0}")]
    NotFound(String),
    #[error("Archive extraction failed: {0}")]
    ExtractionFailed(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Zip error: {0}")]
    ZipError(#[from] zip::result::ZipError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_core_info_xray() {
        let info = get_core_info(CoreType::Xray).unwrap();
        assert!(info.exe_names.contains(&"xray"));
        assert_eq!(info.args_template, "run -c {0}");
    }

    #[test]
    fn get_core_info_singbox() {
        let info = get_core_info(CoreType::SingBox).unwrap();
        assert!(info.exe_names.contains(&"sing-box"));
        assert_eq!(info.args_template, "run -c {0}");
    }

    #[test]
    fn get_core_info_auto_is_none() {
        assert!(get_core_info(CoreType::Auto).is_none());
    }
}
