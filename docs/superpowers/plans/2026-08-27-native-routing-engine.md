# Native Routing Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Standalone first-match routing engine (`xray-tui-route`) with xray/sing-box/DB compile front-ends, lazy DNS resolution with network-breakdown probes, TLS/HTTP sniffing support, and Actions Log surfacing.

**Architecture:** sing-box action-pipeline semantics (ordered rules, terminal decisions, default fallback) over composable matcher items; Xray-compat front-end compiles existing dialects into one native IR. Engine is I/O-free (`async fn decide_async(&Engine, &mut ConnMeta)`); callers own dialing/sniff-byte acquisition.

**Tech Stack:** Rust 2024, tokio, serde/serde_json, thiserror, aho-corasick 1.1 (workspace dep), jiff timestamps, mpsc event stream.

**Spec:** `docs/superpowers/specs/2026-08-27-native-routing-design.md`

## Global Constraints

- Crate name `xray-tui-route`, created under `crates/` — workspace member registration is automatic via `members = ["crates/*"]`.
- Style: Rust 2024 edition; clippy pedantic+nursery clean (`[lints] workspace = true` opt-in per crate manifest); `cargo fmt`; thiserror for errors; unit tests next to code in same file; integration tests under `crates/<c>/tests/*.rs`; semver-pinned minor bounds on direct deps.
- Feature-gated deps: `dns` feature pulls `xray-tui-dns`; `geoip` feature pulls `xray-tui-geoip`. Default features include neither.
- No raw SQL outside the toasty model layer; DB schema changes bump `PRAGMA user_version` tag (currently 5 → 6, Task 14).
- Explicit absence: every deferred upstream capability returns `RouteError::Unsupported`, never silent behavior.
- All rule evaluation is linear first-match with default fallback (upstream-parity semantics, spec §5).
- Timestamps are `jiff::Timestamp` everywhere public.
- Deferred (do NOT implement in this plan): LRU decision cache, AC-trie swap-in behind benches, radix CidrSet — these wait for a criterion baseline showing need (spec R7).

---

### Task 1: Crate scaffold — addr + error

**Files:**
- Create: `crates/xray-tui-route/Cargo.toml`
- Create: `crates/xray-tui-route/src/lib.rs`
- Create: `crates/xray-tui-route/src/addr.rs`
- Create: `crates/xray-tui-route/src/error.rs`
- Test: unit tests in `addr.rs`

**Interfaces:**
- Produces: `NetAddr { host: NetHost, port: u16 }`, `NetHost { Ip(IpAddr), Domain(String) }` (with `new(host:&str)` IP-or-domain inference, `as_str()`, `PartialEq/Eq/Clone/Debug/Hash`), `Cidr { addr: IpAddr, bits: u8 }` with `contains(&IpAddr)`, `parse("10.0.0.0/8") -> Result<Cidr, RouteError>`; `PortRange { start: u16, end: u16 }` inclusive with `contains(u16)`; `RouteError` (thiserror): `Parse { rule_index: usize, field: &'static str, message: String }`, `Unsupported(&'static str)`, `Resolve(String)`.

- [ ] **Step 1: Scaffold Cargo.toml**

```toml
[package]
name = "xray-tui-route"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
aho-corasick = "1.1"
jiff = "0.2"
regex = "1"

[features]
default = []
dns = ["dep:xray-tui-dns"]
geoip = ["dep:xray-tui-geoip"]

[dependencies.xray-tui-dns]
version = "0.1"
path = "../xray-tui-dns"
optional = true

[dependencies.xray-tui-geoip]
version = "0.1"
path = "../xray-tui-geoip"
optional = true

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread", "sync"] }

[lints]
workspace = true
```

(Copy actual version pins from sibling crate manifests before writing.)

- [ ] **Step 2: Write failing tests**

```rust
// addr.rs #[cfg(test)]
#[test]
fn host_infers_ip_vs_domain() {
    assert_eq!(NetHost::new("1.2.3.4"), NetHost::Ip([1, 2, 3, 4].into()));
    assert_eq!(NetHost::new("example.com"), NetHost::Domain("example.com".into()));
}
#[test]
fn cidr_contains_boundary() {
    let c = Cidr::parse("10.0.0.0/8").unwrap();
    assert!(c.contains(&"10.255.1.1".parse().unwrap()));
    assert!(!c.contains(&"11.0.0.1".parse().unwrap()));
}
#[test]
fn cidr_rejects_bad_input() {
    assert!(matches!(Cidr::parse("300.1.1.1/8"), Err(RouteError::Parse { .. })));
}
#[test]
fn port_range_inclusive() {
    let r = PortRange { start: 1000, end: 2000 };
    assert!(r.contains(1000) && r.contains(2000) && !r.contains(999));
}
```

