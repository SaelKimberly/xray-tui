//! Feature-gated production DNS adapter bridging `xray-tui-dns` to the
//! engine's [`DnsSink`](crate::resolve::DnsSink) seam.

use std::{future::Future, net::IpAddr, pin::Pin, sync::Arc};

use crate::{error::RouteError, resolve::DnsSink};

/// Production [`DnsSink`] backed by [`xray_tui_dns::DnsResolver`].
pub struct DnsSinkAdapter {
    /// Shared resolver instance.
    pub resolver: Arc<xray_tui_dns::DnsResolver>,
}

impl DnsSink for DnsSinkAdapter {
    fn lookup_ip(
        &self,
        host: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<IpAddr>, RouteError>> + Send>> {
        let resolver = Arc::clone(&self.resolver);
        Box::pin(async move {
            resolver
                .lookup_ip(&host, true)
                .await
                .map_err(|e| RouteError::Resolve(e.to_string()))
        })
    }
}
