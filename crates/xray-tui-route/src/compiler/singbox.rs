//! sing-box route JSON compiler.
//!
//! Vocabulary mapping (sing-box `route.rules`, verified against
//! `thirdparty/sing-box/docs/configuration/route/rule.md` and
//! `option/rule.go`):
//!
//! | sing-box token / field            | IR target                                    |
//! |-----------------------------------|----------------------------------------------|
//! | `domain`                          | `Domain.exact` AND `Domain.suffix` pair      |
//! | `domain_suffix`                   | `Domain.suffix`                              |
//! | `domain_keyword`                  | `Domain.keywords`                            |
//! | `domain_regex`                    | `Domain.regexes`                             |
//! | `ip_cidr`                         | `IpCidr.cidrs` (bare hosts gain /32 or /128) |
//! | `ip_is_private`                   | `IpCidr.private = true`                      |
//! | `source_ip_cidr`                  | `SourceIpCidr.cidrs`                         |
//! | `source_ip_is_private`            | `SourceIpCidr.private = true`                |
//! | `port`                            | `Ports` (exact ports)                        |
//! | `port_range` `1000:2000`          | `Ports` (colon → dash range, per ruling)     |
//! | `source_port`, `source_port_range`| `SourcePorts`                                |
//! | `network` tcp/udp tokens          | `Network(NetworkMask)`                       |
//! | `inbound`                         | `InboundTag.tags`                            |
//! | `outbound`                        | `OutboundTag.tags`                           |
//! | `protocol` http/tls/dns only      | `Protocol`; anything else Unsupported        |
//! | no `action` key                   | [`RouteError::Parse`] per dispatch ruling    |
//! | action `route` + `outbound`       | `Action::Route { tag }`                      |
//! | action `route` w/o `outbound`     | [`RouteError::Parse`]                        |
//! | route to outbound `bypass`/`direct` | collapse onto tag "direct"                 |
//! | action `hijack-dns`               | `Action::HijackDns`                          |
//! | action `reject` `method`          | `Action::Reject` (default/drop/reply per     |
//! |                                   |   sing-box `rule_action.go`)                 |
//! | action `sniff`/`resolve`/…        | [`RouteError::Unsupported`]                  |
//! | `rule_set` refs, `dns-router`     | [`RouteError::Unsupported`] (T9+ accelerators)|
//! | `invert` (flat or logical)        | [`RouteError::Unsupported`] (awaits T11)     |
//! | logical `mode:"and"`              | children flattened into `Cond::All`          |
//! | logical `mode:"or"`               | distributive flat rules sharing the action   |
//!
//! Cross-field conjunction and same-field alternation mirror upstream
//! semantics (route.md matching logic: `domain`-family fields are OR'd within
//! their group, groups AND across). Because `Cond::All` holds leaf items only
//! and today's engine rejects `Cond::Any` (T11), a rule mixing a multi-value
//! alternative field with other AND fields expands distributively —
//! (A∧B∧p) ∨ (A∧B∧q) — into flat `Cond::All` rules sharing the action;
//! first-match semantics are preserved. Same ruling Task 5 applied to xray
//! multi-protocol whitelists. A T11 note on the OR test documents where real
//! `Cond::Any` emission replaces expansion.
//!
//! A rule with no matchable condition is skipped with a warning. Unknown rule
//! keys are tolerated as warnings; unknown *values* abort with positional
//! [`RouteError::Parse`]. With no `route.final` present, `ruleset.default` is
//! the documented constant [`DEFAULT_SINGBOX_FALLBACK_TAG`].

use crate::addr::{Cidr, PortRange};
use crate::error::RouteError;
use crate::ir::{
    Action, Cond, DefaultRoute, MatchItem, NetworkMask, RejectMethod, ResolveStrategy, Rule,
    RuleSet, SniffedProtocol,
};

use super::{CompileOutput, field_strings};

/// Fallback tag when the config declares no `route.final`.
pub const DEFAULT_SINGBOX_FALLBACK_TAG: &str = "direct";

/// Rule index used for configuration-level (non-rule) warnings and errors.
const CONFIG_WARNING_INDEX: usize = 0;

type Value = serde_json::Value;

fn parse_err(rule_index: usize, field: &'static str, message: impl Into<String>) -> RouteError {
    RouteError::Parse {
        rule_index,
        field,
        message: message.into(),
    }
}

/// Keys understood on a sing-box rule; anything else warns but passes.
const KNOWN_RULE_KEYS: [&str; 18] = [
    "inbound",
    "network",
    "protocol",
    "domain",
    "domain_suffix",
    "domain_keyword",
    "domain_regex",
    "ip_cidr",
    "ip_is_private",
    "source_ip_cidr",
    "source_ip_is_private",
    "source_port",
    "source_port_range",
    "port",
    "port_range",
    "action",
    "outbound",
    "method",
];

/// Keys understood only on a logical (`mode`) rule.
const KNOWN_LOGICAL_KEYS: [&str; 3] = ["mode", "rules", "invert"];