- [ ] **Step 3: Run tests, expect compile failure** — `cargo test -p xray-tui-route --lib`
- [ ] **Step 4: Implement `addr.rs` + `error.rs`** (minimal versions of everything in Interfaces)
- [ ] **Step 5: Tests green, commit**

```bash
git add crates/xray-tui-route
git commit -m "feat(route): scaffold xray-tui-route crate with addr/error primitives"
```

---

### Task 2: Typed IR + serde

**Files:**
- Create: `crates/xray-tui-route/src/ir.rs`
- Modify: `src/lib.rs` (add module declarations)

**Interfaces:**
- Consumes: Task 1 types.
- Produces (all `serde::{Serialize,Deserialize}`):

```rust
pub struct RuleSet {
    pub rules: Vec<Rule>,
    pub default: DefaultRoute,
    pub resolve_strategy: ResolveStrategy,
    pub probes: Vec<String>,   // must-resolve probe hostnames (spec §6 probes; user decision 4)
}
pub struct Rule { pub name: Option<String>, pub cond: Cond, pub action: Action }
#[serde(rename_all = "snake_case")]
pub enum Cond { All(Vec<MatchItem>), Any(Vec<Cond>), Invert(Box<Cond>) }
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MatchItem {
    Domain { exact: Vec<String>, suffix: Vec<String>, keywords: Vec<String>, regexes: Vec<String> },
    IpCidr { cidrs: Vec<Cidr>, private: bool, geo_country: Vec<String> },
    SourceIpCidr { cidrs: Vec<Cidr>, private: bool, geo_country: Vec<String> }, // payload mirrors IpCidr
    Ports(Vec<PortRange>),
    SourcePorts(Vec<PortRange>),
    Network(NetworkMask),
    Protocol(SniffedProtocol),          // whitelist Http|Tls|Dns only
    InboundTag { tags: Vec<String> },
    OutboundTag { tags: Vec<String> },
}
#[serde(rename_all = "snake_case")]
pub enum Action { Route { tag: String, override_addr: Option<NetAddr> },
                  Reject { method: RejectMethod }, HijackDns }
#[derive(Default)]
#[serde(rename_all = "snake_case")]
pub enum RejectMethod { Drop, DefaultReply }
#[serde(rename_all = "snake_case")]
pub enum DefaultRoute { Route { tag: String }, Reject { method: RejectMethod } }
#[serde(rename_all = "snake_case")]
pub enum ResolveStrategy { AsIs, IfNonMatch }
#[derive(Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SniffedProtocol { Http, Tls, Dns }
#[derive(Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct NetworkMask { pub tcp: bool, pub udp: bool }
impl NetworkMask {
    pub const TCP: Self = Self { tcp: true, udp: false };
    pub const UDP: Self = Self { tcp: false, udp: true };
    /// bit-subset check used by Network items against ConnMeta.network
    pub fn contains(self, other: NetworkMask) -> bool;
}
```

- [ ] **Step 1: Failing serde roundtrip test** — build a `RuleSet` exercising every variant arm; `serde_json::to_string` → back → `assert_eq!`; plus `NetworkMask::contains` truth rows.
- [ ] **Step 2: Expect failure, implement `ir.rs`, rerun until pass**
- [ ] **Step 3: Commit** `feat(route): typed rule IR with serde`

---

### Task 3: Compiled matchers — domains + CIDR sets

**Files:**
- Create: `crates/xray-tui-route/src/matchers.rs`

**Interfaces:**
- Consumes: Tasks 1–2 types.
- Produces:

```rust
pub struct DomainRulesSpec { pub exact: Vec<String>, pub suffix: Vec<String>,
                             pub keywords: Vec<String>, pub regexes: Vec<String> }
pub fn empty_spec() -> DomainRulesSpec;

pub struct CompiledDomain { /* exact HashSet lowercase, suffix HashSet leading-dot lowercase,
                             keywords AhoCorasick<u32>, RegexSet */ }
impl CompiledDomain {
    pub fn build(spec: &DomainRulesSpec) -> Result<Self, RouteError>; // invalid regex => Err(Parse{..})
    /// exact hit | domain ends_with ".suffix-entry" | keyword substring | regex hit
    pub fn matches_domain(&self, host: &str) -> bool; // input lowercased inside
    pub fn is_empty(&self) -> bool;
}

/// Correctness-first prefix storage; radix/prefix trie explicitly deferred (Global Constraints).
pub struct CidrSet;
pub struct CidrSetBuilder { v4: Vec<(Ipv4Addr /*masked*/, u8 /*bits*/)>, v6: Vec<(Ipv6Addr, u8)> };
impl CidrSetBuilder {
    pub fn insert(&mut self, c: Cidr);
    pub fn build(self) -> CidrSet;
}
impl CidrSet {
    /// linear scan over stored prefixes matching ip's family + bits compare
    pub fn contains(&self, ip: IpAddr) -> bool;
    pub fn is_empty(&self) -> bool;
    /// RFC1918 10/8,172.16/12,192.168/16 + CGNAT 100.64/10 + loopback + link-local + ULA fc00::/7
    pub fn private_set() -> Self;
}
```

