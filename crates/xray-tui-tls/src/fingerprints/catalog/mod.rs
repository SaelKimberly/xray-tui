//! Generated real-world JA4 catalog (identity evidence only — never used
//! to synthesize hello bytes).

mod catalog_data;

pub use catalog_data::{CATALOG, CatalogEntry};

/// All catalog entries observed for a browser name (e.g. `"chrome"`),
/// optionally filtered to an exact major version.
#[must_use]
pub fn entries_for(application: &str, version: Option<u16>) -> Vec<&'static CatalogEntry> {
    CATALOG
        .iter()
        .filter(|e| {
            if e.application != application {
                return false;
            }
            // The application column stores the browser name; versions
            // ride the user_agent. Match on `/major.` in the UA.
            version.is_none_or(|v| e.user_agent.contains(&format!("/{v}.")))
        })
        .collect()
}

/// True when `ja4` was observed in the wild for this browser (+version).
#[must_use]
pub fn contains(application: &str, version: Option<u16>, ja4: &str) -> bool {
    entries_for(application, version)
        .iter()
        .any(|e| e.ja4 == ja4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_nonempty_and_wellformed() {
        assert!(
            CATALOG.len() > 500,
            "catalog suspiciously small: {}",
            CATALOG.len()
        );
        for e in CATALOG.iter().take(100) {
            assert!(e.ja4.starts_with("t1"), "{}", e.ja4);
            assert!(
                e.ja4.split('_').count() == 3 || e.ja4.split('_').count() == 2,
                "unexpected ja4 shape: {}",
                e.ja4
            );
            assert!(!e.application.is_empty());
        }
    }

    #[test]
    fn chrome_entries_exist() {
        assert!(!entries_for("chrome", None).is_empty());
    }
}