/// Compiles a sing-box `route` JSON document into IR.
///
/// Configuration-level warnings (missing `rules`, unknown config keys) are
/// reported under rule index [`CONFIG_WARNING_INDEX`].
pub fn compile_singbox(json_text: &str) -> Result<CompileOutput, RouteError> {
    let root: Value = serde_json::from_str(json_text)
        .map_err(|e| parse_err(CONFIG_WARNING_INDEX, "root", e.to_string()))?;
    let route = root
        .get("route")
        .ok_or_else(|| parse_err(CONFIG_WARNING_INDEX, "route", "missing route block"))?
        .as_object()
        .ok_or_else(|| parse_err(CONFIG_WARNING_INDEX, "route", "not an object"))?;

    let final_tag = match route.get("final").and_then(Value::as_str) {
        None | Some("") => DEFAULT_SINGBOX_FALLBACK_TAG.to_owned(),
        Some(tag) => tag.to_owned(),
    };
    // A config-level `rule_set` *declaration* is tolerable (referencing
    // rules hit Unsupported at their own positions); record a warning.
    let mut rules: Vec<Rule> = Vec::new();
    let mut warnings: Vec<(usize, String)> = Vec::new();
    if let Some(rs) = route.get("rule_set") {
        warnings.push((
            CONFIG_WARNING_INDEX,
            format!("ignoring rule_set declarations: {rs} (accelerators await T9+)"),
        ));
    }

    if let Some(rules_v) = route.get("rules") {
        let arr = rules_v
            .as_array()
            .ok_or_else(|| parse_err(CONFIG_WARNING_INDEX, "rules", "rules must be an array"))?;
        for (i, rv) in arr.iter().enumerate() {
            compile_rule(rv, i, &mut rules, &mut warnings)?;
        }
    } else {
        warnings.push((
            CONFIG_WARNING_INDEX,
            "route block declares no rules".to_owned(),
        ));
    }

    Ok(CompileOutput {
        ruleset: RuleSet {
            rules,
            default: DefaultRoute::Route { tag: final_tag },
            resolve_strategy: ResolveStrategy::IfNonMatch,
            probes: vec![],
        },
        warnings,
    })
}

/// Compiles one sing-box rule (default or logical shape) into zero or more IR
/// rules appended to `out`.
fn compile_rule(
    rv: &Value,
    i: usize,
    out: &mut Vec<Rule>,
    warnings: &mut Vec<(usize, String)>,
) -> Result<(), RouteError> {
    let obj = rv
        .as_object()
        .ok_or_else(|| parse_err(i, "rule", "rule must be an object"))?;
    check_rule(obj, i, warnings)?;
    let mode = obj.get("mode").and_then(Value::as_str).unwrap_or("");
    let action = parse_action(obj, i)?;

    let arms = if mode.is_empty() {
        let arms = collect_items(obj, i)?;
        if arms[0].is_empty() {
            warnings.push((i, "rule has no matchable condition; skipped".to_owned()));
            return Ok(());
        }
        arms
    } else {
        let arms = logical_arms(obj, i, mode)?;
        if arms.is_empty() {
            warnings.push((
                i,
                format!("logical {mode:?} rule has no conditions; skipped"),
            ));
            return Ok(());
        }
        arms
    };
    for arm in arms {
        out.push(Rule {
            name: None,
            cond: Cond::All(arm),
            action: action.clone(),
        });
    }
    Ok(())
}

/// Checks shared by every rule object, top-level or logical child:
/// unknown-key warnings, deferred-accelerator arms, `invert`, and the
/// logical `mode` value.
fn check_rule(
    obj: &serde_json::Map<String, Value>,
    i: usize,
    warnings: &mut Vec<(usize, String)>,
) -> Result<(), RouteError> {
    for (key, val) in obj {
        let known =
            KNOWN_RULE_KEYS.contains(&key.as_str()) || KNOWN_LOGICAL_KEYS.contains(&key.as_str());
        if !known {
            warnings.push((i, format!("ignored unknown key `{key}`: {val}")));
        }
    }

    // Deferred-accelerator arms fail positionally before any expansion.
    if obj.contains_key("rule_set") {
        return Err(RouteError::Unsupported(
            "rule_set references await accelerator support (T9+); inline fields only for now",
        ));
    }
    if obj.contains_key("dns-router") {
        return Err(RouteError::Unsupported("dns-router action awaits T9+"));
    }
    if obj.get("invert").and_then(Value::as_bool) == Some(true) {
        return Err(RouteError::Unsupported(
            "invert awaits T11 (real Cond::Any/negation evaluation)",
        ));
    }
    let mode = obj.get("mode").and_then(Value::as_str).unwrap_or("");
    if !mode.is_empty() && mode != "and" && mode != "or" {
        return Err(parse_err(
            i,
            "mode",
            format!("unknown logical mode {mode:?}"),
        ));
    }
    Ok(())
}