- [ ] **Step 1: Failing tests**

```rust
#[test]
fn domain_suffix_requires_dot_boundary() { // "foo.com" matches "a.foo.com" NOT "xfoo.com"
    let m = CompiledDomain::build(&DomainRulesSpec{
        suffix: vec!["foo.com".into()], ..empty_spec()}).unwrap();
    assert!(m.matches_domain("a.FOO.com"));
    assert!(!m.matches_domain("xfoo.com"));
}
#[test]
fn domain_exact_and_regex() {
    let m = CompiledDomain::build(&DomainRulesSpec{
        exact: vec!["api.example.com".into()],
        regexes: vec![r"^cdn\d+\.example\.com$".into()], ..empty_spec()}).unwrap();
    assert!(m.matches_domain("api.example.com") && m.matches_domain("cdn42.example.com"));
    assert!(!m.matches_domain("cdn.example.com"));
    assert!(matches!(CompiledDomain::build(&DomainRulesSpec{
        regexes: vec!["(".into()], ..empty_spec()}), Err(RouteError::Parse{..})));
}
#[test]
fn cidrset_v6_contains_exact_prefix() {
    let mut b = CidrSetBuilder::default();
    b.insert(Cidr::parse("fd00::/8").unwrap());
    let s = b.build();
    assert!(s.contains(&"fd00:dead::1".parse().unwrap()));
    assert!(!s.contains(&"fe80::1".parse().unwrap()));
}
#[test]
fn private_set_classifications() {
    let p = CidrSet::private_set();
    assert!(p.contains(&"10.9.9.9".parse().unwrap())
            && p.contains(&"172.31.0.1".parse().unwrap())
            && p.contains(&"100.64.0.7".parse().unwrap())
            && p.contains(&"::1".parse().unwrap())
            && p.contains(&"fc00::".parse().unwrap())
            && !p.contains(&"8.8.8.8".parse().unwrap()));
}
```

- [ ] **Step 2–4: red/green cycle** — normalize all inputs/stored strings to lowercase.
- [ ] **Step 5: Commit** `feat(route): compiled domain + cidr matchers`

---

### Task 4: Flat-rule engine — build + decide (Cond::All first-match)

**Files:**
- Create: `crates/xray-tui-route/src/events.rs`
- Create: `crates/xray-tui-route/src/engine.rs`

**Interfaces:**
- Consumes: Tasks 1–3.
- Produces:

```rust
// events.rs — Resolved / NetworkBreakdown / ProbeRecovered variants added in Task 9.
pub enum RouteEvent {
    DecisionApplied { rule_name: Option<String>, tag: Option<String>,
                      sni: Option<String>, at: jiff::Timestamp },
    CompileWarning { rule_index: usize, message: String },
}

// engine.rs
use std::{net::{IpAddr, SocketAddr}, sync::mpsc};

pub struct ConnMeta {
    pub target: NetAddr,
    pub network: NetworkMask,
    pub inbound_tag: Option<String>,
    pub source: Option<SocketAddr>,
    pub source_resolved_ips: Vec<IpAddr>,
    pub payload_prefix: Option<Vec<u8>>,   // caller-owned leading bytes (spec §2 data-flow 3)
    pub sniffed: Option<SniffedProtocol>,
    pub resolved_host_ips: Vec<IpAddr>,    // filled by IfNonMatch pass or pre-seeded by caller
}
impl ConnMeta { pub fn target_ips(&self) -> &[IpAddr]; }

pub enum Decision {
    Route { tag: String, override_addr: Option<NetAddr> },
    Reject { method: RejectMethod },
    HijackDns,
}

pub struct Engine { /* compiled rule vector + default + strategy */ }
impl Engine {
    pub fn build(rs: RuleSet) -> Result<Self, RouteError>;
    /// Sync pure evaluation: no DNS, no sniffing side effects beyond meta reads.
    pub fn decide(&self, meta: &mut ConnMeta) -> Decision;
    pub fn set_event_sink(&mut self, tx: tokio::sync::mpsc::UnboundedSender<RouteEvent>);
}
```

