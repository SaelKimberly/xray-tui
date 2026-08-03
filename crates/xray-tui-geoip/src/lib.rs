use std::future::Future;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use maxminddb::Reader;
use tokio::io::AsyncWriteExt;

const GEOLITE_DOWNLOAD: &str =
    "https://github.com/P3TERX/GeoLite.mmdb/raw/download/GeoLite2-City.mmdb";

type FetchResult = Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>>> + Send>>;
type Fetcher = dyn Fn() -> FetchResult + Send + Sync;

#[derive(Debug, Clone)]
pub struct Location {
    pub country: String,
    pub city_en: Option<String>,
}

pub struct GeoIp {
    db_path: PathBuf,
    reader: tokio::sync::OnceCell<Arc<Reader<Vec<u8>>>>,
    init_lock: tokio::sync::Mutex<()>,
    fetch: Arc<Fetcher>,
}

impl GeoIp {
    pub fn new(db_path: impl Into<PathBuf>) -> Self {
        Self::new_with_fetcher(db_path, || Box::pin(fetch_geolite_bytes()))
    }

    /// Build a `GeoIp` backed by a custom fetcher, so tests can avoid the
    /// real 70 MB network download.
    pub fn new_with_fetcher<F>(db_path: impl Into<PathBuf>, fetcher: F) -> Self
    where
        F: Fn() -> FetchResult + Send + Sync + 'static,
    {
        Self {
            db_path: db_path.into(),
            reader: tokio::sync::OnceCell::new(),
            init_lock: tokio::sync::Mutex::new(()),
            fetch: Arc::new(fetcher),
        }
    }

    pub async fn location_by_ip(&self, ip: IpAddr) -> anyhow::Result<Option<Location>> {
        if self.reader.get().is_none() {
            // Serialize first init so concurrent lookups can't race the
            // download and observe a partially-written database.
            let _guard = self.init_lock.lock().await;
            // Re-check under the lock: an earlier task may have initialized.
            if self.reader.get().is_none() {
                let reader = self.open_reader_healing().await?;
                let _ = self.reader.set(Arc::new(reader));
            }
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

    fn open_reader(path: &Path) -> anyhow::Result<Reader<Vec<u8>>> {
        Reader::open_readfile(path)
            .map_err(|e| anyhow::anyhow!("Invalid geolite file: {} ({e})", path.display()))
    }

    /// Open the database, healing a corrupt/truncated file by deleting it and
    /// re-downloading once before giving up.
    async fn open_reader_healing(&self) -> anyhow::Result<Reader<Vec<u8>>> {
        self.ensure_db().await?;
        match Self::open_reader(&self.db_path) {
            Ok(reader) => Ok(reader),
            Err(first_err) => {
                tracing::warn!(
                    "GeoLite db {} is corrupt ({first_err}); removing and re-downloading",
                    self.db_path.display()
                );
                // Ignore a missing file: it may already have been removed.
                let _ = tokio::fs::remove_file(&self.db_path).await;
                self.ensure_db().await?;
                Self::open_reader(&self.db_path)
            }
        }
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
        let bytes = (self.fetch)().await?;
        if let Some(parent) = self.db_path.parent()
            && !parent.as_os_str().is_empty()
        {
            tokio::fs::create_dir_all(parent).await?;
        }
        // Download to a temp path and rename into place, so a crash or a
        // concurrent reader can never observe a partially-written database.
        let tmp_path = self.db_path.with_file_name(format!(
            "{}.tmp{}",
            self.db_path
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default(),
            std::process::id()
        ));
        let write = async {
            let mut file = tokio::fs::File::create(&tmp_path).await?;
            file.write_all(&bytes).await?;
            file.sync_all().await?;
            tokio::fs::rename(&tmp_path, &self.db_path).await
        };
        if let Err(e) = write.await {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(e.into());
        }
        Ok(())
    }
}

async fn fetch_geolite_bytes() -> anyhow::Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_mins(2))
        .build()?;
    let bytes = client
        .get(GEOLITE_DOWNLOAD)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    Ok(bytes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn failing_fetcher(msg: &'static str) -> impl Fn() -> FetchResult + Send + Sync {
        move || Box::pin(async move { Err(anyhow::anyhow!("{msg}")) })
    }

    fn bytes_fetcher(bytes: Vec<u8>) -> impl Fn() -> FetchResult + Send + Sync {
        move || {
            let bytes = bytes.clone();
            Box::pin(async move { Ok(bytes) })
        }
    }

    /// A fetcher that counts invocations and tracks the maximum number of
    /// concurrent downloads, so tests can assert first init is serialized.
    fn counting_fetcher(
        bytes: Vec<u8>,
        calls: Arc<AtomicUsize>,
        active: Arc<AtomicUsize>,
        max_active: Arc<AtomicUsize>,
    ) -> impl Fn() -> FetchResult + Send + Sync {
        move || {
            let bytes = bytes.clone();
            let calls = Arc::clone(&calls);
            let active = Arc::clone(&active);
            let max_active = Arc::clone(&max_active);
            Box::pin(async move {
                let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                max_active.fetch_max(now, Ordering::SeqCst);
                // Keep the download in flight long enough for a racing second
                // lookup to reach the same spot, detecting non-serialized init.
                tokio::time::sleep(Duration::from_millis(300)).await;
                active.fetch_sub(1, Ordering::SeqCst);
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(bytes)
            })
        }
    }

    #[tokio::test]
    async fn corrupt_db_is_healed_by_redownload() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("GeoLite2-City.mmdb");
        // A corrupt file that `Reader::open_readfile` cannot parse.
        tokio::fs::write(&db_path, b"this is not a maxmind database")
            .await
            .unwrap();

        // The re-download fails; the contract is that the corrupt file is
        // still removed so it can't keep poisoning every lookup.
        let geo = GeoIp::new_with_fetcher(
            db_path.as_path(),
            failing_fetcher("simulated download failure"),
        );

        let result = geo.location_by_ip("193.29.139.235".parse().unwrap()).await;
        assert!(result.is_err(), "corrupt db must surface an error");
        assert!(
            !db_path.exists(),
            "corrupt db must not remain as a permanent tombstone"
        );
    }