/// Expands a logical (`mode`) rule's children into flat condition arms:
/// `and` children cartesian-combine (each combination one `Cond::All` set),
/// `or` children alternate. Recurses through nested logical children.
/// Upstream logical children are condition-only rule objects; an `action`
/// key on a child is tolerated but ignored — the parent rule's action
/// applies.
fn logical_arms(
    obj: &serde_json::Map<String, Value>,
    i: usize,
    mode: &str,
) -> Result<Vec<Vec<MatchItem>>, RouteError> {
    let children = obj
        .get("rules")
        .ok_or_else(|| parse_err(i, "mode", "logical rule missing \"rules\" array"))?
        .as_array()
        .ok_or_else(|| parse_err(i, "mode", "\"rules\" must be an array"))?;
    let mut child_arms: Vec<Vec<MatchItem>> = Vec::new();
    for cv in children {
        let cobj = cv
            .as_object()
            .ok_or_else(|| parse_err(i, "rule", "logical child must be an object"))?;
        let cmode = cobj.get("mode").and_then(Value::as_str).unwrap_or("");
        let arms = if cmode.is_empty() {
            // Flat child: exactly one arm (empty arm = no conditions).
            collect_items(cobj, i)?
        } else {
            // Nested logical child: already expanded into arms.
            logical_arms(cobj, i, cmode)?
        };
        child_arms.extend(arms);
    }
    if mode == "and" {
        if child_arms.is_empty() {
            return Ok(Vec::new());
        }
        let mut combos: Vec<Vec<MatchItem>> = vec![Vec::new()];
        for arm in &child_arms {
            let mut next: Vec<Vec<MatchItem>> = Vec::with_capacity(combos.len());
            for prefix in &combos {
                let mut merged = prefix.clone();
                merged.extend(arm.iter().cloned());
                next.push(merged);
            }
            combos = next;
        }
        Ok(combos)
    } else {
        Ok(child_arms)
    }
}

/// Extracts the routing action for a rule object. `outbound` names a
/// destination tag only — it never becomes a match condition.
fn parse_action(obj: &serde_json::Map<String, Value>, i: usize) -> Result<Action, RouteError> {
    let action_v = obj
        .get("action")
        .ok_or_else(|| parse_err(i, "action", "missing action key"))?;
    let action_str = action_v
        .as_str()
        .ok_or_else(|| parse_err(i, "action", "action must be a string"))?;

    match action_str {
        "route" => {
            let tag = obj
                .get("outbound")
                .and_then(Value::as_str)
                .filter(|t| !t.is_empty())
                .ok_or_else(|| parse_err(i, "outbound", "route action requires \"outbound\""))?;
            let tag = if tag == "bypass" || tag == "direct" {
                "direct".to_owned()
            } else {
                tag.to_owned()
            };
            Ok(Action::Route {
                tag,
                override_addr: None,
            })
        }
        "hijack-dns" => Ok(Action::HijackDns),
        "reject" => {
            let method = match obj.get("method").and_then(Value::as_str) {
                None | Some("" | "default" | "reply") => RejectMethod::DefaultReply,
                Some("drop") => RejectMethod::Drop,
                Some(other) => {
                    return Err(parse_err(
                        i,
                        "method",
                        format!("unknown reject method {other:?}"),
                    ));
                }
            };
            Ok(Action::Reject { method })
        }
        "direct" | "bypass" => Ok(Action::Route {
            tag: "direct".to_owned(),
            override_addr: None,
        }),
        other => Err(RouteError::Unsupported(
            // Static string: name known actions for the reader.
            other_action_hint(other),
        )),
    }
}

/// Builds the Unsupported message for actions outside the supported set.
fn other_action_hint(other: &str) -> &'static str {
    match other {
        "sniff" => "sniff action is not a routing decision; unsupported",
        "resolve" => "resolve action is not a routing decision; unsupported",
        _ => "rule action outside route/hijack-dns/reject/direct/bypass",
    }
}

