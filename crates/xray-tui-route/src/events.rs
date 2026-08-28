//! Routing engine event stream.
//!
//! Emitted during configuration compilation and connection decisions.

/// Events emitted by the routing engine during configuration compilation and
/// connection decisions.
#[derive(Debug, Clone)]
pub enum RouteEvent {
    /// A terminal routing decision was reached for a connection.
    DecisionApplied {
        /// Name of the matched rule; `None` when the default fell through.
        rule_name: Option<String>,
        /// Target outbound tag for routed decisions, `None` otherwise.
        tag: Option<String>,
        /// Sniffed TLS SNI host (`None` until Task 12 wires sniffing).
        sni: Option<String>,
        /// Wall-clock time of the decision.
        at: jiff::Timestamp,
    },
    /// Non-fatal issue found while compiling the rule set.
    CompileWarning {
        /// Index into [`crate::ir::RuleSet::rules`].
        rule_index: usize,
        /// Human-readable description of the issue.
        message: String,
    },
    /// A DNS resolution completed for a host.
    Resolved {
        /// The hostname that was resolved.
        host: String,
        /// All resolved addresses.
        ips: Vec<std::net::IpAddr>,
        /// Wall-clock time of the resolution.
        at: jiff::Timestamp,
    },
    /// A probe target stopped answering (first failure of a streak).
    NetworkBreakdown {
        /// The probe that failed.
        failed_probe: String,
        /// Wall-clock time of the failure.
        at: jiff::Timestamp,
    },
    /// A previously failed probe answered again (streak reset).
    ProbeRecovered {
        /// The probe that recovered.
        probe: String,
        /// Wall-clock time of the recovery.
        at: jiff::Timestamp,
    },
}