    #[tokio::test]
    async fn concurrent_first_init_downloads_serially() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let db_path = dir.path().join("GeoLite2-City.mmdb");
        let calls = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let geo = Arc::new(GeoIp::new_with_fetcher(
            db_path.as_path(),
            counting_fetcher(
                b"garbage".to_vec(),
                Arc::clone(&calls),
                Arc::clone(&active),
                Arc::clone(&max_active),
            ),
        ));

        let mut handles = Vec::new();
        for _ in 0..2 {
            let geo = Arc::clone(&geo);
            handles.push(tokio::spawn(async move {
                let _ = geo.location_by_ip("193.29.139.235".parse().unwrap()).await;
            }));
        }
        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(
            max_active.load(Ordering::SeqCst),
            1,
            "first init must be serialized ({} concurrent downloads observed)",
            max_active.load(Ordering::SeqCst)
        );
        Ok(())
    }

    #[tokio::test]
    async fn ensure_db_skips_download_when_file_exists() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let db_path = dir.path().join("GeoLite2-City.mmdb");
        tokio::fs::write(&db_path, b"existing db").await?;

        let calls = Arc::new(AtomicUsize::new(0));
        let fetcher_calls = Arc::clone(&calls);
        let geo = GeoIp::new_with_fetcher(db_path.as_path(), move || {
            let calls = Arc::clone(&fetcher_calls);
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(Vec::new())
            })
        });

        geo.ensure_db().await?;
        assert_eq!(calls.load(Ordering::SeqCst), 0, "must not re-download");
        Ok(())
    }

    #[tokio::test]
    async fn ensure_db_writes_atomically_via_temp_file() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let db_path = dir.path().join("GeoLite2-City.mmdb");
        let geo = GeoIp::new_with_fetcher(
            db_path.as_path(),
            bytes_fetcher(b"fake mmdb bytes".to_vec()),
        );

        geo.ensure_db().await?;
        assert_eq!(tokio::fs::read(&db_path).await?, b"fake mmdb bytes");

        // No temp files should be left behind after the rename.
        let mut entries = tokio::fs::read_dir(dir.path()).await?;
        let mut names = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
        assert_eq!(names, vec!["GeoLite2-City.mmdb".to_owned()]);
        Ok(())
    }

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