/// Collects the flat condition arms from a rule object: one arm per
/// multi-value sniffed-protocol alternative (T5's distributive ruling),
/// one shared arm otherwise. An arm of length 0 means "no matchable
/// condition"; the caller decides skip-vs-error.
fn collect_items(
    obj: &serde_json::Map<String, Value>,
    i: usize,
) -> Result<Vec<Vec<MatchItem>>, RouteError> {
    let mut domains = DomainBucket::default();
    let mut ip = IpBucket::default();
    let mut source_ip = IpBucket::default();
    let mut ports: Vec<PortRange> = Vec::new();
    let mut source_ports: Vec<PortRange> = Vec::new();
    let mut network = NetworkMask {
        tcp: false,
        udp: false,
    };
    let mut inbound_tags: Vec<String> = Vec::new();
    let mut protocols = Vec::<SniffedProtocol>::new();

    for (field, v) in obj {
        match field.as_str() {
            "domain" => {
                for tok in field_strings(v, i, "domain")? {
                    // Upstream sing matches a bare `domain` entry as itself
                    // plus all subdomains; register both exact and suffix.
                    domains.exact.push(tok.clone());
                    domains.suffix.push(tok);
                }
            }
            "domain_suffix" => domains.suffix.extend(field_strings(v, i, "domain_suffix")?),
            "domain_keyword" => domains
                .keywords
                .extend(field_strings(v, i, "domain_keyword")?),
            "domain_regex" => domains.regexes.extend(field_strings(v, i, "domain_regex")?),
            "ip_cidr" => collect_ip(v, i, "ip_cidr", &mut ip)?,
            "ip_is_private" => {
                if v.as_bool().unwrap_or(false) {
                    ip.private = true;
                }
            }
            "source_ip_cidr" => collect_ip(v, i, "source_ip_cidr", &mut source_ip)?,
            "source_ip_is_private" => {
                if v.as_bool().unwrap_or(false) {
                    source_ip.private = true;
                }
            }
            "port" | "port_range" => {
                ports.extend(parse_ports(
                    v,
                    i,
                    if field == "port" {
                        "port"
                    } else {
                        "port_range"
                    },
                )?);
            }
            "source_port" | "source_port_range" => {
                source_ports.extend(parse_ports(
                    v,
                    i,
                    if field == "source_port" {
                        "source_port"
                    } else {
                        "source_port_range"
                    },
                )?);
            }
            "network" => network = parse_network(v, i)?,
            "inbound" => inbound_tags = field_strings(v, i, "inbound")?,
            "protocol" => collect_protocols(v, i, &mut protocols)?,
            // action/outbound/method keys are action-side; logical-only
            // keys never reach here (mode branched earlier).
            _ => {}
        }
    }

    let mut items: Vec<MatchItem> = Vec::new();
    if !domains.is_empty() {
        items.push(MatchItem::Domain {
            exact: domains.exact,
            suffix: domains.suffix,
            keywords: domains.keywords,
            regexes: domains.regexes,
        });
    }
    if !ip.is_empty() {
        items.push(MatchItem::IpCidr {
            cidrs: ip.cidrs,
            private: ip.private,
            geo_country: Vec::new(),
        });
    }
    if !source_ip.is_empty() {
        items.push(MatchItem::SourceIpCidr {
            cidrs: source_ip.cidrs,
            private: source_ip.private,
            geo_country: Vec::new(),
        });
    }
    if !ports.is_empty() {
        items.push(MatchItem::Ports(ports));
    }
    if !source_ports.is_empty() {
        items.push(MatchItem::SourcePorts(source_ports));
    }
    if network.tcp || network.udp {
        items.push(MatchItem::Network(network));
    }
    if !inbound_tags.is_empty() {
        items.push(MatchItem::InboundTag { tags: inbound_tags });
    }
    // A multi-value sniffed-protocol whitelist is an OR over sniff results
    // while every other item is an AND (T5's distributive ruling): one arm
    // per protocol value sharing the other AND items; single/none stays one
    // arm.
    Ok(match protocols.len() {
        0 => vec![items],
        1 => {
            items.push(MatchItem::Protocol(protocols[0]));
            vec![items]
        }
        _ => {
            let mut arms: Vec<Vec<MatchItem>> = Vec::with_capacity(protocols.len());
            for p in protocols {
                let mut sub = items.clone();
                sub.push(MatchItem::Protocol(p));
                arms.push(sub);
            }
            arms
        }
    })
}

/// Parses sing-box port tokens. `port` entries are plain `u16` numbers
/// (upstream `Listable[uint16]`; a bare string also tolerated), range
/// fields use colon syntax per upstream docs (`1000:2000`, `:3000`,
/// `4000:`), normalized to dash-ranges in IR.
fn parse_ports(v: &Value, i: usize, field: &'static str) -> Result<Vec<PortRange>, RouteError> {
    let mut ranges = Vec::new();
    for tok in port_strings(v, i, field)? {
        let (start, end) = if field == "port" {
            let p: u16 = tok
                .parse()
                .map_err(|_| parse_err(i, field, format!("invalid port `{tok}`")))?;
            (p, p)
        } else {
            parse_range_token(&tok, i, field)?
        };
        ranges.push(PortRange { start, end });
    }
    Ok(ranges)
}

/// Port fields: string-or-number, or arrays thereof (upstream listable
/// uint16). Numbers stringify; anything else is a positional Parse error.
fn port_strings(v: &Value, i: usize, field: &'static str) -> Result<Vec<String>, RouteError> {
    match v {
        Value::Number(n) => Ok(vec![n.to_string()]),
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for it in items {
                out.extend(port_strings(it, i, field)?);
            }
            Ok(out)
        }
        _ => field_strings(v, i, field),
    }
}

/// Parses one colon-range token (`1000:2000`, `:3000`, `4000:`) into an
/// inclusive range, clamping open ends to the u16 bounds.
fn parse_range_token(tok: &str, i: usize, field: &'static str) -> Result<(u16, u16), RouteError> {
    let (a, b) = tok
        .split_once(':')
        .ok_or_else(|| parse_err(i, field, format!("invalid port range `{tok}`")))?;
    let start: u16 = if a.is_empty() {
        0
    } else {
        a.parse()
            .map_err(|_| parse_err(i, field, format!("invalid port range `{tok}`")))?
    };
    let end: u16 = if b.is_empty() {
        u16::MAX
    } else {
        b.parse()
            .map_err(|_| parse_err(i, field, format!("invalid port range `{tok}`")))?
    };
    Ok((start, end))
}

/// Collects CIDR entries; bare addresses gain `/32` (or `/128` for IPv6).
fn collect_ip(
    v: &Value,
    i: usize,
    field: &'static str,
    out: &mut IpBucket,
) -> Result<(), RouteError> {
    for tok in field_strings(v, i, field)? {
        let spec: String = if tok.contains('/') {
            tok
        } else {
            let bits = if tok.contains(':') { "/128" } else { "/32" };
            format!("{tok}{bits}")
        };
        let cidr = Cidr::parse(&spec).map_err(|e| reindex(e, i, field))?;
        out.cidrs.push(cidr);
    }
    Ok(())
}

