//! The purge policy: which probe evidence moves a config to Purgatory.
//!
//! This module is the ONLY owner of that decision (spec
//! `2026-09-17-purge-reason-design.md` §6/§7). The engine reports typed
//! evidence ([`FailureEvidence`]) where the failure happened; the taxonomy —
//! which classes count, and which are plausibly transient — is product policy
//! and lives here, in one function, so a revision costs one function rather
//! than a schema change.

use xray_tui_db::models::PurgeReason;
use xray_tui_native::error::FailureEvidence;

/// The verdict a failed real probe's evidence earns, or `None` for
/// "inconclusive: probe it again later".
///
/// Deliberately NOT purging: every timeout, dial/DNS/refusal failure, TLS
/// alert, protocol framing failure, and the statuses `429`/`412`/`500`/`503`.
/// Each is either indistinguishable from a transient network condition or a
/// rate-limit/bot-management answer from the CDN, and none of them is a
/// statement the origin made about this config (spec §7).
#[must_use]
pub const fn reason_for(evidence: FailureEvidence) -> Option<PurgeReason> {
    match evidence {
        FailureEvidence::RealityFallback => Some(PurgeReason::RealityFallback),
        FailureEvidence::CertNotValidForName => Some(PurgeReason::CertificateMismatch),
        FailureEvidence::CertExpired => Some(PurgeReason::CertificateExpired),
        FailureEvidence::CleartextPeer => Some(PurgeReason::NotTls),
        // The stored config cannot dial as written: no retry will change that.
        FailureEvidence::ConfigDefect => Some(PurgeReason::ConfigInvalid),
        FailureEvidence::HttpRejected(status) => match status {
            // The CDN could not reach the origin: the config's backend is gone.
            521 | 522 | 526 | 530 => Some(PurgeReason::OriginUnreachable),
            // The proxy's own answer to our exact request: the path or host is
            // wrong, or the server refuses this config.
            301 | 302 | 400 | 403 | 404 | 405 | 409 | 410 => Some(PurgeReason::TransportRejected),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::reason_for;
    use xray_tui_db::models::PurgeReason;
    use xray_tui_native::error::FailureEvidence;

    #[test]
    fn every_typed_evidence_maps_to_its_reason() {
        let cases = [
            (
                FailureEvidence::RealityFallback,
                Some(PurgeReason::RealityFallback),
            ),
            (
                FailureEvidence::CertNotValidForName,
                Some(PurgeReason::CertificateMismatch),
            ),
            (
                FailureEvidence::CertExpired,
                Some(PurgeReason::CertificateExpired),
            ),
            (FailureEvidence::CleartextPeer, Some(PurgeReason::NotTls)),
            (
                FailureEvidence::ConfigDefect,
                Some(PurgeReason::ConfigInvalid),
            ),
        ];
        for (evidence, expected) in cases {
            assert_eq!(reason_for(evidence), expected, "for {evidence:?}");
        }
    }

    #[test]
    fn the_origin_errors_are_their_own_reason() {
        for status in [521_u16, 522, 526, 530] {
            assert_eq!(
                reason_for(FailureEvidence::HttpRejected(status)),
                Some(PurgeReason::OriginUnreachable),
                "status {status} means the CDN could not reach the origin"
            );
        }
    }

    #[test]
    fn the_proxy_refusing_our_request_is_evidence() {
        for status in [301_u16, 302, 400, 403, 404, 405, 409, 410] {
            assert_eq!(
                reason_for(FailureEvidence::HttpRejected(status)),
                Some(PurgeReason::TransportRejected),
                "status {status} is the server's answer to our exact request"
            );
        }
    }

    /// The excluded classes are the point of the taxonomy: each is plausibly
    /// transient or aimed at the client, so a config must not be retired on it.
    #[test]
    fn transient_and_client_side_statuses_never_purge() {
        for status in [200_u16, 204, 412, 429, 500, 502, 503, 520, 525] {
            assert_eq!(
                reason_for(FailureEvidence::HttpRejected(status)),
                None,
                "status {status} must not purge"
            );
        }
    }
}
