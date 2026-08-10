//! Binary-core resolution + version sanity for the e2e pipeline.
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreKind { Xray, SingBox }

impl CoreKind {
    /// Binary file name inside `XRAY_TUI_CORE_BIN_DIR}.
    pub fn bin_name(self) -> &'static str {
        match self { Self::Xray => "xray", Self::SingBox => "sing-box" }
    }
}

/// A resolved, version-checked core binary.
#[derive(Debug, Clone)]
pub struct CoreUnderTest {
    pub kind: CoreKind,
    pub bin: PathBuf,
    pub version: String,
}

impl CoreUnderTest {
    /// Resolve from `XRAY_TUI_CORE_BIN_DIR/<bin_name>`, probe its version,
    /// and sanity-check it against `expected_version` (substring match).
    pub fn resolve(kind: CoreKind, expected_version: &str) -> Result<Self, String> {
        let dir = std::env::var("XRAY_TUI_CORE_BIN_DIR")
            .map_err(|_| "XRAY_TUI_CORE_BIN_DIR is not set".to_string())?;
        let bin = PathBuf::from(dir).join(kind.bin_name());
        if !bin.is_file() {
            return Err(format!("no '{}' binary at {}", kind.bin_name(), bin.display()));
        }
        let actual = probe_version(&bin, kind)?;
        if !check_version(&actual, expected_version) {
            return Err(format!(
                "core version mismatch: want {expected_version:?}, got {actual:?}"
            ));
        }
        Ok(Self { kind, bin, version: actual })
    }

    /// argv for spawning with the given config file.
    pub fn spawn_args(&self, config_path: &std::path::Path) -> Vec<String> {
        let p = config_path.to_string_lossy().into_owned();
        match self.kind {
            CoreKind::Xray => vec!["-c".into(), p],
            CoreKind::SingBox => vec!["run".into(), "-c".into(), p],
        }
    }
}

/// Probe the version string: `-version` first, then `version` (sing-box).
fn probe_version(bin: &std::path::Path, kind: CoreKind) -> Result<String, String> {
    for flag in [Some("-version"), None] {
        let mut cmd = Command::new(bin);
        match flag {
            Some(f) => { cmd.arg(f); }
            None => { cmd.arg("version"); }
        }
        if let Ok(out) = cmd.output() {
            let text = String::from_utf8_lossy(&out.stdout).into_owned();
            let text = text.trim();
            if !text.is_empty() {
                return Ok(text.to_string());
            }
        }
        if kind == CoreKind::SingBox {
            break; // `run` would block; only try `-version` then exit
        }
    }
    Err(format!("failed to probe version of {}", bin.display()))
}

/// Loose sanity: the reported version contains the expected one.
pub fn check_version(actual: &str, want: &str) -> bool {
    actual.contains(want)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_substring_matches() {
        assert!(check_version("Xray 26.3.27 (Xray-core 26.3.27)", "26.3.27"));
        assert!(check_version("sing-box 1.13.16", "1.13.16"));
        assert!(!check_version("sing-box 1.12.0", "1.13.16"));
    }
}
