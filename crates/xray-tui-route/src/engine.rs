//! Flat-rule routing engine: build-once compilation and synchronous pure
//! evaluation (`Cond::All` first-match).

use std::{
    net::{IpAddr, SocketAddr},
    sync::LazyLock,
};

use crate::{
    addr::{NetAddr, NetHost, PortRange},
    error::RouteError,
    events::RouteEvent,
    ir::{Action, Cond, DefaultRoute, NetworkMask, SniffedProtocol},
    matchers::{CidrSet, CidrSetBuilder, CompiledDomain, DomainRulesSpec},
};

/// Metadata describing one connection to route.
pub struct ConnMeta {
    /// Destination address.
    pub target: NetAddr,
    /// Transport-protocol bit mask of the connection.
    pub network: NetworkMask,
    /// Tag of the inbound the connection arrived on.
    pub inbound_tag: Option<String>,
    /// Peer address on the source side, when known.
    pub source: Option<SocketAddr>,
    /// Pre-resolved source IPs.
    pub source_resolved_ips: Vec<IpAddr>,
    /// Caller-owned leading payload bytes (spec §2 data-flow 3).
    pub payload_prefix: Option<Vec<u8>>,
    /// Application protocol detected by sniffing, when available.
    pub sniffed: Option<SniffedProtocol>,
    /// Filled by the `IfNonMatch` pass or pre-seeded by the caller.
    pub resolved_host_ips: Vec<IpAddr>,
}

impl ConnMeta {
    /// Candidate destination IPs contributed by prior resolution passes.
    ///
    /// A literal-IP [`NetHost`] is considered separately at evaluation time
    /// and does not appear here; pass-based enrichment appends to the same
    /// backing vector.
    #[must_use]
    pub fn target_ips(&self) -> &[IpAddr] {
        &self.resolved_host_ips
    }
}

/// Terminal outcome of evaluating a connection against the rule set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Forward to the outbound `tag`, optionally rewriting the address.
    Route {
        /// Outbound tag to forward through.
        tag: String,
        /// Rewritten destination, if the matched action requested one.
        override_addr: Option<NetAddr>,
    },
    /// Refuse the connection with a method.
    Reject {
        /// How the rejection manifests.
        method: crate::ir::RejectMethod,
    },
    /// Intercept into the built-in DNS resolver.
    HijackDns,
}

/// A match item compiled once at build time.
///
/// `geo_country` entries are carried verbatim but evaluate `false` this task:
/// the geoip feature resolves them later, and erasing them here would let a
/// CN-routing config compile clean while silently routing through default.
enum ItemMatcher {
    Domain(CompiledDomain),
    IpCidr {
        cidrs: CidrSet,
        private: bool,
        // Retained verbatim for the geoip feature's later resolution pass.
        #[allow(dead_code)]
        geo_country: Vec<String>,
    },
    SourceIpCidr {
        cidrs: CidrSet,
        private: bool,
        // Retained verbatim for the geoip feature's later resolution pass.
        #[allow(dead_code)]
        geo_country: Vec<String>,
    },
    Ports(Vec<PortRange>),
    SourcePorts(Vec<PortRange>),
    Network(NetworkMask),
    Protocol(SniffedProtocol),
    InboundTag(std::collections::HashSet<String>),
    /// Inert until `ConnMeta` gains an outbound-tag field (Task 12).
    OutboundTag,
}

impl ItemMatcher {
    /// Evaluates one compiled item against connection metadata.
    fn matches(&self, meta: &ConnMeta) -> bool {
        match self {
            Self::Domain(c) => match &meta.target.host {
                NetHost::Domain(d) => c.matches_domain(d.as_str()),
                // IP targets fail domain items for now; Task 12 wires
                // SNI/DNS enrichment as the reverse-lookup path.
                NetHost::Ip(_) => false,
            },
            Self::IpCidr { cidrs, private, .. } => {
                let literal_hit = match &meta.target.host {
                    NetHost::Ip(ip) => ip_candidate_matches(cidrs, *private, ip),
                    NetHost::Domain(_) => false,
                };
                literal_hit
                    || meta
                        .resolved_host_ips
                        .iter()
                        .any(|ip| ip_candidate_matches(cidrs, *private, ip))
            }
            Self::SourceIpCidr { cidrs, private, .. } => {
                let socket_hit = meta
                    .source
                    .as_ref()
                    .is_some_and(|s| ip_candidate_matches(cidrs, *private, &s.ip()));
                socket_hit
                    || meta
                        .source_resolved_ips
                        .iter()
                        .any(|ip| ip_candidate_matches(cidrs, *private, ip))
            }
            Self::Ports(ranges) => ranges.iter().any(|r| r.contains(meta.target.port)),
            // Absent source information can never satisfy source items.
            Self::SourcePorts(ranges) => meta
                .source
                .as_ref()
                .is_some_and(|s| ranges.iter().any(|r| r.contains(s.port()))),
            Self::Network(mask) => mask.contains(meta.network),
            Self::Protocol(p) => meta.sniffed == Some(*p),
            Self::InboundTag(tags) => meta.inbound_tag.as_ref().is_some_and(|t| tags.contains(t)),
            Self::OutboundTag => false,
        }
    }
}

