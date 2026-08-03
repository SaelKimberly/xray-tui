use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use maxminddb::Reader;
use tokio::io::AsyncWriteExt;

const GEOLITE_DOWNLOAD: &str =
    "https://github.com/P3TERX/GeoLite.mmdb/raw/download/GeoLite2-City.mmdb";

#[derive(Debug, Clone)]
pub struct Location {
    pub country: String,
    pub city_en: Option<String>,
}

pub struct GeoIp {
    db_path: PathBuf,
    reader: tokio::sync::OnceCell<Arc<Reader<Vec<u8>>>>,
}

impl GeoIp {
    pub fn new(db_path: impl Into<PathBuf>) -> Self {
        Self {
            db_path: db_path.into(),
            reader: tokio::sync::OnceCell::new(),
        }
    }

    pub async fn location_by_ip(&self, ip: IpAddr) -> anyhow::Result<Option<Location>> {
        if self.reader.get().is_none() {
            self.ensure_db().await?;
            let reader = Reader::open_readfile(&self.db_path).map_err(|e| {
                anyhow::anyhow!("Invalid geolite file: {} ({e})", self.db_path.display())
            })?;
            let _ = self.reader.set(Arc::new(reader));
        }
        let reader = Arc::clone(self.reader.get().expect("reader set above"));
        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Option<Location>> {
            let result = reader.lookup(ip)?;
            let Some(country) =
                result.decode_path::<String>(&maxminddb::path!["country", "iso_code"])?
            else {
                return Ok(None);
            };
            let Some(city_en) =
                result.decode_path::<String>(&maxminddb::path!["city", "names", "en"])?
            else {
                return Ok(Some(Location {
                    country,
                    city_en: None,
                }));
            };
            Ok(Some(Location {
                country,
                city_en: Some(city_en),
            }))
        })
        .await??;
        Ok(result)
    }

    async fn ensure_db(&self) -> anyhow::Result<()> {
        if self.db_path.is_file() {
            return Ok(());
        }
        tracing::info!(
            "Downloading {GEOLITE_DOWNLOAD} to {}",
            self.db_path.display()
        );
        // Hard deadline: a stalled 70MB download must fail (and degrade to
        // `🏴`) instead of hanging every lookup forever.
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;
        let bytes = client
            .get(GEOLITE_DOWNLOAD)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        if let Some(parent) = self.db_path.parent()
            && !parent.as_os_str().is_empty()
        {
            tokio::fs::create_dir_all(parent).await?;
        }
        match tokio::fs::File::create_new(&self.db_path).await {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&bytes).await {
                    let _ = tokio::fs::remove_file(&self.db_path).await;
                    return Err(e.into());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "downloads 70MB GeoLite2 database over network"]
    async fn test_location_by_ip() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let geo = GeoIp::new(dir.path().join("GeoLite2-City.mmdb"));
        let result = geo.location_by_ip("193.29.139.235".parse()?).await?;
        assert!(result.is_some());
        Ok(())
    }
}