- [ ] **Step 1: Failing truth tables**

```rust
fn rs(rules: Vec<(MatchItem, Action)>) -> RuleSet { /* Cond::All single-item rules + default direct */ }
fn meta(host: &str, port: u16, network: NetworkMask) -> ConnMeta;

#[test] fn first_match_wins_in_declaration_order() {
    // rule1 ports 80..=80 → "a"; rule2 net tcp → "b"; tcp:80 request returns "a"
}
#[test] fn unmatched_items_skip_the_rule_not_abort() {
    // rule1 = ports[80] AND tcp ; udp:443 falls past it to default
}
#[test] fn default_fallback_when_nothing_matched();      // DefaultRoute::Route{"direct"}
#[test] fn reject_and_hijackdns_are_terminal();          // both arms observed exactly
```

- [ ] **Step 2–4: implement** — evaluate each item of `Cond::All` against meta; emit `DecisionApplied` after terminal return when sink set (`sni: None` until Task 12 wires sniffing).
- [ ] **Step 5: Commit** `feat(route): first-match engine loop with default fallback`

---

### Task 5: Xray-core JSON compiler

**Files:**
- Create: `crates/xray-tui-route/src/compiler/mod.rs` (+ shared glue types)
- Create: `crates/xray-tui-route/src/compiler/xray.rs`
- Create: `crates/xray-tui-route/tests/fixtures/xray_sample.json`

**Interfaces:**
- Produces:
```rust
pub struct CompileOutput { pub ruleset: RuleSet, pub warnings: Vec<(usize, String)> }
pub fn compile_xray(json_text: &str) -> Result<CompileOutput, RouteError>;
```

Fixture content mirrors real shapes from thirdparty/Xray-core samples:
```json
{ "routing": { "domainStrategy": "AsIs",
  "rules": [
    { "type": "field", "outboundTag": "block",
      "domain": ["domain:doubleclick.net", "keyword:adservice"] },
    { "type": "field", "outboundTag": "direct",
      "ip": ["geoip:private", "10.0.0.0/8"], "ports": "80,443,1000-2000" },
    { "type": "field", "outboundTag": "proxy-a",
      "domain": ["example.com"], "network": "tcp,udp" } ] } }
```

Vocabulary mapping lives verbatim in the doc-comment above `compile_xray` (copied from spec §4 table so executors read one file standalone).

`geoip:` tokens other than `private`: kept as `MatchItem::IpCidr.geo_country` entries; unknown country without the `geoip` feature fails at `Engine::build` listing exactly what is missing (no silent degradation anywhere).

- [ ] **Step 1: Failing fixture assertion test**

```rust
#[test]
fn fixture_parses_to_golden_ir() {
    let out = compile_xray(include_str!("../tests/fixtures/xray_sample.json")).unwrap();
    assert_eq!(out.ruleset.rules.len(), 3);
    assert_eq!(out.warnings.len(), 0);
    assert!(matches!(&out.ruleset.default, DefaultRoute::Route { tag } if tag == "proxy-a")); // final unmatched → proxy-a via fixture's tail rule semantics? NO — fixture has no fallback key; compile_xray defaults to Route{tag:"proxy"} documented constant, asserted here
    // per-rule assertions: suffix doubleclick.net, keyword adservice, IpCidr private+10/8, Ports=[80,443,{1000..2000}], NetworkMask tcp+udp
}
#[test]
fn unsupported_dat_geosite_is_positional_error() {
    let txt = r#"{ "routing": { "rules": [{ "type":"field","outboundTag":"x",
        "domain":["geosite:cn"] }] } }"#;
    match compile_xray(txt) {
        Err(RouteError::Parse { rule_index: 0, .. }) => {}
        other => panic!("{other:?}"),
    }
}
```

- [ ] **Step 2–4: implement parser** — navigate `serde_json::Value`; ports `"80,443"` comma-split then dash-ranges; network `"tcp,udp"` → bits; positional errors per Global Constraints.
- [ ] **Step 5: Commit** `feat(route): xray-core routing JSON compiler`

---

### Task 6: sing-box JSON compiler

**Files:**
- Create: `crates/xray-tui-route/src/compiler/singbox.rs`
- Create: `crates/xray-tui-route/tests/fixtures/singbox_sample.json`

**Interfaces:**
- Consumes: shared `CompileOutput` contract from Task 5.
- Produces: `pub fn compile_singbox(json_text: &str) -> Result<CompileOutput, RouteError>;`