/// CIDR or private-range membership for one candidate IP.
fn ip_candidate_matches(cidrs: &CidrSet, private: bool, ip: &IpAddr) -> bool {
    cidrs.contains(ip) || (private && private_contains(ip))
}

/// True when `ip` falls inside the shared compiled private-address set.
fn private_contains(ip: &IpAddr) -> bool {
    static PRIVATE: LazyLock<CidrSet> = LazyLock::new(CidrSet::private_set);
    PRIVATE.contains(ip)
}

/// One rule with its predicate pre-compiled into leaf matchers.
struct CompiledRule {
    name: Option<String>,
    items: Vec<ItemMatcher>,
    action: Action,
}

impl CompiledRule {
    fn matches(&self, meta: &ConnMeta) -> bool {
        self.items.iter().all(|item| item.matches(meta))
    }
}

/// First-match flat-rule routing engine.
pub struct Engine {
    rules: Vec<CompiledRule>,
    default_route: DefaultRoute,
    #[allow(dead_code)] // consulted by IfNonMatch resolution wiring (Task 9+)
    resolve_strategy: crate::ir::ResolveStrategy,
    event_sink: Option<tokio::sync::mpsc::UnboundedSender<RouteEvent>>,
}

impl Engine {
    /// Compiles a [`RuleSet`] into an evaluation-ready engine.
    ///
    /// Regexes and keyword automata compile once here; port ranges are
    /// sanity-checked so evaluation cannot encounter inverted bounds.
    ///
    /// # Errors
    /// Returns [`RouteError::Parse`] when any rule fails to compile, and
    /// [`RouteError::Unsupported`] for conditions this engine does not yet
    /// evaluate (`Cond::Any`, `Cond::Invert`).
    pub fn build(rs: crate::ir::RuleSet) -> Result<Self, RouteError> {
        let mut rules = Vec::with_capacity(rs.rules.len());
        for (idx, rule) in rs.rules.into_iter().enumerate() {
            let items = match rule.cond {
                Cond::All(items) => items
                    .into_iter()
                    .map(|item| compile_item(item, idx))
                    .collect::<Result<Vec<_>, _>>()?,
                Cond::Any(_) | Cond::Invert(_) => {
                    return Err(RouteError::Unsupported(
                        "Cond::Any/Invert not evaluated yet",
                    ));
                }
            };
            rules.push(CompiledRule {
                name: rule.name,
                items,
                action: rule.action,
            });
        }
        Ok(Self {
            rules,
            default_route: rs.default,
            resolve_strategy: rs.resolve_strategy,
            event_sink: None,
        })
    }

    /// Synchronous pure evaluation against [`ConnMeta`]: first rule whose
    /// every `Cond::All` item holds wins; otherwise the rule-set default.
    ///
    /// No DNS resolution, sniffing side effects, or I/O happens here; `meta`
    /// is reserved mutable for the Task 12 integration (DNS/SNI enrichment
    /// pass mutates `resolved_host_ips`/`sniffed`; this allow must be removed
    /// there once that lands).
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn decide(&self, meta: &mut ConnMeta) -> Decision {
        if let Some(rule) = self.rules.iter().find(|rule| rule.matches(meta)) {
            let decision = Decision::from(&rule.action);
            self.emit_decision(&decision, rule.name.as_deref());
            return decision;
        }
        let decision = Decision::from(&self.default_route);
        self.emit_decision(&decision, None);
        decision
    }