/// Parses `network` tokens into a transport mask.
fn parse_network(v: &Value, i: usize) -> Result<NetworkMask, RouteError> {
    let mut mask = NetworkMask {
        tcp: false,
        udp: false,
    };
    for tok in field_strings(v, i, "network")?.join(",").split(',') {
        let tok = tok.trim();
        if tok.is_empty() {
            continue;
        }
        if tok.eq_ignore_ascii_case("tcp") && !mask.tcp {
            mask.tcp = true;
        } else if tok.eq_ignore_ascii_case("udp") && !mask.udp {
            mask.udp = true;
        } else {
            return Err(parse_err(
                i,
                "network",
                format!("unknown network token {tok:?}"),
            ));
        }
    }
    Ok(mask)
}

/// Maps sniffed-protocol tokens onto the IR whitelist (http/tls/dns only).
fn collect_protocols(
    v: &Value,
    i: usize,
    out: &mut Vec<SniffedProtocol>,
) -> Result<(), RouteError> {
    for tok in field_strings(v, i, "protocol")? {
        let p = if tok.eq_ignore_ascii_case("http") {
            SniffedProtocol::Http
        } else if tok.eq_ignore_ascii_case("tls") {
            SniffedProtocol::Tls
        } else if tok.eq_ignore_ascii_case("dns") {
            SniffedProtocol::Dns
        } else {
            return Err(RouteError::Unsupported(
                "sniffed protocol outside http/tls/dns",
            ));
        };
        out.push(p);
    }
    Ok(())
}

#[derive(Default)]
struct DomainBucket {
    exact: Vec<String>,
    suffix: Vec<String>,
    keywords: Vec<String>,
    regexes: Vec<String>,
}

impl DomainBucket {
    const fn is_empty(&self) -> bool {
        self.exact.is_empty()
            && self.suffix.is_empty()
            && self.keywords.is_empty()
            && self.regexes.is_empty()
    }
}

#[derive(Default)]
struct IpBucket {
    cidrs: Vec<Cidr>,
    private: bool,
}

impl IpBucket {
    const fn is_empty(&self) -> bool {
        self.cidrs.is_empty() && !self.private
    }
}

