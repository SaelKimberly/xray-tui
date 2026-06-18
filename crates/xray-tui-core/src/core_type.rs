use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoreType {
    Xray,
    SingBox,
    #[default]
    Auto,
}
impl fmt::Display for CoreType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xray => write!(f, "xray"),
            Self::SingBox => write!(f, "sing-box"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseCoreTypeError(String);

impl fmt::Display for ParseCoreTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid core type: '{}'", self.0)
    }
}

impl FromStr for CoreType {
    type Err = ParseCoreTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "xray" => Ok(Self::Xray),
            "sing-box" | "singbox" => Ok(Self::SingBox),
            "auto" => Ok(Self::Auto),
            _ => Err(ParseCoreTypeError(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_matches_serde() {
        for (variant, expected) in [
            (CoreType::Xray, "xray"),
            (CoreType::SingBox, "sing-box"),
            (CoreType::Auto, "auto"),
        ] {
            assert_eq!(variant.to_string(), expected);
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, format!("\"{expected}\""));
        }
    }

    #[test]
    fn from_str_round_trip() {
        for original in [CoreType::Xray, CoreType::SingBox, CoreType::Auto] {
            let s = original.to_string();
            let parsed: CoreType = s.parse().unwrap();
            assert_eq!(parsed, original);
        }
    }

    #[test]
    fn default_is_auto() {
        assert_eq!(CoreType::default(), CoreType::Auto);
    }

    #[test]
    fn from_str_accepts_singbox_no_hyphen() {
        let parsed: CoreType = "singbox".parse().unwrap();
        assert_eq!(parsed, CoreType::SingBox);
    }

    #[test]
    fn from_str_rejects_invalid() {
        assert!("invalid".parse::<CoreType>().is_err());
    }
}
