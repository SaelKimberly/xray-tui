//! The single fingerprint identity used across the crate.

/// Browsers with at least one hand-transcribed hello.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Browser {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Brave,
    Opera,
    SamsungInternet,
}

impl Browser {
    /// Stable lowercase identifier (configs/logs).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Firefox => "firefox",
            Self::Safari => "safari",
            Self::Edge => "edge",
            Self::Brave => "brave",
            Self::Opera => "opera",
            Self::SamsungInternet => "samsung_internet",
        }
    }
}

/// Operating system of the impersonated client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Os {
    Windows,
    MacOs,
    Linux,
    Android,
    Ios,
}

impl Os {
    /// Stable lowercase identifier.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::MacOs => "macos",
            Self::Linux => "linux",
            Self::Android => "android",
            Self::Ios => "ios",
        }
    }
}

/// Device class of the impersonated client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Device {
    Desktop,
    Phone,
    Tablet,
}

impl Device {
    /// Stable lowercase identifier.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Phone => "phone",
            Self::Tablet => "tablet",
        }
    }
}

/// The fingerprint identity: browser + optional exact major version,
/// OS and device class. Unset fields fall back to the resolution table.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Fingerprint {
    pub browser: Browser,
    pub version: Option<u16>,
    pub os: Option<Os>,
    pub device: Option<Device>,
}

impl Fingerprint {
    /// A bare identity; fill optionals with the `with_*` setters.
    #[must_use]
    pub const fn new(browser: Browser) -> Self {
        Self {
            browser,
            version: None,
            os: None,
            device: None,
        }
    }

    /// Latest-known version/os/device defaults for a browser. Latest-known
    /// values are filled by the resolution table (Task 4); today this
    /// returns a bare identity.
    #[must_use]
    pub const fn default_for(browser: Browser) -> Self {
        Self::new(browser)
    }

    /// Platform-sensible default. Windows → Chrome/Windows,
    /// macOS → Safari/macOS, Android → Chrome/Android, everything else
    /// (incl. Linux) → Firefox/Linux-desktop.
    #[must_use]
    pub const fn platform_default() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self::new(Browser::Chrome).with_os(Os::Windows)
        }
        #[cfg(target_os = "macos")]
        {
            Self::new(Browser::Safari).with_os(Os::MacOs)
        }
        #[cfg(target_os = "android")]
        {
            Self::new(Browser::Chrome).with_os(Os::Android)
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "android")))]
        {
            Self::new(Browser::Firefox)
                .with_os(Os::Linux)
                .with_device(Device::Desktop)
        }
    }

    #[must_use]
    pub const fn with_version(mut self, version: u16) -> Self {
        self.version = Some(version);
        self
    }

    #[must_use]
    pub const fn with_os(mut self, os: Os) -> Self {
        self.os = Some(os);
        self
    }

    #[must_use]
    pub const fn with_device(mut self, device: Device) -> Self {
        self.device = Some(device);
        self
    }

    /// Human-readable identity: `chrome@133/windows/desktop`; unset
    /// parts render as `-`.
    #[must_use]
    pub fn render(&self) -> String {
        let v = self.version.map_or_else(|| "-".into(), |v| v.to_string());
        let os = self.os.map_or_else(|| "-", Os::name);
        let d = self.device.map_or_else(|| "-", Device::name);
        format!("{}/{v}/{os}/{}", self.browser.name(), d)
    }
}

impl Default for Fingerprint {
    fn default() -> Self {
        Self::platform_default()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_for_returns_bare_identity_until_table_lands() {
        let fp = Fingerprint::default_for(Browser::Chrome);
        assert_eq!(fp.browser, Browser::Chrome);
        assert!(fp.version.is_none());
        assert!(fp.device.is_none());
    }

    #[test]
    fn platform_default_matches_target_os_family() {
        let fp = Fingerprint::platform_default();
        #[cfg(target_os = "linux")]
        assert_eq!((fp.browser, fp.os), (Browser::Firefox, Some(Os::Linux)));
        #[cfg(target_os = "macos")]
        assert_eq!((fp.browser, fp.os), (Browser::Safari, Some(Os::MacOs)));
        #[cfg(target_os = "windows")]
        assert_eq!((fp.browser, fp.os), (Browser::Chrome, Some(Os::Windows)));
        #[cfg(target_os = "android")]
        assert_eq!((fp.browser, fp.os), (Browser::Chrome, Some(Os::Android)));
    }

    #[test]
    fn builder_style_setters_consume_self() {
        let fp = Fingerprint::new(Browser::Firefox)
            .with_version(120)
            .with_os(Os::Linux)
            .with_device(Device::Desktop);
        assert_eq!(fp.version, Some(120));
        assert_eq!(fp.os, Some(Os::Linux));
    }

    #[test]
    fn serde_roundtrip() {
        let fp = Fingerprint::new(Browser::Chrome).with_version(133);
        let json = serde_json::to_string(&fp).unwrap();
        assert_eq!(serde_json::from_str::<Fingerprint>(&json).unwrap(), fp);
    }
}