    /// Registers a sink receiving events after terminal decisions.
    pub fn set_event_sink(&mut self, tx: tokio::sync::mpsc::UnboundedSender<RouteEvent>) {
        self.event_sink = Some(tx);
    }

    /// Emits `DecisionApplied` when a sink is registered; drops on send error
    /// (a closed receiver must never fail routing). Tag extraction happens
    /// only here so the sink-less hot path clones nothing.
    fn emit_decision(&self, decision: &Decision, rule_name: Option<&str>) {
        let Some(tx) = &self.event_sink else { return };
        let tag = match decision {
            Decision::Route { tag, .. } => Some(tag.clone()),
            _ => None,
        };
        let _ = tx.send(RouteEvent::DecisionApplied {
            rule_name: rule_name.map(str::to_owned),
            tag,
            sni: None,
            at: jiff::Timestamp::now(),
        });
    }
}

impl From<&Action> for Decision {
    fn from(action: &Action) -> Self {
        match action {
            Action::Route { tag, override_addr } => Self::Route {
                tag: tag.clone(),
                override_addr: override_addr.clone(),
            },
            Action::Reject { method } => Self::Reject { method: *method },
            Action::HijackDns => Self::HijackDns,
        }
    }
}

impl From<&DefaultRoute> for Decision {
    fn from(default: &DefaultRoute) -> Self {
        match default {
            DefaultRoute::Route { tag } => Self::Route {
                tag: tag.clone(),
                override_addr: None,
            },
            DefaultRoute::Reject { method } => Self::Reject { method: *method },
        }
    }
}

/// Compiles one [`crate::ir::MatchItem`] with rule-index attribution on errors.
fn compile_item(item: crate::ir::MatchItem, idx: usize) -> Result<ItemMatcher, RouteError> {
    use crate::ir::MatchItem as M;
    Ok(match item {
        M::Domain {
            exact,
            suffix,
            keywords,
            regexes,
        } => ItemMatcher::Domain(
            CompiledDomain::build(&DomainRulesSpec {
                exact,
                suffix,
                keywords,
                regexes,
            })
            .map_err(|e| reindex(e, idx))?,
        ),
        M::IpCidr {
            cidrs,
            private,
            geo_country,
        } => ItemMatcher::IpCidr {
            cidrs: build_cidr_set(cidrs, idx)?,
            private,
            geo_country,
        },
        M::SourceIpCidr {
            cidrs,
            private,
            geo_country,
        } => ItemMatcher::SourceIpCidr {
            cidrs: build_cidr_set(cidrs, idx)?,
            private,
            geo_country,
        },
        M::Ports(ranges) => ItemMatcher::Ports(check_ranges(ranges, idx, "ports")?),
        M::SourcePorts(ranges) => {
            ItemMatcher::SourcePorts(check_ranges(ranges, idx, "source_ports")?)
        }
        M::Network(mask) => ItemMatcher::Network(mask),
        M::Protocol(p) => ItemMatcher::Protocol(p),
        M::InboundTag { tags } => ItemMatcher::InboundTag(tags.into_iter().collect()),
        M::OutboundTag { .. } => ItemMatcher::OutboundTag,
    })
}

/// Builds a CIDR set, attributing out-of-range prefixes to their rule.
fn build_cidr_set(cidrs: Vec<crate::addr::Cidr>, idx: usize) -> Result<CidrSet, RouteError> {
    let mut builder = CidrSetBuilder::default();
    for c in cidrs {
        builder.insert(c).map_err(|e| reindex(e, idx))?;
    }
    Ok(builder.build())
}

/// Rejects inverted `[start, end]` port ranges at compile time.
fn check_ranges(
    ranges: Vec<PortRange>,
    idx: usize,
    field: &'static str,
) -> Result<Vec<PortRange>, RouteError> {
    for r in &ranges {
        if r.start > r.end {
            return Err(RouteError::Parse {
                rule_index: idx,
                field,
                message: format!("inverted range {}-{}", r.start, r.end),
            });
        }
    }
    Ok(ranges)
}