/// Re-attributes a Cidr parse error to the owning rule and field.
const fn reindex(mut e: RouteError, rule_index: usize, field: &'static str) -> RouteError {
    if let RouteError::Parse {
        rule_index: idx,
        field: f,
        ..
    } = &mut e
    {
        *idx = rule_index;
        *f = field;
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decision, Engine, NetAddr, NetHost};

    const FIXTURE: &str = include_str!("../../tests/fixtures/singbox_sample.json");

    fn err_at(txt: &str, idx: usize) -> RouteError {
        match compile_singbox(txt) {
            Err(e @ RouteError::Parse { rule_index, .. }) if rule_index == idx => e,
            other => panic!("expected Parse at {idx}, got {other:?}"),
        }
    }

    /// The brief's sample (upstream-valid shape): config-level `rule_set`
    /// declaration tolerated with a warning; a rule *referencing* `rule_set`
    /// is Unsupported.
    const BRIEF_SAMPLE: &str = r#"{
  "route": { "final": "proxy-main",
    "rules": [
      { "domain_suffix": [".google.com"], "action": "hijack-dns" },
      { "mode": "and", "rules": [
          { "ip_cidr": ["10.0.0.0/8"] }, { "inbound": ["tun-in"] } ],
        "action": "route", "outbound": "local-bypass" } ],
    "rule_set": [{ "type": "local", "tag": "geo" }] }
}"#;

    /// A rule *referencing* `rule_set` is Unsupported (T9+ accelerator); a
    /// config-level declaration alone is a warning. Both asserted here.
    #[test]
    fn brief_sample_rule_set_ref_is_unsupported() {
        match compile_singbox(BRIEF_SAMPLE) {
            Ok(out) => {
                assert!(
                    out.warnings.iter().any(|(_, m)| m.contains("rule_set")),
                    "config-level declaration warns: {:?}",
                    out.warnings
                );
                assert!(!out.ruleset.rules.is_empty());
            }
            other => panic!("expected tolerated declaration, got {other:?}"),
        }
        let with_ref = r#"{"route":{"rules":[
            {"rule_set":["geo"],"action":"route","outbound":"o"}]}}"#;
        match compile_singbox(with_ref) {
            Err(RouteError::Unsupported(msg)) => {
                assert!(msg.contains("rule_set"), "msg: {msg}");
            }
            other => panic!("expected Unsupported for rule_set ref, got {other:?}"),
        }
    }
    #[test]
    fn fixture_parses_to_golden_ir() {
        let out = compile_singbox(FIXTURE).unwrap();
        // 4 source rules: hijack + and-flatten + reject + 3-way protocol
        // expansion = 6 flat rules. The config-level rule_set *declaration*
        // is tolerated as a warning (referencing rules would be Unsupported).
        assert_eq!(out.ruleset.rules.len(), 6, "one flat rule per protocol");
        assert_eq!(out.warnings.len(), 1, "rule_set declaration warning");
        assert!(out.warnings[0].1.contains("rule_set"));
        assert!(
            matches!(&out.ruleset.default, DefaultRoute::Route { tag } if tag == "proxy-main"),
            "final: proxy-main maps to RuleSet.default"
        );
        assert_eq!(out.ruleset.resolve_strategy, ResolveStrategy::IfNonMatch);
        assert_eq!(out.ruleset.probes.len(), 0);

        // Rule 0: domain_suffix .google.com → HijackDns.
        let r0 = &out.ruleset.rules[0];
        let Cond::All(items) = &r0.cond else {
            panic!("rule 0 cond");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            MatchItem::Domain {
                exact: vec![],
                suffix: vec![".google.com".into()],
                keywords: vec![],
                regexes: vec![],
            }
        );
        assert_eq!(r0.action, Action::HijackDns);

        // Rule 1: ip_cidr + inbound tun-in "and" flattens to one flat rule
        // Cond::All with IpCidr + InboundTag items (per dispatch expectation).
        let r1 = &out.ruleset.rules[1];
        let Cond::All(items) = &r1.cond else {
            panic!("rule 1 cond");
        };
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0],
            MatchItem::IpCidr {
                cidrs: vec![Cidr::parse("10.0.0.0/8").unwrap()],
                private: false,
                geo_country: vec![],
            }
        );
        assert_eq!(
            items[1],
            MatchItem::InboundTag {
                tags: vec!["tun-in".into()]
            }
        );
        assert_eq!(
            r1.action,
            Action::Route {
                tag: "local-bypass".into(),
                override_addr: None
            }
        );
        // Rule 2: port_range colon syntax → dash ranges; "8080:" opens to
        // u16::MAX; reject method drop maps to RejectMethod::Drop.
        let r2 = &out.ruleset.rules[2];
        let Cond::All(items) = &r2.cond else {
            panic!("rule 2 cond");
        };
        assert_eq!(
            items[0],
            MatchItem::Ports(vec![
                PortRange {
                    start: 1000,
                    end: 2000
                },
                PortRange {
                    start: 8080,
                    end: u16::MAX
                },
            ])
        );
        assert_eq!(
            r2.action,
            Action::Reject {
                method: RejectMethod::Drop
            }
        );

        // Rule 3: protocol [http, tls, dns] + bypass action expands
        // distributively to 3 flat rules sharing Route { tag: "direct" }.
        let shared = Action::Route {
            tag: "direct".into(),
            override_addr: None,
        };
        for (rule, proto) in out.ruleset.rules[3..].iter().zip([
            SniffedProtocol::Http,
            SniffedProtocol::Tls,
            SniffedProtocol::Dns,
        ]) {
            let Cond::All(items) = &rule.cond else {
                panic!("expanded rule cond must stay All");
            };
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], MatchItem::Protocol(proto));
            assert_eq!(rule.action, shared);
        }
    }

    #[test]
    fn logical_or_expands_distributively_sharing_action() {
        // OR-mode logical rule: two child conditions expand to two flat rules
        // sharing the action. T11 note: once the Engine evaluates Cond::Any,
        // this expansion is replaced by genuine `Cond::Any` emission — the
        // expansion is purely an engine-compatibility encoding of the same
        // alternation.
        let txt = r#"{"route":{"rules":[{
            "mode":"or",
            "rules":[
                {"domain_suffix":[".a.com"],"action":"route","outbound":"o1"},
                {"ip_cidr":["192.168.0.0/16"],"action":"route","outbound":"o1"}
            ],
            "action":"route","outbound":"proxy-x"}]}}"#;
        let out = compile_singbox(txt).unwrap();
        assert_eq!(out.ruleset.rules.len(), 2);
        for r in &out.ruleset.rules {
            assert_eq!(
                r.action,
                Action::Route {
                    tag: "proxy-x".into(),
                    override_addr: None
                }
            );
        }
        let Cond::All(items0) = &out.ruleset.rules[0].cond else {
            panic!()
        };
        assert_eq!(
            items0[0],
            MatchItem::Domain {
                exact: vec![],
                suffix: vec![".a.com".into()],
                keywords: vec![],
                regexes: vec![]
            }
        );
        let Cond::All(items1) = &out.ruleset.rules[1].cond else {
            panic!()
        };
        assert_eq!(
            items1[0],
            MatchItem::IpCidr {
                cidrs: vec![Cidr::parse("192.168.0.0/16").unwrap()],
                private: false,
                geo_country: vec![]
            }
        );
    }

    #[test]
    fn missing_action_is_parse_error() {
        // Dispatch ruling: action missing ⇒ Parse error.
        err_at(r#"{"route":{"rules":[{"domain_suffix":[".x.com"]}]}}"#, 0);
    }

    #[test]
    fn route_action_without_outbound_is_parse_error() {
        err_at(r#"{"route":{"rules":[{"action":"route"}]}}"#, 0);
    }

    #[test]
    fn bypass_and_direct_collapse_to_direct_tag() {
        // Action-only rule has no condition: skipped with a warning (same
        // empty-rule handling as every other action).
        for action in ["bypass", "direct"] {
            let txt = format!(r#"{{"route":{{"rules":[{{"action":"{action}"}}]}}}}"#);
            let out = compile_singbox(&txt).unwrap();
            assert_eq!(out.ruleset.rules.len(), 0, "{action}: empty rule skips");
            assert_eq!(out.warnings.len(), 1, "{action}: skip warned");
        }
        // With a real condition, both map to Route { tag: "direct" }.
        for action in ["bypass", "direct"] {
            let txt = format!(
                r#"{{"route":{{"rules":[{{"domain_suffix":[".x.com"],"action":"{action}"}}]}}}}"#
            );
            let out = compile_singbox(&txt).unwrap();
            assert_eq!(out.ruleset.rules.len(), 1);
            assert_eq!(
                out.ruleset.rules[0].action,
                Action::Route {
                    tag: "direct".into(),
                    override_addr: None
                }
            );
        }
    }

    #[test]
    fn route_to_bypass_or_direct_outbound_collapses() {
        let txt = r#"{"route":{"rules":[{"domain_suffix":[".x.com"],"action":"route","outbound":"bypass"}]}}"#;
        let out = compile_singbox(txt).unwrap();
        assert_eq!(
            out.ruleset.rules[0].action,
            Action::Route {
                tag: "direct".into(),
                override_addr: None
            }
        );
    }

    #[test]
    fn unknown_action_is_unsupported() {
        let txt = r#"{"route":{"rules":[{"domain_suffix":[".x.com"],"action":"sniff"}]}}"#;
        assert!(matches!(
            compile_singbox(txt),
            Err(RouteError::Unsupported(_))
        ));
    }

    #[test]
    fn mode_on_flat_rule_is_parse_error() {
        // Ruling: `mode` without `rules` is a correct positional Parse
        // error — upstream logical rules always carry a `rules` array.
        let e = err_at(
            r#"{"route":{"rules":[{"ip_cidr":["10.0.0.0/8"],"mode":"and","action":"route","outbound":"o"}]}}"#,
            0,
        );
        assert!(
            e.to_string().contains("rules"),
            "error names the missing rules array: {e}"
        );
    }

    #[test]
    fn invert_is_unsupported_awaiting_t11() {
        let txt = r#"{"route":{"rules":[{"domain_suffix":[".x.com"],"invert":true,"action":"route","outbound":"o"}]}}"#;
        match compile_singbox(txt) {
            Err(RouteError::Unsupported(msg)) => assert!(msg.contains("T11"), "{msg}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn dns_router_action_is_unsupported() {
        let txt = r#"{"route":{"rules":[{"domain_suffix":[".x.com"],"action":"hijack-dns","dns-router":"dns-out"}]}}"#;
        match compile_singbox(txt) {
            Err(RouteError::Unsupported(msg)) => assert!(msg.contains("dns-router"), "{msg}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn logical_and_flattens_children_into_single_all() {
        let txt = r#"{"route":{"rules":[{
            "mode":"and",
            "rules":[
                {"domain_suffix":[".a.com"],"action":"route","outbound":"o"},
                {"port":[443],"action":"route","outbound":"o"}
            ],
            "action":"route","outbound":"and-out"}]}}"#;
        let out = compile_singbox(txt).unwrap();
        assert_eq!(out.ruleset.rules.len(), 1);
        let Cond::All(items) = &out.ruleset.rules[0].cond else {
            panic!()
        };
        assert_eq!(items.len(), 2);
        assert_eq!(
            out.ruleset.rules[0].action,
            Action::Route {
                tag: "and-out".into(),
                override_addr: None
            }
        );
    }

    #[test]
    fn reject_method_variants_map() {
        for (method, expected) in [
            ("drop", RejectMethod::Drop),
            ("reply", RejectMethod::DefaultReply),
            ("default", RejectMethod::DefaultReply),
            ("", RejectMethod::DefaultReply),
        ] {
            let txt = format!(
                r#"{{"route":{{"rules":[{{"port":[53],"action":"reject","method":"{method}"}}
                ]}}}}"#
            );
            // method:"" serializes as missing in some producers; the empty
            // string case exercises the None arm too.
            let txt = txt.replace("\"method\":\"\"", "\"method\":null");
            let out = compile_singbox(&txt).unwrap();
            assert_eq!(
                out.ruleset.rules[0].action,
                Action::Reject { method: expected }
            );
        }
    }

    #[test]
    fn port_range_open_and_closed_forms() {
        let txt = r#"{"route":{"rules":[
            {"port_range":[":3000","4000:"],"action":"route","outbound":"o"},
            {"port":[80,443],"action":"route","outbound":"o"}]}}"#;
        let out = compile_singbox(txt).unwrap();
        let Cond::All(items0) = &out.ruleset.rules[0].cond else {
            panic!()
        };
        assert_eq!(
            items0[0],
            MatchItem::Ports(vec![
                PortRange {
                    start: 0,
                    end: 3000
                },
                PortRange {
                    start: 4000,
                    end: u16::MAX
                },
            ])
        );
        let Cond::All(items1) = &out.ruleset.rules[1].cond else {
            panic!()
        };
        assert_eq!(
            items1[0],
            MatchItem::Ports(vec![
                PortRange { start: 80, end: 80 },
                PortRange {
                    start: 443,
                    end: 443
                },
            ])
        );
    }

    #[test]
    fn bare_ip_cidr_gains_host_prefix() {
        let txt = r#"{"route":{"rules":[{"ip_cidr":["192.168.0.1","2001:db8::1"],"action":"route","outbound":"o"}]}}"#;
        let out = compile_singbox(txt).unwrap();
        let Cond::All(items) = &out.ruleset.rules[0].cond else {
            panic!()
        };
        assert_eq!(
            items[0],
            MatchItem::IpCidr {
                cidrs: vec![
                    Cidr::parse("192.168.0.1/32").unwrap(),
                    Cidr::parse("2001:db8::1/128").unwrap(),
                ],
                private: false,
                geo_country: vec![]
            }
        );
    }

    #[test]
    fn source_fields_and_ip_is_private_map() {
        let txt = r#"{"route":{"rules":[
            {"source_ip_cidr":["10.0.0.0/8"],"source_ip_is_private":true,
             "ip_is_private":true,"source_port_range":["53:64"],
             "action":"route","outbound":"o"}]}}"#;
        let out = compile_singbox(txt).unwrap();
        let Cond::All(items) = &out.ruleset.rules[0].cond else {
            panic!()
        };
        assert!(matches!(&items[0], MatchItem::IpCidr { private: true, .. }));
        assert!(matches!(
            &items[1],
            MatchItem::SourceIpCidr { private: true, .. }
        ));
        assert!(matches!(&items[2], MatchItem::SourcePorts(_)));
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn domain_bare_entry_pairs_exact_and_suffix() {
        let txt =
            r#"{"route":{"rules":[{"domain":["test.com"],"action":"route","outbound":"o"}]}}"#;
        let out = compile_singbox(txt).unwrap();
        let Cond::All(items) = &out.ruleset.rules[0].cond else {
            panic!()
        };
        assert_eq!(
            items[0],
            MatchItem::Domain {
                exact: vec!["test.com".into()],
                suffix: vec!["test.com".into()],
                keywords: vec![],
                regexes: vec![]
            }
        );
    }

    #[test]
    fn network_tokens_map_to_mask() {
        let txt =
            r#"{"route":{"rules":[{"network":["tcp","udp"],"action":"route","outbound":"o"}]}}"#;
        let out = compile_singbox(txt).unwrap();
        let Cond::All(items) = &out.ruleset.rules[0].cond else {
            panic!()
        };
        assert_eq!(
            items[0],
            MatchItem::Network(NetworkMask {
                tcp: true,
                udp: true
            })
        );
    }

    #[test]
    fn unknown_sniff_protocol_is_unsupported() {
        let txt = r#"{"route":{"rules":[{"protocol":["quic"],"action":"route","outbound":"o"}]}}"#;
        assert!(matches!(
            compile_singbox(txt),
            Err(RouteError::Unsupported(_))
        ));
    }

    #[test]
    fn unknown_rule_key_is_tolerated_with_warning() {
        let txt = r#"{"route":{"rules":[{"domain_suffix":[".x.com"],"action":"route","outbound":"o","clash_mode":"direct"}]}}"#;
        let out = compile_singbox(txt).unwrap();
        assert_eq!(out.warnings.len(), 1);
        assert!(out.warnings[0].1.contains("clash_mode"));
    }

    #[test]
    fn empty_rule_is_skipped_with_warning() {
        let txt = r#"{"route":{"rules":[
            {"action":"route","outbound":"a"},
            {"domain_suffix":[".k.com"],"action":"route","outbound":"b"}]}}"#;
        let out = compile_singbox(txt).unwrap();
        assert_eq!(out.ruleset.rules.len(), 1);
        assert_eq!(out.warnings.len(), 1);
        assert_eq!(out.warnings[0].0, 0);
    }

    #[test]
    fn missing_route_block_is_root_parse_error() {
        err_at("{}", 0);
    }

    #[test]
    fn default_final_is_direct_constant() {
        let out = compile_singbox(r#"{"route":{"rules":[]}}"#).unwrap();
        assert!(
            matches!(&out.ruleset.default, DefaultRoute::Route { tag } if tag == DEFAULT_SINGBOX_FALLBACK_TAG)
        );
    }

    /// Engine smoke: compiled ruleset builds + decides per first-match.
    #[test]
    fn compiled_fixture_builds_engine_and_decides() {
        let out = compile_singbox(FIXTURE).unwrap();
        let engine = Engine::build(out.ruleset).unwrap();
        let meta = |host: &str, port: u16, network: NetworkMask| crate::ConnMeta {
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
        };
        // Port range 1000:2000 reject drop.
        assert_eq!(
            engine.decide(&mut meta("any.host", 1500, NetworkMask::TCP)),
            Decision::Reject {
                method: RejectMethod::Drop
            }
        );
        // Domain suffix hijack.
        assert_eq!(
            engine.decide(&mut meta("www.google.com", 443, NetworkMask::TCP)),
            Decision::HijackDns
        );
        // No match → final proxy-main. Port 700 avoids every fixture range
        // (1000:2000 and the open-ended 8080:).
        assert!(matches!(
            engine.decide(&mut meta("other.net", 700, NetworkMask::TCP)),
            Decision::Route { tag, .. } if tag == "proxy-main"
        ));
    }
}
