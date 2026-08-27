//! Routing engine event stream.
//!
//! Resolved / `NetworkBreakdown` / `ProbeRecovered` variants arrive in Task 9.

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
}