/// Rewrites a matcher error's placeholder rule index to the real one.
const fn reindex(mut e: RouteError, idx: usize) -> RouteError {
    if let RouteError::Parse { rule_index, .. } = &mut e {
        *rule_index = idx;
    }
    e
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use super::*;
    use crate::{
        addr::Cidr,
        ir::{Action, Cond, DefaultRoute, MatchItem, RejectMethod, ResolveStrategy, Rule, RuleSet},
    };

    const ALL_PORTS: PortRange = PortRange {
        start: 0,
        end: u16::MAX,
    };

    fn base_default() -> RuleSet {
        RuleSet {
            rules: vec![],
            default: DefaultRoute::Route {
                tag: "direct".to_owned(),
            },
            resolve_strategy: ResolveStrategy::AsIs,
            probes: vec![],
        }
    }

    fn single_item_rule(item: MatchItem, action: Action) -> Rule {
        Rule {
            name: None,
            cond: Cond::All(vec![item]),
            action,
        }
    }

    fn rs(rules: Vec<(MatchItem, Action)>) -> RuleSet {
        let mut set = base_default();
        set.rules = rules
            .into_iter()
            .map(|(item, action)| single_item_rule(item, action))
            .collect();
        set
    }

    fn meta(host: &str, port: u16, network: NetworkMask) -> ConnMeta {
        ConnMeta {
            target: NetAddr {
                host: NetHost::new(host),
                port,
            },
            network,
            inbound_tag: None,
            source: None,
            source_resolved_ips: vec![],
            payload_prefix: None,
            sniffed: None,
            resolved_host_ips: vec![],
        }
    }

    fn route(tag: &str) -> Action {
        Action::Route {
            tag: tag.to_owned(),
            override_addr: None,
        }
    }

    fn domain(exact: &[&str], suffix: &[&str]) -> MatchItem {
        MatchItem::Domain {
            exact: exact.iter().map(ToString::to_string).collect(),
            suffix: suffix.iter().map(ToString::to_string).collect(),
            keywords: vec![],
            regexes: vec![],
        }
    }

    #[test]
    fn first_match_wins_in_declaration_order() {
        let e = Engine::build(rs(vec![
            (
                MatchItem::Ports(vec![PortRange { start: 80, end: 80 }]),
                route("a"),
            ),
            (MatchItem::Network(NetworkMask::TCP), route("b")),
        ]))
        .unwrap();
        assert_eq!(
            e.decide(&mut meta("example.com", 80, NetworkMask::TCP)),
            Decision::Route {
                tag: "a".to_owned(),
                override_addr: None
            }
        );
    }

    #[test]
    fn unmatched_items_skip_the_rule_not_abort() {
        let mut set = base_default();
        set.rules.push(Rule {
            name: None,
            cond: Cond::All(vec![
                MatchItem::Ports(vec![PortRange { start: 80, end: 80 }]),
                MatchItem::Network(NetworkMask::TCP),
            ]),
            action: route("blocked"),
        });
        let e = Engine::build(set).unwrap();
        assert_eq!(
            e.decide(&mut meta("example.com", 443, NetworkMask::UDP)),
            Decision::Route {
                tag: "direct".to_owned(),
                override_addr: None
            }
        );
    }

    #[test]
    fn default_fallback_when_nothing_matched() {
        let e = Engine::build(rs(vec![(
            MatchItem::Ports(vec![PortRange {
                start: 9999,
                end: 9999,
            }]),
            route("rare"),
        )]))
        .unwrap();
        assert_eq!(
            e.decide(&mut meta("example.org", 443, NetworkMask::TCP)),
            Decision::Route {
                tag: "direct".to_owned(),
                override_addr: None
            }
        );
    }

    #[test]
    fn reject_and_hijackdns_are_terminal() {
        let reject = Engine::build(rs(vec![(
            MatchItem::Ports(vec![ALL_PORTS]),
            Action::Reject {
                method: RejectMethod::Drop,
            },
        )]))
        .unwrap();
        assert_eq!(
            reject.decide(&mut meta("example.com", 443, NetworkMask::TCP)),
            Decision::Reject {
                method: RejectMethod::Drop
            }
        );

        let hijack = Engine::build(rs(vec![(
            MatchItem::Ports(vec![ALL_PORTS]),
            Action::HijackDns,
        )]))
        .unwrap();
        assert_eq!(
            hijack.decide(&mut meta("dns.example.com", 53, NetworkMask::UDP)),
            Decision::HijackDns
        );

        // DefaultRoute maps identically.
        let mut set = base_default();
        set.default = DefaultRoute::Reject {
            method: RejectMethod::DefaultReply,
        };
        let default_reject = Engine::build(set).unwrap();
        assert_eq!(
            default_reject.decide(&mut meta("example.com", 1234, NetworkMask::TCP)),
            Decision::Reject {
                method: RejectMethod::DefaultReply
            }
        );
    }

    #[test]
    fn event_sink_receives_decision_applied_after_terminal_decision() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut rules = rs(vec![(
            domain(&["block.example.net"], &[]),
            route("blackhole"),
        )]);
        rules.rules[0].name = Some("ad-block".to_owned());
        let mut e = Engine::build(rules).unwrap();
        e.set_event_sink(tx);

        let d = e.decide(&mut meta("block.example.net", 80, NetworkMask::TCP));
        assert_eq!(
            d,
            Decision::Route {
                tag: "blackhole".to_owned(),
                override_addr: None
            },
            "decision itself is unchanged by the sink"
        );

        let ev = rx
            .try_recv()
            .expect("event emitted after terminal decision");
        let crate::events::RouteEvent::DecisionApplied {
            rule_name,
            tag,
            sni,
            at,
        } = ev
        else {
            panic!("expected DecisionApplied, got other variant");
        };
        assert_eq!(rule_name.as_deref(), Some("ad-block"));
        assert_eq!(tag.as_deref(), Some("blackhole"));
        assert_eq!(sni, None, "sni stays None until Task 12 wires sniffing");
        let _ = at; // wall-clock timestamp present
    }

    #[test]
    fn no_sink_means_no_event_and_decisions_still_flow() {
        let e = Engine::build(rs(vec![(MatchItem::Ports(vec![ALL_PORTS]), route("a"))])).unwrap();
        assert!(
            e.decide(&mut meta("h.example", 1, NetworkMask::UDP))
                .is_routed_to("a")
        );
    }

    #[test]
    fn domain_exact_and_suffix_match_destination_only_when_domain_target() {
        let e = Engine::build(rs(vec![(
            domain(&["example.com"], &["sub.corp"]),
            route("d"),
        )]))
        .unwrap();
        assert!(
            e.decide(&mut meta("example.com", 80, NetworkMask::TCP))
                .is_routed_to("d")
        );
        assert!(
            e.decide(&mut meta("x.sub.CORP", 80, NetworkMask::TCP))
                .is_routed_to("d")
        );
        // Suffix requires a label boundary.
        assert!(
            !e.decide(&mut meta("xfoo.com", 80, NetworkMask::TCP))
                .is_routed_to("d")
        );
        // IP targets fail domain items this task (Task 12 wires enrichment).
        assert!(
            !e.decide(&mut meta("93.184.216.34", 80, NetworkMask::TCP))
                .is_routed_to("d")
        );
    }

    #[test]
    fn regexes_participate_in_domain_matching() {
        let item = MatchItem::Domain {
            exact: vec![],
            suffix: vec![],
            keywords: vec![],
            regexes: vec![r"\.cdn\d*\.net$".to_owned()],
        };
        let e = Engine::build(rs(vec![(item, route("fast"))])).unwrap();
        assert!(
            e.decide(&mut meta("img.cdn7.net", 443, NetworkMask::TCP))
                .is_routed_to("fast")
        );
        assert!(
            !e.decide(&mut meta("plain.net", 443, NetworkMask::TCP))
                .is_routed_to("fast")
        );
    }

    #[test]
    fn ip_cidr_hits_via_literal_target_or_resolved_host_ips() {
        let cidrs = || vec![Cidr::parse("10.0.0.0/8").unwrap()];
        let e = Engine::build(rs(vec![(
            MatchItem::IpCidr {
                cidrs: cidrs(),
                private: false,
                geo_country: vec![],
            },
            route("lan"),
        )]))
        .unwrap();
        // Literal IP target.
        assert!(
            e.decide(&mut meta("10.1.2.3", 8000, NetworkMask::TCP))
                .is_routed_to("lan")
        );
        assert!(
            !e.decide(&mut meta("8.8.8.8", 8000, NetworkMask::TCP))
                .is_routed_to("lan")
        );
        // Domain target resolved to a LAN IP counts as a hit.
        let mut m = meta("nas.home", 5000, NetworkMask::TCP);
        m.resolved_host_ips.push(IpAddr::from([10u8, 9, 9, 9]));
        assert!(e.decide(&mut m).is_routed_to("lan"));
    }

    #[test]
    fn private_cidr_matches_private_ranges_and_geo_country_is_inert() {
        let e = Engine::build(rs(vec![(
            MatchItem::IpCidr {
                cidrs: vec![],
                private: true,
                geo_country: vec![],
            },
            route("home"),
        )]))
        .unwrap();
        assert!(
            e.decide(&mut meta("192.168.1.5", 22, NetworkMask::TCP))
                .is_routed_to("home")
        );
        assert!(
            !e.decide(&mut meta("8.8.8.8", 22, NetworkMask::TCP))
                .is_routed_to("home")
        );

        // geo_country is carried verbatim through compilation but evaluates
        // false this task; the geoip feature resolves it later.
        let geo = Engine::build(rs(vec![(
            MatchItem::IpCidr {
                cidrs: vec![],
                private: false,
                geo_country: vec!["CN".into()],
            },
            route("cn"),
        )]))
        .unwrap();
        assert!(
            !geo.decide(&mut meta("1.2.3.4", 80, NetworkMask::TCP))
                .is_routed_to("cn")
        );
        // Entries survive compilation verbatim (no silent erasure): a later
        // task's geoip feature finds them right where it needs them.
        let ItemMatcher::IpCidr { geo_country, .. } = &geo.rules[0].items[0] else {
            panic!("expected compiled IpCidr item");
        };
        assert_eq!(geo_country, &["CN".to_owned()]);
        // Source-side mirror carries them too.
        let src_geo = Engine::build(rs(vec![(
            MatchItem::SourceIpCidr {
                cidrs: vec![],
                private: false,
                geo_country: vec!["DE".into()],
            },
            route("de"),
        )]))
        .unwrap();
        let ItemMatcher::SourceIpCidr { geo_country, .. } = &src_geo.rules[0].items[0] else {
            panic!("expected compiled SourceIpCidr item");
        };
        assert_eq!(geo_country, &["DE".to_owned()]);
    }

    #[test]
    fn source_items_need_source_data_and_use_both_source_fields() {
        let sp = |p: u16| MatchItem::SourcePorts(vec![PortRange { start: p, end: p }]);
        let e = Engine::build(rs(vec![(sp(5000), route("elevated"))])).unwrap();
        // Absent source => item false.
        assert!(
            !e.decide(&mut meta("srv.example", 443, NetworkMask::TCP))
                .is_routed_to("elevated")
        );

        let mut with_src = meta("srv.example", 443, NetworkMask::TCP);
        with_src.source = Some(SocketAddr::from(([127u8, 0, 0, 1], 5000)));
        assert!(e.decide(&mut with_src).is_routed_to("elevated"));

        // SourceIpCidr reads source_resolved_ips too.
        let sic = MatchItem::SourceIpCidr {
            cidrs: vec![Cidr::parse("10.0.0.0/8").unwrap()],
            private: false,
            geo_country: vec![],
        };
        let e2 = Engine::build(rs(vec![(sic, route("corp-client"))])).unwrap();
        let mut via_resolved = meta("relay.example", 993, NetworkMask::TCP);
        via_resolved
            .source_resolved_ips
            .push(IpAddr::from([10u8, 77, 0, 1]));
        assert!(e2.decide(&mut via_resolved).is_routed_to("corp-client"));
    }

    #[test]
    fn network_mask_requires_connection_within_item_bits() {
        // Item tcp&&udp accepts either transport.
        let both = Engine::build(rs(vec![(
            MatchItem::Network(NetworkMask {
                tcp: true,
                udp: true,
            }),
            route("any"),
        )]))
        .unwrap();
        assert!(
            both.decide(&mut meta("h", 1, NetworkMask::TCP))
                .is_routed_to("any")
        );
        assert!(
            both.decide(&mut meta("h", 1, NetworkMask::UDP))
                .is_routed_to("any")
        );

        // Single-bit item needs equality: conn carrying both bits escapes it.
        let tcp_only = Engine::build(rs(vec![(
            MatchItem::Network(NetworkMask::TCP),
            route("tcp"),
        )]))
        .unwrap();
        let mut both_bits = meta(
            "h",
            1,
            NetworkMask {
                tcp: true,
                udp: true,
            },
        );
        assert!(!tcp_only.decide(&mut both_bits).is_routed_to("tcp"));
    }

    #[test]
    fn protocol_requires_sniffed_agreement_and_none_misses() {
        let e = Engine::build(rs(vec![(
            MatchItem::Protocol(crate::ir::SniffedProtocol::Tls),
            route("tls-out"),
        )]))
        .unwrap();
        assert!(
            !e.decide(&mut meta("a.b", 443, NetworkMask::TCP))
                .is_routed_to("tls-out")
        );
        let mut tls = meta("a.b", 443, NetworkMask::TCP);
        tls.sniffed = Some(crate::ir::SniffedProtocol::Tls);
        assert!(e.decide(&mut tls).is_routed_to("tls-out"));
        let mut dns = meta("a.b", 443, NetworkMask::TCP);
        dns.sniffed = Some(crate::ir::SniffedProtocol::Dns);
        assert!(!e.decide(&mut dns).is_routed_to("tls-out"));
    }

    #[test]
    fn inbound_tag_membership_matches_outbound_tag_inert_until_field_exists() {
        let ib = MatchItem::InboundTag {
            tags: vec!["proxy-in".to_owned()],
        };
        let e = Engine::build(rs(vec![(ib, route("in"))])).unwrap();
        let mut tagged = meta("h", 1, NetworkMask::TCP);
        tagged.inbound_tag = Some("proxy-in".to_owned());
        assert!(e.decide(&mut tagged).is_routed_to("in"));
        let mut wrong = meta("h", 1, NetworkMask::TCP);
        wrong.inbound_tag = Some("other".to_owned());
        assert!(!e.decide(&mut wrong).is_routed_to("in"));

        // No ConnMeta.outbound_tag field exists yet; the item never matches.
        let ob = MatchItem::OutboundTag {
            tags: vec!["x".to_owned()],
        };
        let e2 = Engine::build(rs(vec![(ob, route("out"))])).unwrap();
        assert!(
            !e2.decide(&mut meta("h", 1, NetworkMask::TCP))
                .is_routed_to("out")
        );
    }

    #[test]
    fn target_ips_exposes_pass_resolution_results() {
        let mut m = meta("mixed.example", 443, NetworkMask::TCP);
        assert!(m.target_ips().is_empty());
        m.resolved_host_ips.push(IpAddr::from([203u8, 0, 113, 7]));
        assert_eq!(m.target_ips().len(), 1);
    }

    #[test]
    fn build_validates_compilable_pieces() {
        // Invalid regex fails at build time, not decide time.
        let bad_regex = MatchItem::Domain {
            exact: vec![],
            suffix: vec![],
            keywords: vec![],
            regexes: vec!["[".to_owned()],
        };
        let res = Engine::build(rs(vec![(bad_regex, route("x"))]));
        assert!(res.is_err(), "invalid regex must fail at build time");

        // Out-of-range CIDR prefix length fails at build time.
        let res = Engine::build(rs(vec![(
            MatchItem::IpCidr {
                cidrs: vec![Cidr {
                    addr: IpAddr::from([10u8, 0, 0, 0]),
                    bits: 33,
                }],
                private: false,
                geo_country: vec![],
            },
            route("x"),
        )]));
        assert!(
            res.is_err(),
            "out-of-range prefix length must fail at build time"
        );

        // Inverted port range sanity check.
        let res = Engine::build(rs(vec![(
            MatchItem::Ports(vec![PortRange {
                start: 200,
                end: 100,
            }]),
            route("x"),
        )]));
        assert!(res.is_err(), "inverted port range must fail at build time");

        // Cond variants beyond All are not supported by this engine yet.
        let any_set = RuleSet {
            rules: vec![Rule {
                name: None,
                cond: Cond::Any(vec![Cond::All(vec![])]),
                action: route("x"),
            }],
            ..base_default()
        };
        assert!(matches!(
            Engine::build(any_set),
            Err(RouteError::Unsupported(_))
        ));
    }

    /// Test-only helper naming which outbound a decision selects.
    trait DecisionExt {
        fn is_routed_to(&self, tag: &str) -> bool;
    }
    impl DecisionExt for Decision {
        fn is_routed_to(&self, tag: &str) -> bool {
            matches!(self, Self::Route { tag: t, .. } if t == tag)
        }
    }
}