```json
{ "route": { "final": "proxy-main",
  "rules": [
    { "domain_suffix": [".google.com"], "action": "hijack-dns" },
    { "ip_cidr": ["10.0.0.0/8"], "inbound": ["tun-in"], "mode": "and",
      "action": "route", "outbound": "local-bypass" } ],
  "rule_set": [{ "type": "local", "tag": "geo" }] } }
```
Notes pinned in doc-comments: missing `"action"` ⇒ route to `outbound`; `mode:"and"` + invert:false flattens to `Cond::All`; `"or"` produces `Cond::Any`; `bittorrent` protocol value hits `RouteError::Unsupported` (whitelist Http/Tls/Dns); `rule_set` presence ⇒ `RouteError::Unsupported` (asserted in test, covering the deferred-accelerator arm).

- [ ] **Step 1: Failing golden test** — asserting IR shape incl. logical AND flattening + Unsupported arms for `rule_set`.
- [ ] **Step 2–4: implement** (share navigation helpers from Task 5's `compiler/mod.rs`).
- [ ] **Step 5: Commit** `feat(route): sing-box route JSON compiler`

---

### Task 7: Merge front-ends into one RuleSet

**Files:**
- Modify: `crates/xray-tui-route/src/compiler/mod.rs`

**Interfaces:**
- Produces:
```rust
pub enum MergeOrigin { Xray, SingBox, DbRows, NativeFile }
pub fn merge(sources: Vec<(MergeOrigin, RuleSet)>) -> CompileOutput;
```
Semantics (locked):
- Rules concatenated in argument order.
- Tag collisions: later collider gets `-<source_index>` suffix; every later occurrence of that tag inside its own ruleset's `Action::Route.tag`/`DefaultRoute` remaps identically. Earlier sources untouched.
- Conflicting non-absent defaults: last-wins, exactly ONE `CompileWarning{rule_index:0, message:"conflicting defaults ..."}` appended.
- `probes`: union, deduped case-insensitively, first spelling preserved.
- Warnings from constituent compiles pass through unchanged.

- [ ] **Step 1: Failing tests** — three-source merge covering collision rename + cross-reference remap + duplicate-probe dedup + last-wins default warning + earlier-source isolation.
- [ ] **Step 2–4: implement.**
- [ ] **Step 5: Commit** `feat(route): multi-source ruleset merge`

---

### Task 8: TUI-crate converter — DB RoutingRule rows → IR

**Files:**
- Create: `crates/xray-tui/src/route_compile.rs` (unit tests in-file)

**Interfaces:**
- Consumes: `xray_tui_db::models_toasty::RoutingRule` columns (models_toasty.rs:306–327: `domain_matcher/domains/ips/inbound_tags/ports/source_ports/network/protocols/outbound_tag/balancer_tag/domain_strategy/sort_order/rule_set_file/rule_set_url`).
- Produces:
```rust
pub fn rule_from_row(row: &RoutingRule, index: usize)
    -> Result<xray_tui_route::ir::Rule, xray_tui_route::RouteError>;
```
Mapping table (verbatim):
- `domains` + `domain_matcher`: `Some("exact")` ⇒ MatchItem::Domain.exact; otherwise (None or `"domain"`) plain entries become `.suffix` (mirrors xray `domain:` default semantics).
- `ips`: parseable-CIDR entries → IpCidr.cidrs; `geoip:<cc>` prefixed → geo_country; private/loopback specials noted in warning when unrecognized.
- `ports` → Ports; `source_ports` → SourcePorts; `network` (comma-able `"tcp,udp"`) → NetworkMask item; `protocols` (IANA names http/tls/dns) → Protocol items ANDed; `inbound_tags` → InboundTag.
- `outbound_tag` → `Action::Route{tag}`.
- `balancer_tag` present ⇒ `Err(Unsupported("balancers deferred — see routing spec §1"))` surfaced to caller (never silently dropped).
- `rule_set_file/url` set ⇒ caller-visible warning string returned via second method `warnings_from_row(row:&RoutingRule)->Vec<String>`; not silent.
- `domain_strategy` per-row is xray legacy noise: ignored with a warning.

Ordering: caller sorts rows by `sort_order` before mapping (rows without it retain given order).

Row construction in tests initializes the model struct literally (all fields), mirroring existing db-crate test idioms.

- [ ] **Step 1: Failing coverage test** asserting EVERY RoutingRule column exercised (spec R3 gate): one test whose outcome differs per field + balancer error case + warning cases.
- [ ] **Step 2–4: implement converter.**
- [ ] **Step 5: Commit** `feat(tui): compile Settings→Routing rows into native IR`

---

### Task 9: Resolver seam + TTL cache + probe tracker

**Files:**
- Create: `crates/xray-tui-route/src/resolve.rs`
- Modify: `events.rs` (add variants below)

**Interfaces:**
- Produces:
```rust
pub trait DnsSink: Send + Sync {
    fn lookup_ip(&self, host: String)
        -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<IpAddr>, RouteError>> + Send>>;
}

#[cfg(feature = "dns")]
pub struct DnsSinkAdapter { pub resolver: Arc<xray_tui_dns::DnsResolver> }
#[cfg(feature = "dns")]
impl DnsSink for DnsSinkAdapter {
    /// delegates resolver.lookup_ip(hostname, allow_ipv6=true) and maps anyhow errors to RouteError::Resolve
}

pub struct ResolvedCache { /* HashMap<String,(Vec<IpAddr>, jiff::Timestamp)> , ttl_secs */ }
impl ResolvedCache {
    pub fn new(ttl_secs: i64) -> Self;
    pub fn get_fresh(&self, host: &str, now: jiff::Timestamp) -> Option<&[IpAddr]>;
    pub fn put(&mut self, host: String, ips: Vec<IpAddr>, now: jiff::Timestamp);
}

/// Consecutive-failure streak tracker; zero-cost no-op while probes list empty.
pub struct ProbeTracker { streaks: HashMap<String, u32> }
impl ProbeTracker {
    pub fn update(&mut self, probes: &[String], any_failed_this_cycle: bool,
                  sink_probe_result: Option<(bool /*failed*/, Option<&str> /*which*/)>,
                  tx: &Option<tokio::sync::mpsc::UnboundedSender<RouteEvent>>);
}
```
Events additions:
```rust
Resolved { host: String, ips: Vec<IpAddr>, at: jiff::Timestamp },
NetworkBreakdown { failed_probe: String, at: jiff::Timestamp },
ProbeRecovered { probe: String, at: jiff::Timestamp },
```

Plugin point consumed by `Engine::with_resolver(Arc<dyn DnsSink>)` in Task 12 — mechanics live here.

- [ ] **Step 1: Faking-sink battery tests** — sink defined ONCE in `tests/common/mod.rs` (shared with Task 12):

```rust
// crates/xray-tui-route/tests/common/mod.rs
use std::{future::Future, pin::Pin};
use xray_tui_route::{error::RouteError, resolve::DnsSink};

pub struct SeqSink {
    pub results: std::sync::Mutex<Vec<Result<Vec<std::net::IpAddr>, RouteError>>>,
}
impl DnsSink for SeqSink {
    fn lookup_ip(&self, _host: String)
        -> Pin<Box<dyn Future<Output = Result<Vec<std::net::IpAddr>, RouteError>> + Send>> {
        let mut q = self.results.lock().unwrap();
        let r = if q.is_empty()
            { Err(RouteError::Resolve("exhausted".into())) } else { q.remove(0) };
        Box::pin(async move { r })
    }
}
```

Unit battery in resolve.rs `#[cfg(test)]` uses tokio dev-dep runtime:

```rust
#[tokio::test] async fn cache_ttl_expiry_forces_refetch();
#[tokio::test] async fn probe_streak_fail_then_recover_emits_exactly_once_each();
#[tokio::test] async fn streak_reset_on_success_between_failures();
#[tokio::test] async fn probe_list_empty_means_zero_events();
```

- [ ] **Step 2–4: implement** (no network I/O lives here; production adapter stays feature-gated).
- [ ] **Step 5: Commit** `feat(route): resolver seam, TTL cache, breakdown probes`

---

### Task 10: Sniffer — TLS ClientHello SNI + HTTP Host

**Files:**
- Create: `crates/xray-tui-route/src/sniff.rs`
- Create: `crates/xray-tui-route/tests/fixtures/tls_hello_chrome.bin`

**Interfaces:**
- Produces:
```rust
pub struct SniffResult { pub protocol: SniffedProtocol, pub host: Option<String> }
/// None = indeterminate (garbage/truncated/oversize) — never panics on malformed wire data.
pub fn probe(bytes: &[u8]) -> Option<SniffResult>;
```
TLS arm: `bytes[0]==0x16`, record version ≥ 0x0301, single ClientHello bounds-checked walk through session-id/cipher-suites/compression/extensions, extracting server_name extension (type 0x0000). HTTP arm: request-line start, `\r\n\r\n` present within slice, case-insensitive `host:` header trimmed. Slice > 64 KiB ⇒ early None.

The chrome hello fixture binary comes from rendering `chrome_130` via `xray-tui-tls`'s own hand-profile API inside the test (never hand-hexed):

```rust
// construct via xray_tui_tls public API in the test body; serialize bytes once with a tiny
// bincode-free hex dump helper committed alongside — deterministic across runs.
```

- [ ] **Step 1: Byte-fixture tests**

```rust
#[test] fn tls_hello_yields_tls_with_sni_target_example_com();
#[test] fn http_get_request_yields_http_host_case_insensitive_and_trimmed();
#[test] fn garbage_returns_none();
#[test] fn truncated_hello_returns_none();
#[test] fn oversized_slice_returns_none_early();
```

- [ ] **Step 2–4: implement** (lowercase-only compares; every length prefix validated before read).
- [ ] **Step 5: Commit** `feat(route): tls/http sniffer over bounded payload prefix`

---

### Task 11: Logical conditions — Any + Invert evaluator

**Files:**
- Modify: `crates/xray-tui-route/src/engine.rs`

**Interfaces:**
- Change only internal eval: recursive `eval_cond(cond: &Cond, meta: &ConnMeta) -> bool` replacing flat loop's per-item iteration. Public API surface unchanged; decisions/events identical.

- [ ] **Step 1: Truth-matrix tests**

```rust
#[test] fn any_short_circuits_on_first_true_arm();
#[test] fn invert_negates_subtree_result();
#[test] fn nested_any_inside_all_inside_invert_evaluates_correctly();
    // semantics annotated inline citing upstream route/rule/rule_abstract.go behavior
#[test] fn flat_all_compiles_identically_to_before_task11(); // guards regression vs Task 4 tables
```

- [ ] **Step 2–4: implement recursion.**
- [ ] **Step 5: Commit** `feat(route): logical Any/Invert condition evaluation`

---

### Task 12: Engine integration — needs declarations, sniff enrichment, IfNonMatch resolve

**Files:**
- Modify: `engine.rs`
- Modify: `resolve.rs` (ProbeTracker consumption glue)
- Create: `crates/xray-tui-route/tests/engine_integration.rs`
- Reuse: `tests/common/mod.rs` SeqSink from Task 9

**Interfaces:**
- Consumes: Tasks 3, 4, 9, 10.
- Produces:
```rust
impl Engine {
    pub fn with_resolver(mut self, sink: std::sync::Arc<dyn DnsSink>) -> Self;
    /// True when any rule carries a Protocol item needing payload_prefix sniffing.
    pub fn needs_sniff(&self) -> bool;
    /// True when strategy==IfNonMatch OR IP-bearing rules could need target resolution.
    pub fn needs_resolve(&self) -> bool;
}
pub async fn decide_async(engine: &Engine, meta: &mut ConnMeta) -> Decision;
```
Locked semantics (doc-comment these verbatim in decide_async):
- `Protocol(item)`: if `meta.sniffed.is_none()` && `meta.payload_prefix.is_some()` → run `sniff::probe` once per connection, stash onto meta. Missing both ⇒ item evaluates FALSE; sync `decide()` remains fully usable sans prefix.
- IfNonMatch: resolver Some + unresolved domain target ⇒ await resolve once, fill `resolved_host_ips`, retry whole loop under cycle-guard flag preventing further passes.
- After EVERY resolve attempt run ProbeTracker with combined result semantics: success only when Ok(non-empty); `Ok(vec![])` (NXDOMAIN-style miss) counts failed=true; `Err(_)` (transport broke) ALSO counts failed=true — breakdown probing measures reachability, mirroring user intent #4 verbatim from the spec header.
- Resolver failures degrade silently per-connection (no Decision-level error branch).

Integration battery:

```rust
#[tokio::test] async fn if_non_match_resolves_once_then_matches_ip_rule();
#[tokio::test] async fn cycle_guard_prevents_second_resolution_pass();
#[tokio::test] async fn protocol_item_consumes_payload_prefix_sniff();
#[tokio::test] async fn probe_breakdown_and_recovery_flow_end_to_end_via_events_rx();
#[tokio::test] async fn needs_flags_reflect_declared_item_mix();
```

- [ ] Steps red→green→commit `feat(route): lazy resolve + sniff enrichment wired into decide`

---

### Task 13: Workspace integration — hakari + lint gate

**Files:**
- Regenerate: `crates/xray-tui-hakari/Cargo.toml` via `cargo hakari generate`
- Root Cargo.toml: likely untouched (jiff already workspace-shared); touch ONLY if a new dep appears in ≥2 crates.

- [ ] Step 1: `cargo hakari generate && just quality-gate code`
- [ ] Step 2: fix clippy/fmt fallout; commit `chore(route): hakari regen + quality gate green`

---

### Task 14: DB storage for probes + schema tag bump

**Files:**
- Modify: `crates/xray-tui-db/src/models_toasty.rs` — append model following in-file idioms:

```rust
#[derive(Debug, Clone, toasty::Model)]
#[table = "route_probes"]
pub struct RouteProbes {
    #[key]
    pub id: String,     // singleton row, id == "global"
    pub hosts: Vec<String>,
}
```

- Modify: `crates/xray-tui-db/src/database.rs` — bump `PRAGMA user_version=5` constant to 6; add methods modeled on existing typed ones:
```rust
pub async fn get_route_probes(&self) -> Vec<String>;
pub async fn upsert_route_probes(&self, hosts: Vec<String>);
```

- [ ] Step 1: failing roundtrip test adjacent to existing database.rs tests (insert→fetch→update path)
- [ ] Step 2–4: implement model + methods + tag bump
- [ ] Step 5: Commit `feat(db): route_probes singleton table (schema v6)`

---

### Task 15: TUI bridge — CoreEvent fan-out, Actions Log surfacing, probes editor

**Files:**
- Modify: `crates/xray-tui/src/types.rs` — add CoreEvent variant (types.rs:279 enum):

```rust
/// Native-core routing decision/probe event (surfaced in Actions Log).
Route(xray_tui_route::events::RouteEvent),
```

- Modify: `crates/xray-tui/src/ops/events.rs` — handle `CoreEvent::Route(ev)` in poll_core_events' match: render each variant into an existing `LogLine { level, target:"route", message, timestamp_nanos }` and feed the SAME insertion path the current `LogLine` arm takes directly above (locate by reading that arm; mirror it 1:1). Rendering helper kept pure + unit-tested:

```rust
fn render_route_event(ev: &RouteEvent) -> LogLine {
    use xray_tui_route::events::RouteEvent::*;
    let msg = match ev {
        DecisionApplied { rule_name, tag, sni, at } =>
            format!("route: {} → {}{} ({at})", rule_name.as_deref().unwrap_or("<rule>"),
                    tag.as_deref().unwrap_or("<default>"),
                    sni.as_deref().map(|s| format!(" sni={s}")).unwrap_or_default()),
        Resolved { host, ips, at } => format!("route: resolved {host} → {ips:?} ({at})"),
        NetworkBreakdown { failed_probe, at } =>
            format!("route: NETWORK BREAKDOWN probe {failed_probe} ({at})"),
        ProbeRecovered { probe, at } => format!("route: probe recovered {probe} ({at})"),
        CompileWarning { rule_index, message } =>
            format!("route: compile warning rule#{rule_index}: {message}"),
    };
    LogLine { level: "info".into(), target: "route".into(), message: msg,
              timestamp_nanos: at.as_second() * 1_000_000_000 + at.subsec_nanosecond() as i64 }
}
```

- Modify producer wiring where the native engine gets instantiated for routed sessions: attach `UnboundedSender<RouteEvent>` via `Engine::set_event_sink` and spawn a forwarding task next to existing event pumps converting received events into `core_event_tx.send(CoreEvent::Route(..))`.

- Settings probes editor (spec R8 deliverable): reuse the split-pane Settings machinery (AGENTS.md "Adding subscription management features" recipe naming ui/settings.rs SplitRightPane/GroupList-GroupForm patterns as the template). Add a Routing-section-adjacent single-textarea pane editing the newline-separated probe host list; load via `db.get_route_probes()`, save button writes `db.upsert_route_probes(split_input())`. Landing spot inside `SETTINGS_TREE` near the Routing section entry is implementer-chosen (tree/name wiring follows the exact SETTINGS_TREE const pattern at ui/settings.rs).

- [ ] Step 1: `cargo build -p xray-tui` green after type changes
- [ ] Step 2: unit test around render_route_event (all five variants assert rendered fields)
- [ ] Step 3: Commit `feat(tui): route events bridge into Actions Log + probes editor`

*(Full dial-through-native-routed-tag e2e harness rows land with the fast-switch/native-connect SP; verification tier stays tier-1 hermetic per spec §8 until that exists.)*

---

## Execution Notes

- Tasks 1–12 are pure-library and independently committable; Task 13 gates the whole workspace; 14–15 touch consumers.
- Read spec §2 mermaid diagram before wiring Task 15's producer side.
- Keep imports tight per-task rather than sweeping later (`cargo machete` gate runs globally).
