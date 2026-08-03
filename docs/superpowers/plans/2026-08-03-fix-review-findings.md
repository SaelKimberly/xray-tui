# Fix Review Findings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix all 31 findings from the CodebaseReview of xray-tui (5 critical, 22 medium, 4 low), excluding L4 (subscription_url_split Aho-Corasick behavior — user explicitly chose to keep current behavior).

**Architecture:** 7 independent phases by subsystem: config builders, import/parse, ping/speed, profile lifecycle/DB, enrichment + utility crates, UI/UX, core plumbing. Each phase touches disjoint files; tasks within a phase are sequential (shared files), phases are independent. TDD: write failing test → verify fail → implement → verify pass → commit.

**Tech Stack:** Rust 2024 workspace (9 crates), tokio, serde_json, toasty ORM 0.9 + turso, heed LMDB, ratatui, reqwest, tonic.

## Global Constraints

- Rust 2024 edition; `cargo fmt` + clippy (workspace lints pedantic+nursery at warn) must stay clean on touched files.
- All DB `spec_blob` values are legacy JSON wrapped in `PlaceholderConfig` (verified — `convert_spec_blob` and `encode_profile_spec` both wrap; typed `ProtocolConfig` variants are never stored today). Fixes MUST handle the legacy dotted-key format, not just the typed format.
- `parse_settings()` (config_builder/mod.rs:39) tries `serde_json::from_slice::<ProtocolConfig>` first, falls back to raw JSON extraction of `protocol_settings`/`stream_settings`.
- Port allocation: there must be exactly ONE shared port counter between CorePool and batch real-ping after Task 10.
- `aggressive::clippy` pedantic: no `unwrap()` on user-controlled data; use `saturating_*`/`checked_*` where overflow is possible.
- Do NOT touch `subscription.rs` `subscription_url_split` (user decision).
- Every network call in touched code MUST have a hard deadline (repo rule).
- Each task ends with a passing targeted test and a commit. Full `cargo test` + `cargo clippy` run once at the end of each phase.

---

## Phase 1: Config Builders

### Task 1: sing-box outbound builder reads credentials from p_settings (C1)

**Files:**
- Modify: `crates/xray-tui-core/src/config_builder/singbox.rs:199` (and test module `:1020+`)
- Test: same file, `mod tests`

**Interfaces:**
- Consumes: `parse_settings()` returning `(p_settings, _)` where typed `to_settings` puts `id`/`uuid`/`password` into p_settings (verified: Vmess→`id`, Vless→`id`, Trojan→`password`, Ss→`password`, Ssr→`password`, Tuic→`uuid`+`password`).
- Produces: `build_proxy_outbound` emits real credentials for TUIC/SS/SSR/VMess/VLESS/Trojan.

- [ ] **Step 1: Write the failing test**

In `singbox.rs` `mod tests`, after the existing helpers:

```rust
#[test]
fn proxy_outbound_uses_protocol_credentials_not_empty_user_id() {
    let (endpoint, mut protocol) = test_endpoint_and_protocol(Protocol::Tuic as i32);
    // Typed TUIC to_settings puts uuid+password in p_settings.
    set_protocol_settings_json(
        &mut protocol,
        r#"{"uuid": "11111111-2222-3333-4444-555555555555", "password": "sekrit"}"#,
    );
    let (params, rules, dns) = default_params();
    let config = ConfigBuilder::build(&endpoint, &protocol, CoreType::SingBox, &params, &rules, &dns)
        .expect("build");
    let json = config.to_json();
    let outbounds = json["outbounds"].as_array().expect("outbounds");
    let proxy = outbounds.iter().find(|o| o["tag"] == "proxy").expect("proxy");
    assert_eq!(
        proxy["uuid"].as_str().unwrap(),
        "11111111-2222-3333-4444-555555555555"
    );
    assert_eq!(proxy["password"].as_str().unwrap(), "sekrit");
}
```

(Check the exact `BackendConfig`/`to_json` API used by existing singbox tests — e.g. `assert_singbox_top_level(&json)` at the existing tests; mirror their construction. If the existing tests call something like `config.to_json()`, use that.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-core proxy_outbound_uses_protocol_credentials_not_empty_user_id`
Expected: FAIL — `proxy["uuid"]` is `""`.

- [ ] **Step 3: Implement**

Replace line 199:

```rust
let user_id = ""; // TODO: cached on ProtocolRow
```

with:

```rust
// Credentials live in p_settings (typed to_settings puts id/uuid/password
// there; the legacy path injects user_id as "id"). Same extraction as the
// xray builder.
let user_id = p_settings
    .get("id")
    .or_else(|| p_settings.get("uuid"))
    .or_else(|| p_settings.get("password"))
    .or_else(|| p_settings.get("pass"))
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();
```

Note: `let (p_settings, _s_settings) = parse_settings(protocol);` already exists above it — keep that binding.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-core proxy_outbound_uses_protocol_credentials_not_empty_user_id`
Expected: PASS. Also run the full builder test module: `cargo test -p xray-tui-core config_builder::singbox` — all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-core/src/config_builder/singbox.rs
git commit -m "fix(singbox): read outbound credentials from p_settings instead of empty user_id"
```

### Task 2: xray streamSettings synthesis — TLS/transport no longer dropped (C2)

**Files:**
- Modify: `crates/xray-tui-proto/src/proto_spec/common.rs` (add converter)
- Modify: `crates/xray-tui-proto/src/proto_spec/mod.rs` (typed `to_settings` for Vmess/Vless/Trojan)
- Modify: `crates/xray-tui-core/src/config_builder/xray.rs` (use converter; legacy dotted conversion)
- Test: `xray.rs` `mod tests` + `proto_spec/mod.rs` tests

**Interfaces:**
- Consumes: `SecurityConfig`, `TlsConfig::{Tls(TlsOpts), Reality(RealityOpts)}`, `TransportConfig` (all in proto_spec/common.rs); legacy `stream_settings` JSON with dotted keys (`ws.path`, `ws.host`, `grpc.serviceName`, `tls.enable`, `sni`, `fingerprint`, `alpn`, `security`, `realitySettings.*`).
- Produces: `pub fn to_xray_stream_settings(security: &SecurityConfig, transport: &TransportConfig) -> Option<serde_json::Value>` in proto common.rs; xray builder emits xray-shaped `streamSettings` for both typed and legacy profiles.

- [ ] **Step 1: Write the failing tests (xray builder, legacy profile)**

In `xray.rs` `mod tests`, add (mirror existing test helpers — `test_endpoint_and_protocol`, `set_protocol_settings_json`, `set_stream_settings_json` exist in singbox tests; xray.rs:519 has its own `mod tests` — check and reuse/duplicate the helpers there):

```rust
#[test]
fn legacy_vless_ws_tls_produces_xray_stream_settings() {
    let (endpoint, mut protocol) = test_endpoint_and_protocol(Protocol::Vless as i32);
    protocol.network = Some("ws".to_string());
    set_stream_settings_json(
        &mut protocol,
        r#"{"tls.enable": true, "sni": "cdn.example.com", "ws.path": "/ws", "ws.host": "cdn.example.com"}"#,
    );
    let (params, rules, dns) = default_params();
    let config = ConfigBuilder::build(&endpoint, &protocol, CoreType::Xray, &params, &rules, &dns)
        .expect("build");
    let json = config.to_json();
    let outbounds = json["outbounds"].as_array().expect("outbounds");
    let proxy = outbounds.iter().find(|o| o["tag"] == "proxy").expect("proxy");
    let ss = proxy["streamSettings"].as_object().expect("streamSettings present");
    assert_eq!(ss["network"], "ws");
    assert_eq!(ss["security"], "tls");
    assert_eq!(ss["tlsSettings"]["serverName"], "cdn.example.com");
    assert_eq!(ss["wsSettings"]["path"], "/ws");
    assert_eq!(ss["wsSettings"]["headers"]["Host"], "cdn.example.com");
}
```

And a typed-config test in `crates/xray-tui-proto/src/proto_spec/mod.rs` (or common.rs tests):

```rust
#[test]
fn vless_to_settings_emits_xray_stream_settings() {
    // Build a typed VlessConfig with security=tls, transport=ws and assert
    // to_settings() returns non-empty s_settings with xray shape.
}
```

(Construct the typed config via `VlessConfig::try_parse` on a `vless://uuid@host:443?security=tls&type=ws&path=%2Fws&host=cdn.example.com#r` URL — parse helpers live in proto_spec/vless.rs; then call `ProtocolConfig::Vless(c).to_settings()` and assert `s_settings["network"]=="ws"`, `s_settings["security"]=="tls"`, `s_settings["wsSettings"]["path"]=="/ws"`.)

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p xray-tui-core legacy_vless_ws_tls_produces_xray_stream_settings`
Expected: FAIL — `proxy.get("streamSettings")` is None (assert on `streamSettings present` panics).
Run: `cargo test -p xray-tui-proto vless_to_settings_emits_xray_stream_settings`
Expected: FAIL — s_settings is `{}`.

- [ ] **Step 3a: Add the converter to `common.rs`**

```rust
/// Build xray-core `streamSettings` JSON from typed security + transport.
/// Returns `None` when there is nothing to emit (tcp + no TLS).
pub fn to_xray_stream_settings(
    security: &SecurityConfig,
    transport: &TransportConfig,
) -> Option<serde_json::Value> {
    let mut ss = serde_json::Map::new();
    let network = transport.type_str();
    if network != "tcp" {
        ss.insert("network".into(), serde_json::Value::String(network.to_string()));
    }
    match &security.tls {
        Some(TlsConfig::Tls(opts)) => {
            ss.insert("security".into(), serde_json::json!("tls"));
            let mut t = serde_json::Map::new();
            if let Some(ref sni) = opts.sni {
                t.insert("serverName".into(), serde_json::json!(sni.as_str()));
            }
            if let Some(insecure) = opts.insecure {
                t.insert("allowInsecure".into(), serde_json::json!(insecure));
            }
            if let Some(ref fp) = opts.fp {
                t.insert("fingerprint".into(), serde_json::json!(fp.as_str()));
            }
            if let Some(ref alpn) = opts.alpn {
                let list: Vec<&str> = alpn.split(',').map(str::trim).collect();
                t.insert("alpn".into(), serde_json::json!(list));
            }
            if !t.is_empty() {
                ss.insert("tlsSettings".into(), serde_json::Value::Object(t));
            }
        }
        Some(TlsConfig::Reality(opts)) => {
            ss.insert("security".into(), serde_json::json!("reality"));
            let mut r = serde_json::Map::new();
            if let Some(ref sni) = opts.sni {
                r.insert("serverName".into(), serde_json::json!(sni.as_str()));
            }
            if let Some(ref pbk) = opts.pbk {
                r.insert("publicKey".into(), serde_json::json!(pbk));
            }
            if let Some(ref sid) = opts.sid {
                r.insert("shortId".into(), serde_json::json!(sid.as_str()));
            }
            if let Some(ref spx) = opts.spx {
                r.insert("spiderX".into(), serde_json::json!(spx.as_str()));
            }
            if let Some(ref fp) = opts.fp {
                r.insert("fingerprint".into(), serde_json::json!(fp.as_str()));
            }
            ss.insert("realitySettings".into(), serde_json::Value::Object(r));
        }
        None => {}
    }
    match transport {
        TransportConfig::Ws(cfg) => {
            let mut w = serde_json::Map::new();
            if let Some(ref p) = cfg.path {
                w.insert("path".into(), serde_json::json!(p.as_str()));
            }
            if let Some(ref h) = cfg.host {
                w.insert(
                    "headers".into(),
                    serde_json::json!({ "Host": h.as_str() }),
                );
            }
            if !w.is_empty() {
                ss.insert("wsSettings".into(), serde_json::Value::Object(w));
            }
        }
        TransportConfig::Grpc(cfg) => {
            let mut g = serde_json::Map::new();
            if let Some(ref sn) = cfg.service_name {
                g.insert("serviceName".into(), serde_json::json!(sn.as_str()));
            }
            if !g.is_empty() {
                ss.insert("grpcSettings".into(), serde_json::Value::Object(g));
            }
        }
        TransportConfig::Http(cfg) => {
            let mut h = serde_json::Map::new();
            if let Some(ref p) = cfg.path {
                h.insert("path".into(), serde_json::json!(p.as_str()));
            }
            if let Some(ref host) = cfg.host {
                h.insert("host".into(), serde_json::json!([host.as_str()]));
            }
            if !h.is_empty() {
                ss.insert("httpSettings".into(), serde_json::Value::Object(h));
            }
        }
        TransportConfig::HttpUpgrade(cfg) => {
            let mut u = serde_json::Map::new();
            if let Some(ref p) = cfg.path {
                u.insert("path".into(), serde_json::json!(p.as_str()));
            }
            if let Some(ref host) = cfg.host {
                u.insert("host".into(), serde_json::json!([host.as_str()]));
            }
            if !u.is_empty() {
                ss.insert("httpupgradeSettings".into(), serde_json::Value::Object(u));
            }
        }
        TransportConfig::XHttp(cfg) => {
            let mut x = serde_json::Map::new();
            if let Some(ref p) = cfg.path {
                x.insert("path".into(), serde_json::json!(p.as_str()));
            }
            if let Some(ref host) = cfg.host {
                x.insert("host".into(), serde_json::json!(host.as_str()));
            }
            if !x.is_empty() {
                ss.insert("splithttpSettings".into(), serde_json::Value::Object(x));
            }
        }
        TransportConfig::Tcp | TransportConfig::Quic | TransportConfig::Kcp(_) => {}
    }
    if ss.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(ss))
    }
}
```

- [ ] **Step 3b: Wire typed `to_settings` in `proto_spec/mod.rs`**

For `Self::Vmess(c)`, `Self::Vless(c)`, `Self::Trojan(c)` arms: replace the `json!({})` second tuple element with `to_xray_stream_settings(&c.security, &c.transport).unwrap_or_else(|| json!({}))`. (Check `TrojanConfig` has `security`/`transport` fields — verify in proto_spec/trojan.rs; if it lacks them, build from what it has or skip the typed arm for Trojan and rely on the legacy converter.)

- [ ] **Step 3c: Legacy dotted-key conversion in `xray.rs`**

In `build_proxy_outbound`, replace the `s_settings` computation block:

```rust
let s_settings =
    if s_settings_raw.is_null() || s_settings_raw.as_object().is_some_and(|o| o.is_empty()) {
        None
    } else {
        Some(s_settings_raw)
    };
```

with:

```rust
let s_settings = build_xray_stream_settings(protocol, s_settings_raw);
```

and add this module-level fn (plus the dotted-key expansion helper):

```rust
/// xray-shaped `streamSettings` for a profile. Typed configs (Vmess/Vless/
/// Trojan) build from SecurityConfig+TransportConfig; legacy
/// PlaceholderConfig blobs carry a homegrown dotted-key format
/// ("ws.path", "tls.enable", "realitySettings.publicKey", ...) that must be
/// expanded into the xray shape.
fn build_xray_stream_settings(
    protocol: &ProtocolRow,
    s_settings_raw: serde_json::Value,
) -> Option<serde_json::Value> {
    use xray_tui_proto::proto_spec::ProtocolConfig;
    match serde_json::from_slice::<ProtocolConfig>(&protocol.spec_blob) {
        Ok(ProtocolConfig::Vmess(c)) => {
            xray_tui_proto::proto_spec::common::to_xray_stream_settings(&c.security, &c.transport)
        }
        Ok(ProtocolConfig::Vless(c)) => {
            xray_tui_proto::proto_spec::common::to_xray_stream_settings(&c.security, &c.transport)
        }
        Ok(ProtocolConfig::Trojan(c)) => {
            xray_tui_proto::proto_spec::common::to_xray_stream_settings(&c.security, &c.transport)
        }
        _ => legacy_stream_settings_to_xray(s_settings_raw, protocol.network.as_deref()),
    }
}

/// Expand the legacy dotted-key stream_settings format into xray shape.
fn legacy_stream_settings_to_xray(
    raw: serde_json::Value,
    network: Option<&str>,
) -> Option<serde_json::Value> {
    let obj = raw.as_object()?;
    if obj.is_empty() {
        return None;
    }
    let mut ss: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    let mut get = |k: &str| obj.get(k).cloned();

    if let Some(net) = network.filter(|n| !n.is_empty() && *n != "tcp") {
        ss.insert("network".into(), serde_json::json!(net));
    }
    // Legacy `security` == "reality" (the only value the legacy parser stored).
    if get("security").and_then(|v| v.as_str()) == Some("reality") {
        ss.insert("security".into(), serde_json::json!("reality"));
    } else if get("tls.enable").and_then(|v| v.as_bool()) == Some(true)
        || get("tls.enable").and_then(|v| v.as_str()).is_some()
    {
        ss.insert("security".into(), serde_json::json!("tls"));
    }
    let mut tls = serde_json::Map::new();
    if let Some(v) = get("sni").and_then(|v| v.as_str().map(str::to_string)) {
        tls.insert("serverName".into(), serde_json::json!(v));
    }
    if let Some(v) = get("tls.allow_insecure").and_then(|v| v.as_bool()) {
        tls.insert("allowInsecure".into(), serde_json::json!(v));
    }
    if let Some(v) = get("fingerprint").and_then(|v| v.as_str().map(str::to_string)) {
        tls.insert("fingerprint".into(), serde_json::json!(v));
    }
    if let Some(v) = get("alpn").and_then(|v| v.as_str().map(str::to_string)) {
        let list: Vec<&str> = v.split(',').map(str::trim).collect();
        tls.insert("alpn".into(), serde_json::json!(list));
    }
    if ss.get("security") == Some(&serde_json::json!("reality")) {
        // realitySettings already xray-shaped in legacy output
        if let Some(rs) = obj.get("realitySettings").cloned() {
            ss.insert("realitySettings".into(), rs);
        }
        if let Some(server_name) = tls.get("serverName").cloned() {
            if let Some(rs) = ss.get_mut("realitySettings").and_then(|v| v.as_object_mut()) {
                rs.insert("serverName".into(), server_name);
            }
        }
    } else if !tls.is_empty() {
        ss.insert("tlsSettings".into(), serde_json::Value::Object(tls));
    }
    // Transport blocks
    let mut ws = serde_json::Map::new();
    if let Some(v) = get("ws.path").and_then(|v| v.as_str().map(str::to_string)) {
        ws.insert("path".into(), serde_json::json!(v));
    }
    if let Some(v) = get("ws.host").and_then(|v| v.as_str().map(str::to_string)) {
        ws.insert("headers".into(), serde_json::json!({ "Host": v }));
    }
    if !ws.is_empty() {
        ss.insert("wsSettings".into(), serde_json::Value::Object(ws));
    }
    let mut grpc = serde_json::Map::new();
    if let Some(v) = get("grpc.serviceName").and_then(|v| v.as_str().map(str::to_string)) {
        grpc.insert("serviceName".into(), serde_json::json!(v));
    }
    if !grpc.is_empty() {
        ss.insert("grpcSettings".into(), serde_json::Value::Object(grpc));
    }
    if ss.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(ss))
    }
}
```

Existing xray tests at :791/:805 assert `proxy.get("streamSettings").is_none()` for plain profiles — those must still pass (tcp + no TLS → None).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p xray-tui-core legacy_vless_ws_tls_produces_xray_stream_settings` and `cargo test -p xray-tui-core config_builder::xray` (whole module).
Run: `cargo test -p xray-tui-proto vless_to_settings_emits_xray_stream_settings` and the full proto test suite.
Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-proto/src/proto_spec/common.rs crates/xray-tui-proto/src/proto_spec/mod.rs crates/xray-tui-core/src/config_builder/xray.rs
git commit -m "fix(xray): synthesize streamSettings for TLS/transport instead of dropping it"
```

### Task 3: xray Hysteria2 outbound emits auth (M5)

**Files:**
- Modify: `crates/xray-tui-core/src/config_builder/xray.rs` (Hysteria2 arm, ~:378-390)
- Test: `xray.rs` `mod tests`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn xray_hysteria2_outbound_includes_auth() {
    let (endpoint, mut protocol) = test_endpoint_and_protocol(Protocol::Hysteria2 as i32);
    set_protocol_settings_json(&mut protocol, r#"{"password": "hy2-secret"}"#);
    let (params, rules, dns) = default_params();
    let config = ConfigBuilder::build(&endpoint, &protocol, CoreType::Xray, &params, &rules, &dns)
        .expect("build");
    let json = config.to_json();
    let proxy = json["outbounds"].as_array().unwrap().iter()
        .find(|o| o["tag"] == "proxy").expect("proxy");
    assert_eq!(proxy["settings"]["auth"].as_str().unwrap(), "hy2-secret");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-core xray_hysteria2_outbound_includes_auth`
Expected: FAIL — `settings` has no `auth` key.

- [ ] **Step 3: Implement**

In the `Protocol::Hysteria2` arm of `build_proxy_outbound` (xray.rs), change the settings json:

```rust
Protocol::Hysteria2 => {
    let auth = p_settings
        .get("password")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    Ok(Outbound {
        tag: "proxy".to_string(),
        protocol: "hysteria2".to_string(),
        settings: json!({
            "version": 2,
            "address": address,
            "port": port,
            "auth": auth
        }),
        stream_settings: s_settings.clone(),
    })
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-core xray_hysteria2_outbound_includes_auth` — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-core/src/config_builder/xray.rs
git commit -m "fix(xray): add auth to hysteria2 outbound"
```

### Task 4: Routing rules — validate ≥1 matcher, skip matcher-less, emit dead fields (M19)

**Files:**
- Modify: `crates/xray-tui/src/ops/settings.rs:571-594` (`save_routing_rule`)
- Modify: `crates/xray-tui-core/src/config_builder/xray.rs` (`build_routing`, ~:441-478)
- Modify: `crates/xray-tui-core/src/config_builder/singbox.rs` (`build_routing`, ~:764-804)
- Test: xray.rs + singbox.rs test modules; settings.rs is TUI-side (no unit tests — cover via builder tests)

- [ ] **Step 1: Write the failing tests**

xray.rs:

```rust
#[test]
fn routing_skips_matcher_less_rules() {
    let rule = RoutingRule {
        id: "r1".to_string(), group_id: None, r#type: 0, domain_matcher: None,
        domains: None, ips: None, inbound_tags: None, port: None, source_ports: None,
        network: None, protocols: None, domain_strategy: None,
        outbound_tag: Some("direct".to_string()), balancer_tag: None,
        rule_set_file: None, rule_set_url: None, sort_order: None,
    };
    let routing = build_routing(&[rule], false);
    assert!(routing.rules.is_empty());
}

#[test]
fn routing_emits_protocols_and_domain_matcher() {
    let rule = RoutingRule {
        id: "r2".to_string(), group_id: None, r#type: 0,
        domain_matcher: Some("linear".to_string()),
        domains: Some("example.com".to_string()), ips: None, inbound_tags: None,
        port: None, source_ports: None, network: Some("tcp".to_string()),
        protocols: Some("http,tls".to_string()), domain_strategy: None,
        outbound_tag: Some("proxy".to_string()), balancer_tag: None,
        rule_set_file: None, rule_set_url: None, sort_order: None,
    };
    let routing = build_routing(&[rule], false);
    let rule_json = &routing.rules[0];
    assert_eq!(rule_json["domainMatcher"], "linear");
    assert_eq!(rule_json["protocol"], json!(["http", "tls"]));
}
```

singbox.rs (same shape — assert `routing.rules` empty for matcher-less, and `rule_json["domain_matcher"]`/`rule_json["protocol"]` for the emitted case; check singbox RouteConfig field name for rules — it is `rules: Vec<Value>`).

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p xray-tui-core routing_skips_matcher_less_rules`
Expected: FAIL — rules is not empty (`{"type":"field","outboundTag":"direct"}` present).

- [ ] **Step 3: Implement**

xray.rs `build_routing` — inside the `filter_map`, before building the rule, add:

```rust
let has_matcher = r.domains.is_some()
    || r.ips.is_some()
    || r.inbound_tags.is_some()
    || r.port.is_some()
    || r.source_ports.is_some()
    || r.network.is_some()
    || r.protocols.is_some();
if !has_matcher {
    return None; // xray-core 26+ rejects matcher-less rules
}
```

and after the existing matcher blocks add:

```rust
if let Some(protocols) = &r.protocols {
    rule["protocol"] = json!(parse_comma_list(protocols));
}
if let Some(matcher) = &r.domain_matcher {
    rule["domainMatcher"] = json!(matcher);
}
```

singbox.rs `build_routing` — same `has_matcher` guard at the top of the `filter_map`; then:

```rust
if let Some(protocols) = &r.protocols {
    rule["protocol"] = json!(parse_comma_list(protocols));
}
if let Some(matcher) = &r.domain_matcher {
    rule["domain_matcher"] = json!(matcher);
}
```

`ops/settings.rs` `save_routing_rule` — after building `rule`, before insert/update:

```rust
let has_matcher = rule.domains.is_some()
    || rule.ips.is_some()
    || rule.inbound_tags.is_some()
    || rule.port.is_some()
    || rule.source_ports.is_some()
    || rule.network.is_some()
    || rule.protocols.is_some()
    || rule.rule_set_file.is_some()
    || rule.rule_set_url.is_some();
if !has_matcher {
    state.log_trace("error", "tui::ops::settings", "Routing rule needs at least one match condition");
    return;
}
```

(Note: rule_set_file/url count as matchers for sing-box; the xray builder does not emit rule_set fields — sing-box's rule_set emission is intentionally out of scope here; the fields remain persisted for future sing-box rule-set support. Document this in a code comment.)

- [ ] **Step 4: Run to verify they pass**

Run: `cargo test -p xray-tui-core routing_` — all PASS. Run: `cargo test -p xray-tui-core config_builder::xray config_builder::singbox` (full modules).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui/src/ops/settings.rs crates/xray-tui-core/src/config_builder/xray.rs crates/xray-tui-core/src/config_builder/singbox.rs
git commit -m "fix(routing): reject/skip matcher-less rules, emit protocols and domain_matcher"
```

### Task 5: Hysteria2 pinSHA256 wired into TlsOpts (L7)

**Files:**
- Modify: `crates/xray-tui-proto/src/proto_spec/hysteria2.rs:114-150`
- Test: `hysteria2.rs` `mod tests`

- [ ] **Step 1: Write the failing test**

In hysteria2.rs tests:

```rust
#[test]
fn pin_sha256_lands_in_tls_opts() {
    let raw = RawUrlX::from("hysteria2://secret@host:443/?pinSHA256=deadbeef...&sni=host#r");
    let config = Hysteria2Config::try_parse(&raw).expect("parse");
    let Some(TlsConfig::Tls(opts)) = config.security.tls else {
        panic!("expected Tls opts");
    };
    assert_eq!(opts.pin_sha256.as_deref(), Some("deadbeef..."));
}
```

(Match the test harness style of the file — check how existing tests construct RawUrlX and whether `security` is accessible. `pin_sha256` field exists on `TlsOpts`.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-proto pin_sha256_lands_in_tls_opts`
Expected: FAIL — `opts.pin_sha256` is None.

- [ ] **Step 3: Implement**

In `hysteria2.rs` `try_parse`: move the `pin_sha256` extraction (currently at ~:142) ABOVE the `security` construction (~:122), then use it:

```rust
let pin_sha256 = utils::query_get_multi(&query, &["pinSHA256", "pin_sha256"]).map(TinyText::from);
let security = SecurityConfig {
    tls: Some(TlsConfig::Tls(TlsOpts {
        pin_sha256,
        sni: ...,
        ...
    })),
    enc: None,
};
```

Remove the later duplicate `let pin_sha256 = ...` (keep the struct field assignment `pin_sha256` in `Ok(Self { ... })` — it now borrows the same value; adjust so the value is moved into TlsOpts and the struct field gets a clone or the same variable is reused appropriately — the struct field is also `Option<TinyText>`).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-proto pin_sha256_lands_in_tls_opts` and the full `hysteria2` test module. PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-proto/src/proto_spec/hysteria2.rs
git commit -m "fix(hysteria2): wire pinSHA256 into TlsOpts so cert pinning works"
```

---

## Phase 2: Import / Parse / Dedup

### Task 6: PlaceholderConfig gets a deterministic non-zero sig — uid never 0 (C3)

**Files:**
- Modify: `crates/xray-tui-proto/src/proto_spec/mod.rs` (`PlaceholderConfig` + `from_legacy_parse`)
- Test: `crates/xray-tui-config/src/import_export.rs` tests (behavior via `parse_share_url`) + a proto unit test

**Design (user-mandated uid model):** `uid = sig XOR cred_hash`. `sig` = deterministic hash of SEMANTIC identity (protocol, transport, security TYPE — never exact security values like pbk/sid). `cred_hash` = hash of CREDENTIAL values (uuid, pbk+sid, password). When no credentials are extractable, `cred_hash = 0` and `uid == sig`. `sig` must NEVER be zero and must ALWAYS be computable. For opaque/undecomposable configs (e.g. slipnet-enc, and our legacy `PlaceholderConfig`), `sig` = hash of the ENTIRE body. Reference: sub-healer `SlipnetEncConfig` (thirdparty/sub-healer/src/proto_spec/slipnet.rs) — but its current `compute_sig` (hash of just the schema name) is WRONG per the user: the whole body must be hashed.

- [ ] **Step 1: Write the failing tests**

proto unit test (mod.rs tests or a dedicated test):

```rust
#[test]
fn placeholder_config_sig_is_deterministic_nonzero_body_hash() {
    let blob = serde_json::json!({
        "protocol_settings": {"password": "sekrit"},
        "stream_settings": {}
    });
    let json = serde_json::to_vec(&blob).unwrap();
    let a = ProtocolConfig::from_legacy_parse("wireguard", json.clone());
    let b = ProtocolConfig::from_legacy_parse("wireguard", json.clone());
    let c = ProtocolConfig::from_legacy_parse("wireguard", serde_json::to_vec(&serde_json::json!({
        "protocol_settings": {"password": "other"},
        "stream_settings": {}
    })).unwrap());
    assert_ne!(a.sig(), 0, "sig must never be zero");
    assert_eq!(a.sig(), b.sig(), "same body -> same sig (dedup)");
    assert_ne!(a.sig(), c.sig(), "different body -> different sig");
    assert_eq!(a.cred_hash(), 0, "opaque blob has no extractable credentials");
    assert_eq!(a.uid(), a.sig(), "uid == sig when cred_hash is 0");
}
```

Behavior test (import_export.rs tests):

```rust
#[test]
fn hostless_typed_parse_failure_gets_deterministic_nonzero_uid() {
    // Hostless wireguard:// URL: legacy parser accepts, typed parser rejects.
    let url = "wireguard://publickey@?address=10.0.0.2/32";
    let settings = ValidationSettings::default();
    let a = parse_share_url(url, &settings).expect("legacy parse ok");
    let b = parse_share_url(url, &settings).expect("legacy parse ok");
    let uid_a = a.sig ^ a.cred_hash;
    let uid_b = b.sig ^ b.cred_hash;
    assert_ne!(uid_a, 0, "uid must never be zero (primary-key collapse)");
    assert_eq!(uid_a, uid_b, "same URL must dedup to the same uid");
    let url2 = "wireguard://publickey@?address=10.0.0.3/32";
    let c = parse_share_url(url2, &settings).expect("legacy parse ok");
    assert_ne!(uid_a, c.sig ^ c.cred_hash, "different configs must differ");
}
```

(Check the actual `ParsedProtocol` field names — it exposes `sig` and `cred_hash` (used by `format_share_url`); use the real names.)

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p xray-tui-proto placeholder_config_sig_is_deterministic_nonzero_body_hash`
Expected: FAIL — PlaceholderConfig sig() returns 0.
Run: `cargo test -p xray-tui-config hostless_typed_parse_failure_gets_deterministic_nonzero_uid`
Expected: FAIL — uid is 0.

- [ ] **Step 3: Implement**

`proto_spec/mod.rs` `PlaceholderConfig` — add the cache field and a real sig:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct PlaceholderConfig {
    pub proto_name: String,
    /// Opaque JSON blob containing `protocol_settings/stream_settings` from legacy parsing.
    pub settings_json: Vec<u8>,
    #[serde(skip)]
    sig_cache: std::sync::OnceLock<std::num::NonZeroU64>,
}
```

`impl ProtoSpec for PlaceholderConfig` — replace `fn sig(&self) -> u64 { 0 }` and `fn set_sig_cache(&self, _v) {}` with:

```rust
fn set_sig_cache(&self, v: std::num::NonZeroU64) {
    _ = self.sig_cache.set(v);
}
impl_sig_cache!();
```

and add a private `compute_sig` on `PlaceholderConfig` (mirror the typed configs' pattern — the `impl_sig_cache!` macro calls `self.compute_sig()`):

```rust
impl PlaceholderConfig {
    fn compute_sig(&self) -> u64 {
        // Opaque legacy blob: we cannot decompose semantic fields reliably,
        // so the sig is a deterministic rapidhash over the ENTIRE body
        // (proto_name + settings_json). Same body -> same uid (dedup); never
        // zero (mapped to NonZeroU64::MIN by the macro).
        use rapidhash::v3::{RapidStreamHasherV3, DEFAULT_RAPID_SECRETS};
        use std::hash::Hasher;
        let mut hasher = RapidStreamHasherV3::new(&DEFAULT_RAPID_SECRETS);
        hasher.write(self.proto_name.as_bytes());
        hasher.write(&self.settings_json);
        hasher.finish()
    }
}
```

`from_legacy_parse` — the `placeholder` closure must init the new field:

```rust
let placeholder = |name: &str, json: Vec<u8>| PlaceholderConfig {
    proto_name: name.to_string(),
    settings_json: json,
    sig_cache: std::sync::OnceLock::new(),
};
```

`convert_spec_blob` (import_export.rs:376-384) — the fallback code stays as-is (`profile.id = config.uid() as i64; profile.sig = config.sig() as i64; profile.cred_hash = config.cred_hash() as i64;`) — it now yields a non-zero deterministic uid. Update its comment to explain.

- [ ] **Step 4: Run to verify they pass**

Run: `cargo test -p xray-tui-proto placeholder_config_sig_is_deterministic_nonzero_body_hash` — PASS.
Run: `cargo test -p xray-tui-config hostless_typed_parse_failure_gets_deterministic_nonzero_uid` — PASS. Full proto + config suites — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-proto/src/proto_spec/mod.rs crates/xray-tui-config/src/import_export.rs
git commit -m "fix(import): PlaceholderConfig sig is a deterministic body hash (uid never 0)"
```

### Task 7: Transport host forwarded for Ws/Grpc/Http too (M16)

**Files:**
- Modify: `crates/xray-tui-proto/src/proto_spec/common.rs:125-143` (`TransportConfig::with_host`)
- Modify: `crates/xray-tui-proto/src/proto_spec/vless.rs:327-335` (reconstruct host for Ws/Grpc/Http)
- Modify: `crates/xray-tui-proto/src/proto_spec/vmess.rs` (reconstruct — check whether it emits host; add for Ws)
- Test: common.rs + vless.rs tests

- [ ] **Step 1: Write the failing tests**

common.rs:

```rust
#[test]
fn with_host_forwards_host_to_ws_grpc_http() {
    let cases = vec![
        TransportConfig::Ws(WebSocketConfig::default()),
        TransportConfig::Grpc(GrpcConfig::default()),
        TransportConfig::Http(HttpConfig::default()),
    ];
    for t in cases {
        let t = t.with_host(Some("cdn.example.com".into()), None, None);
        let host = match &t {
            TransportConfig::Ws(c) => c.host.as_deref(),
            TransportConfig::Grpc(c) => c.authority.as_deref(),
            TransportConfig::Http(c) => c.host.as_deref(),
            _ => None,
        };
        assert_eq!(host, Some("cdn.example.com"), "{t:?} keeps host");
    }
}
```

(Check field names: GrpcConfig uses `authority` not `host` — verify; if so, `with_host` must set `authority` for Grpc.)

vless.rs roundtrip test:

```rust
#[test]
fn ws_host_survives_parse_roundtrip() {
    let url = "vless://11111111-2222-3333-4444-555555555555@example.com:443?security=tls&type=ws&host=cdn.example.com&path=%2Fws#r";
    let cfg = VlessConfig::try_parse(&RawUrlX::from(url)).expect("parse");
    let out = cfg.to_url();
    assert!(out.contains("host=cdn.example.com"), "roundtrip keeps ws host: {out}");
}
```

(Check the exact `to_url` method name — vless.rs reconstruct code shown is inside a `to_url`-like fn.)

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p xray-tui-proto with_host_forwards_host_to_ws_grpc_http` — FAIL (host lost).
Run: `cargo test -p xray-tui-proto ws_host_survives_parse_roundtrip` — FAIL (host not emitted).

- [ ] **Step 3: Implement**

`common.rs` `with_host` — extend the match:

```rust
Self::Ws(cfg) => Self::Ws(WebSocketConfig {
    host: cfg.host.or(resolved),
    ..cfg
}),
Self::Grpc(cfg) => Self::Grpc(GrpcConfig {
    authority: cfg.authority.or(resolved),
    ..cfg
}),
Self::Http(cfg) => Self::Http(HttpConfig {
    host: cfg.host.or(resolved),
    ..cfg
}),
Self::HttpUpgrade(cfg) => ...existing...,
Self::XHttp(cfg) => ...existing...,
other => other,
```

`vless.rs` reconstruct (the `match &self.transport` block emitting host) — add arms:

```rust
TransportConfig::Ws(cfg) => {
    if let Some(ref host) = cfg.host
        && !should_skip_param(&self.host, host)
    {
        q.append_pair("host", host);
    }
}
TransportConfig::Grpc(cfg) => {
    if let Some(ref auth) = cfg.authority
        && !should_skip_param(&self.host, auth)
    {
        q.append_pair("host", auth);
    }
}
TransportConfig::Http(cfg) => {
    if let Some(ref host) = cfg.host
        && !should_skip_param(&self.host, host)
    {
        q.append_pair("host", host);
    }
}
```

Do the same in vmess.rs if its reconstruct has a similar match (check and mirror; if vmess.rs doesn't emit host at all, add the Ws arm only if the file already handles transports — otherwise leave vmess.rs unchanged and note it).

- [ ] **Step 4: Run to verify they pass**

Run: `cargo test -p xray-tui-proto with_host_forwards_host_to_ws_grpc_http` and `cargo test -p xray-tui-proto ws_host_survives_parse_roundtrip` — PASS. Run full proto suite — all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-proto/src/proto_spec/common.rs crates/xray-tui-proto/src/proto_spec/vless.rs crates/xray-tui-proto/src/proto_spec/vmess.rs
git commit -m "fix(proto): keep Host/authority for ws/grpc/http transports in parse and roundtrip"
```

### Task 8: PortSpec u16 overflow (M17)

**Files:**
- Modify: `crates/xray-tui-proto/src/urlx/port_spec.rs` (`length()` at ~:106, `PortSpecIter::next` at ~:185)
- Test: `port_spec.rs` tests

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn full_range_ports_do_not_overflow() {
    let mut spec = PortSpec::default();
    spec.add_range(1..=65535);
    assert_eq!(spec.length(), 65535);
    let all: Vec<u16> = spec.iter().collect();
    assert_eq!(all.len(), 65535);
    assert_eq!(all[0], 1);
    assert_eq!(*all.last().unwrap(), 65535);
}
```

(Check the actual API: `add_range` takes `Range<u16>` — 1..=65535 as `RangeInclusive` vs `Range`; the existing code uses `Range<u16>` (`range.start`/`range.end`). Use the same constructor the existing tests use, e.g. `add_range(1..65536)` is impossible for u16 — check existing test usage and adapt: if `add_range` takes `Range<u16>`, use `(1..=65535)` converted or add via `(1..65535)` plus `add(65535)`. Mirror the file's test style.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-proto full_range_ports_do_not_overflow`
Expected: FAIL (debug: overflow panic; release: wrong count/duplicates).

- [ ] **Step 3: Implement**

`length()`:

```rust
PortDecl::Range(r) => length += (u32::from(r.end) - u32::from(r.start) + 1) as usize,
```

`PortSpecIter::next` — make `inner_idx: u32` (field type) and:

```rust
PortDecl::Range(r) => {
    let port = u32::from(r.start) + self.inner_idx;
    if port <= u32::from(r.end) {
        self.inner_idx += 1;
        Some(port as u16)
    } else {
        self.outer_idx += 1;
        self.inner_idx = 0;
        self.next()
    }
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-proto full_range_ports_do_not_overflow` — PASS. Full port_spec module — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-proto/src/urlx/port_spec.rs
git commit -m "fix(port_spec): u16 overflow on full ranges (debug panic / release wrap)"
```

### Task 9: PortSpec::add_range coalesces overlapping ranges (M18)

**Files:**
- Modify: `crates/xray-tui-proto/src/urlx/port_spec.rs:68-76`
- Test: `port_spec.rs` tests

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn add_range_coalesces_overlapping_decls() {
    let mut spec = PortSpec::default();
    spec.add_range(10..=20);
    spec.add_range(30..=40);
    spec.add_range(15..=35);
    assert_eq!(spec.length(), 31, "10..=40 is 31 ports, no duplicates");
    let mut all: Vec<u16> = spec.iter().collect();
    all.sort_unstable();
    all.dedup();
    assert_eq!(all.len(), 31);
    assert_eq!(all.first(), Some(&10));
    assert_eq!(*all.last().unwrap(), 40);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-proto add_range_coalesces_overlapping_decls`
Expected: FAIL — length 26+duplicates (double-merged spans).

- [ ] **Step 3: Implement**

Replace `add_range` with a coalescing implementation:

```rust
pub fn add_range(&mut self, range: Range<u16>) {
    // Collect every decl that touches the new range, merge them all into one span.
    let mut new_start = range.start;
    let mut new_end = range.end;
    let mut removed_len = 0usize;
    let mut touched = false;
    self.ports.retain_mut(|decl| match decl {
        &mut PortDecl::Single(p) if range.contains(&p) => {
            removed_len += 1;
            new_start = new_start.min(p);
            new_end = new_end.max(p);
            touched = true;
            false
        }
        PortDecl::Range(r) if range.contains(&r.start) || range.contains(&r.end)
            || (r.start <= range.start && range.end <= r.end) => {
            removed_len += r.len();
            new_start = new_start.min(r.start);
            new_end = new_end.max(r.end);
            touched = true;
            false
        }
        _ => true,
    });
    self.ports.push(PortDecl::Range(new_start..new_end));
    self.total = self.total.saturating_sub(removed_len);
    self.total += (u32::from(new_end) - u32::from(new_start) + 1) as usize;
    let _ = touched;
}
```

Adjust to the actual field/method names (`self.total`, `PortDecl`, `range.len()` semantics) and keep `merged` behavior: when nothing touched, plain push. Keep the `PortDecl::Single` dedup behavior consistent with the existing code.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-proto add_range_coalesces_overlapping_decls` — PASS. Full port_spec module + full proto suite — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-proto/src/urlx/port_spec.rs
git commit -m "fix(port_spec): coalesce overlapping ranges in add_range (duplicates, inflated total)"
```

---

## Phase 3: Ping & Speed Test

### Task 10: One shared port allocator between pool and batch real ping (C4)

**Files:**
- Modify: `crates/xray-tui-core/src/ping/real/pool.rs` (add `port_allocator()` accessor; TTL reaper)
- Modify: `crates/xray-tui/src/ops/ping.rs` (`start_batch_ping` spawn + `dispatch_real_ping_batch` signature, ~:429)
- Modify: `crates/xray-tui/src/state.rs` (remove dead `next_real_ping_port`, ~:94/284)
- Test: pool.rs unit test (port allocator monotonic) + manual reasoning; batch wiring compiles

**Interfaces:**
- Produces: `CorePool::port_allocator(&self) -> Arc<AtomicU16>`; `dispatch_real_ping_batch(..., port_allocator: Arc<AtomicU16>, batch_active: Arc<AtomicBool>, ...)`.

- [ ] **Step 1: Write the failing test (pool allocator)**

pool.rs tests:

```rust
#[tokio::test]
async fn port_allocator_is_monotonic_and_shared() {
    let pool = CorePool::new(
        PathBuf::from("/tmp/not-used-bin"),
        PathBuf::from("/tmp/not-used-configs"),
        "127.0.0.1".to_string(),
        10800,
    );
    let a = pool.port_allocator();
    let b = pool.port_allocator();
    assert!(Arc::ptr_eq(&a, &b), "same shared counter");
    let p1 = a.fetch_add(1, Ordering::Relaxed);
    let p2 = b.fetch_add(1, Ordering::Relaxed);
    assert_eq!(p2, p1 + 1);
}
```

(The key regression test for the batch collision is integration-level; the compile-time signature change plus this monotonicity test cover the mechanism.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-core port_allocator_is_monotonic_and_shared`
Expected: FAIL — `port_allocator` does not exist.

- [ ] **Step 3: Implement**

pool.rs:

```rust
/// Shared monotonically-increasing port allocator. Batch real ping and the
/// pool both draw from this counter so they can never collide.
#[must_use]
pub fn port_allocator(&self) -> Arc<AtomicU16> {
    self.next_port.clone()
}
```

Also add a TTL reaper: a `tokio::spawn` in `CorePool::new` is not possible (new is sync) — instead make `ping()` evict a stale core after the ping completes if it exceeded TTL during the ping, OR document that TTL is lazily enforced at reuse. Minimal correct change: after `real_ping` completes in `pooled_ping`, if `pooled.last_used.elapsed() >= POOL_TTL`, stop+evict the core (frees its port promptly):

```rust
// In pooled_ping, after build_result:
{
    let mut guard = self.core.lock().await;
    if let Some(pooled) = guard.as_ref()
        && pooled.last_used.elapsed() >= POOL_TTL
    {
        let mut old = guard.take();
        if let Some(mut p) = old.take() {
            let _ = p.manager.stop().await;
        }
    }
}
```

`ops/ping.rs`:
- Add a helper to get-or-create the pool (extract from `start_real_ping`):
```rust
fn get_or_create_pool(state: &mut AppState) -> Arc<CorePool> {
    if let Some(p) = &state.core_pool {
        return p.clone();
    }
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("xray-tui");
    let bin_dir = config_dir.join("bin");
    let bin_configs_dir = config_dir.join("binConfigs");
    let proxy_addr = state.config.inbound.listen.clone();
    let base_port = state.config.inbound.socks_port;
    let pool = Arc::new(CorePool::new(bin_dir, bin_configs_dir, proxy_addr, base_port));
    state.core_pool = Some(pool.clone());
    pool
}
```
- In `start_real_ping`, replace the inline pool creation with `get_or_create_pool(state)`.
- In `start_batch_ping`'s spawn setup: create the pool BEFORE the `tokio::spawn` (via `get_or_create_pool(state)` — it's `&mut AppState`, available there), then capture `let phase2_pool = pool.clone();` into the phase-2 task, and at phase-2 start set `pool.batch_active_flag().store(true, Ordering::Relaxed)`; use a small RAII guard:

```rust
struct BatchActiveGuard(Arc<AtomicBool>);
impl Drop for BatchActiveGuard {
    fn drop(&mut self) { self.0.store(false, Ordering::Relaxed); }
}
```

- Change `dispatch_real_ping_batch` signature: replace `base_proxy_port: u16` with `port_allocator: Arc<AtomicU16>`. Replace `let mut port_counter = base_proxy_port + 1; ... let assigned_port = port_counter; port_counter += 1;` with `let assigned_port = port_allocator.fetch_add(1, Ordering::Relaxed);`.
- Update both call sites (`phase2_base_port` → pass `phase2_pool.port_allocator()`; drop `phase2_base_port`).
- Remove `state.next_real_ping_port` field and its init in `state.rs` (check no other references — grep before removing).

- [ ] **Step 4: Verify**

Run: `cargo test -p xray-tui-core port_allocator_is_monotonic_and_shared` — PASS.
Run: `cargo check -p xray-tui` — compiles (batch wiring).
Run: `cargo test -p xray-tui-core ping` — all ping tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-core/src/ping/real/pool.rs crates/xray-tui/src/ops/ping.rs crates/xray-tui/src/state.rs
git commit -m "fix(ping): single shared port allocator for pool and batch (batch/core collision)"
```

### Task 11: CorePool holds the mutex across the HTTP ping (M7)

**Files:**
- Modify: `crates/xray-tui-core/src/ping/real/pool.rs:251` (`drop(guard)` in `pooled_ping`)
- Test: logic change — covered by existing pool tests + code review; add a comment

- [ ] **Step 1: Write the failing test**

Unit-testing concurrency here is heavy; the observable contract is "two concurrent single pings on different profiles do not share a core mid-ping". Write a focused test at the `pooled_ping` level using a fake manager is not feasible (RealCoreManager is concrete). Instead: assert the source invariant — that the pool lock is held across `real_ping` — via a test that issues two concurrent `pool.ping()` calls against a MockCoreManager-backed pool and asserts both results are correct. Since `CorePool` is constructed with a `RealCoreManager` internally, mock seams do not exist.

Given the constraints, this task's verification is: (a) the code change (guard not dropped), (b) existing pool test suite passes, (c) a targeted regression test in `pool.rs` that the pooled core is not evicted between config reload and ping — skip; rely on review. Document the invariant in a comment. (If the implementer finds an existing mock seam in pool.rs tests, add the concurrency test there.)

- [ ] **Step 2: N/A (no unit test possible with current seams)**

- [ ] **Step 3: Implement**

In `pooled_ping`, remove `drop(guard);` before the `real_ping` call. The guard must stay held until after `real_ping` and `build_result`. Restructure: the `if should_reuse { ... }` block currently ends with `drop(guard); let rp_result = ...; self.build_result(...)`. Change to:

```rust
// Hold the lock across the HTTP ping: a concurrent single ping must not
// SIGHUP/reload the same core while requests are in flight, and TTL
// eviction must not kill it mid-ping.
let rp_result = crate::speed_test::real_ping(...).await;
let result = self.build_result(config_type, endpoint, rp_result);
result
```

(dropping the guard at block end, after `real_ping`). Ensure the `should_reuse == false` branch still takes/evicts correctly.

- [ ] **Step 4: Verify**

Run: `cargo test -p xray-tui-core ping` — all pass. `cargo check -p xray-tui` — compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-core/src/ping/real/pool.rs
git commit -m "fix(pool): hold core lock across real ping (concurrent reload corrupts results)"
```

### Task 12: udp_test control-plane I/O timeouts (M6)

**Files:**
- Modify: `crates/xray-tui-core/src/speed_test.rs:302-341` (`udp_test`)
- Test: `speed_test.rs` tests (unit-testable helper)

- [ ] **Step 1: Write the failing test**

The control-plane reads/writes need a timeout wrapper. Make it a small helper and test it:

```rust
/// Wrap an I/O future in `timeout`; maps timeout to SpeedTestError::Timeout.
async fn io_timeout<T>(
    timeout: Duration,
    fut: impl std::future::Future<Output = std::io::Result<T>>,
) -> Result<T, SpeedTestError> {
    match tokio::time::timeout(timeout, fut).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(SpeedTestError::Io(e)),
        Err(_) => Err(SpeedTestError::Timeout(timeout)),
    }
}
```

Test (in speed_test.rs tests):

```rust
#[tokio::test]
async fn io_timeout_maps_elapsed_to_timeout_error() {
    use std::io::Read;
    let fut = async {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        Ok::<_, std::io::Error>(())
    };
    let result = io_timeout(std::time::Duration::from_millis(50), fut).await;
    assert!(matches!(result, Err(SpeedTestError::Timeout(_))));
}
```

(Check `SpeedTestError` variants — `Timeout(Duration)` and `Io(std::io::Error)` exist per the file's existing usage; adapt if names differ.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-core io_timeout_maps_elapsed_to_timeout_error`
Expected: FAIL — `io_timeout` not defined.

- [ ] **Step 3: Implement**

Add `io_timeout` (above). Rewrite `udp_test`'s control-plane I/O to use it — every `w.write_all(...)`, `r.read_exact(...)`, including the IPv6 `extra` read:

```rust
w.write_all(&handshake).await?;      // → io_timeout(test_timeout, w.write_all(&handshake)).await?
r.read_exact(&mut response).await?;  // → io_timeout(test_timeout, r.read_exact(&mut response)).await?
// ... every control read/write ...
```

Leave the UDP data-exchange portion (after ASSOCIATE) as-is if it already uses timeouts; check the rest of the function (lines 349+) and wrap any remaining unbounded control I/O.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-core io_timeout_maps_elapsed_to_timeout_error` — PASS. `cargo test -p xray-tui-core speed_test` — all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-core/src/speed_test.rs
git commit -m "fix(speedtest): bound SOCKS5 control-plane I/O in udp_test with timeouts"
```

### Task 13: speed_test throughput math — no truncation, no false timeout (L3)

**Files:**
- Modify: `crates/xray-tui-core/src/speed_test.rs:283-289`
- Test: `speed_test.rs` tests

- [ ] **Step 1: Write the failing test**

The computation is inline in `speed_test`; extract it into a testable fn:

```rust
/// bits-per-second from bytes and elapsed. Never truncates to whole seconds;
/// a sub-second elapsed with bytes flowing is NOT a timeout.
fn throughput_bps(total_bytes: u64, elapsed: std::time::Duration) -> Option<u64> {
    if total_bytes == 0 {
        return None; // caller maps None -> Timeout
    }
    let secs = elapsed.as_secs_f64().max(0.001);
    Some((total_bytes as f64 * 8.0 / secs) as u64)
}
```

Test:

```rust
#[test]
fn throughput_uses_fractional_seconds() {
    // 1 MiB in 0.5s → ~16.7 Mbps, NOT a timeout, NOT 2x inflated.
    let bps = throughput_bps(1024 * 1024, std::time::Duration::from_millis(500)).unwrap();
    assert!((16_000_000..18_000_000).contains(&bps), "bps={bps}");
    assert!(throughput_bps(0, std::time::Duration::from_secs(5)).is_none());
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-core throughput_uses_fractional_seconds`
Expected: FAIL — `throughput_bps` not defined.

- [ ] **Step 3: Implement**

Add `throughput_bps`; in `speed_test` replace:

```rust
let elapsed = start.elapsed();
if elapsed.as_secs() == 0 {
    return Err(SpeedTestError::Timeout(max_duration));
}
let bits = total_bytes * 8;
Ok(bits / elapsed.as_secs())
```

with:

```rust
let elapsed = start.elapsed();
match throughput_bps(total_bytes, elapsed) {
    Some(bps) => Ok(bps),
    None => Err(SpeedTestError::Timeout(max_duration)),
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-core throughput_uses_fractional_seconds` — PASS. `cargo test -p xray-tui-core speed_test` — all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-core/src/speed_test.rs
git commit -m "fix(speedtest): fractional-second throughput math (no 2x inflation, no false timeout)"
```

---

## Phase 4: Profile Lifecycle & DB

### Task 14: last_seen_at bumped on connect (C5)

**Files:**
- Modify: `crates/xray-tui-db/src/database.rs:614` (`update_last_used`)
- Test: `database.rs` tests (or new db test)

- [ ] **Step 1: Write the failing test**

Find the db test module. Add (mirror existing test setup helpers — check how tests create a Database and insert endpoints):

```rust
#[tokio::test]
async fn update_last_used_also_refreshes_last_seen_at() {
    let db = Database::open_in_memory_or_temp().await.expect("db");
    // insert endpoint + protocol with last_seen_at = now - 100_000 (stale)
    let now = unix_now_secs();
    let endpoint_id = db.insert_endpoint(...).await.unwrap();
    let protocol_id = db.insert_protocol(...).await.unwrap();
    // connect: touch
    db.update_last_used(protocol_id, now).await.unwrap();
    let active = db.get_active_endpoints(now + 1).await.unwrap(); // threshold just above now
    assert!(
        active.iter().any(|r| r.endpoint.id == endpoint_id),
        "touched profile must count as active (last_seen_at refreshed)"
    );
}
```

(Adapt to the actual test helpers in database.rs — look for existing tests that insert endpoints/protocols and call get_active_endpoints.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-db update_last_used_also_refreshes_last_seen_at`
Expected: FAIL — profile not in active list (last_seen_at stale).

- [ ] **Step 3: Implement**

```rust
toasty::sql::statement(
    "UPDATE protocol_rows SET last_used_at = ?1, last_seen_at = ?1 WHERE id = ?2",
)
```

Update the doc comment: "Record when a protocol was last activated; also refreshes last_seen_at so active use keeps a profile out of the Stale/purge lists."

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-db update_last_used_also_refreshes_last_seen_at` — PASS. Full db suite — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-db/src/database.rs
git commit -m "fix(db): connect refreshes last_seen_at (daily-used profiles stay Active)"
```

### Task 15: StatsUpdate looks up the right row (M1)

**Files:**
- Modify: `crates/xray-tui/src/ops/events.rs:93-96`
- Test: logic is event-handling; verify via grep + compile. Add a helper if the lookup is extractable.

- [ ] **Step 1: Write the failing test (extract helper)**

Extract the endpoint lookup into a testable fn in events.rs (or ops/profiles.rs):

```rust
/// Find the endpoint row owning `protocol_id` (a ProtocolRow id).
pub(crate) fn endpoint_row_for_protocol<'a>(
    endpoints: &'a [EndpointRow],
    protocol_id: i64,
) -> Option<&'a EndpointRow> {
    endpoints
        .iter()
        .find(|r| r.protocols.iter().any(|p| p.id == protocol_id))
}
```

Test (in a test module in events.rs; construct two EndpointRow fixtures with protocols having distinct ids):

```rust
#[test]
fn endpoint_row_for_protocol_matches_protocol_id_not_endpoint_id() {
    // endpoint id 100 has protocol id 7; endpoint id 101 has protocol id 9
    let rows = vec![/* two EndpointRow fixtures */];
    assert_eq!(endpoint_row_for_protocol(&rows, 9).map(|r| r.endpoint.id), Some(101));
    assert!(endpoint_row_for_protocol(&rows, 999).is_none());
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui endpoint_row_for_protocol_matches_protocol_id_not_endpoint_id`
Expected: FAIL — helper not defined.

- [ ] **Step 3: Implement**

Add the helper; in the StatsUpdate handler replace:

```rust
if let Some(row) = state
    .endpoints
    .iter_mut()
    .find(|r| r.endpoint.id == protocol_id)
{
    row.stats.insert(protocol_id, stats);
}
```

with:

```rust
if let Some(row) = state
    .endpoints
    .iter_mut()
    .find(|r| r.protocols.iter().any(|p| p.id == protocol_id))
{
    row.stats.insert(protocol_id, stats);
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui endpoint_row_for_protocol_matches_protocol_id_not_endpoint_id` — PASS. `cargo check -p xray-tui` — compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui/src/ops/events.rs
git commit -m "fix(events): StatsUpdate matches protocol rows by protocol id (live traffic cell)"
```

### Task 16: delete_group removes orphaned profiles (M3)

**Files:**
- Modify: `crates/xray-tui-db/src/database.rs` (`delete_group`, ~:1009-1026)
- Modify: `crates/xray-tui/src/ops/subscriptions.rs:189` (drop `purge_expired(0)`)
- Test: database.rs tests

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn delete_group_removes_orphaned_profiles() {
    let db = ...open...;
    // group g1 with one endpoint (2 protocols), group g2 with another endpoint
    // delete g1
    db.delete_group("g1").await.unwrap();
    let all = db.get_active_endpoints(0).await.unwrap();
    assert!(!all.iter().any(|r| /* endpoint from g1 */), "g1 profiles gone");
    assert!(all.iter().any(|r| /* endpoint from g2 */), "g2 profiles remain");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-db delete_group_removes_orphaned_profiles`
Expected: FAIL — g1 profiles still in the active list.

- [ ] **Step 3: Implement**

In `delete_group`, after removing group-endpoint links and the group row, delete data for endpoints that now belong to no group (reuse the same cascade shape as `purge_expired`):

```rust
// Delete profiles of endpoints that no longer belong to any group.
toasty::sql::statement(
    "DELETE FROM profile_extensions WHERE protocol_id IN ( \
     SELECT p.id FROM protocol_rows p \
     INNER JOIN endpoints e ON e.id = p.endpoint_id \
     WHERE e.id NOT IN (SELECT DISTINCT endpoint_id FROM endpoint_groups))",
)
.exec(&mut tx).await?;
toasty::sql::statement(
    "DELETE FROM server_stats WHERE protocol_id IN ( \
     SELECT p.id FROM protocol_rows p \
     INNER JOIN endpoints e ON e.id = p.endpoint_id \
     WHERE e.id NOT IN (SELECT DISTINCT endpoint_id FROM endpoint_groups))",
)
.exec(&mut tx).await?;
toasty::sql::statement(
    "DELETE FROM protocol_rows WHERE endpoint_id NOT IN \
     (SELECT DISTINCT endpoint_id FROM endpoint_groups)",
)
.exec(&mut tx).await?;
toasty::sql::statement(
    "DELETE FROM endpoints WHERE id NOT IN \
     (SELECT DISTINCT endpoint_id FROM endpoint_groups)",
)
.exec(&mut tx).await?;
```

(`subscriptions.rs` `delete_group`: remove the `let _ = state.db.purge_expired(0).await;` line.)

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-db delete_group_removes_orphaned_profiles` — PASS. Full db suite — PASS. `cargo check -p xray-tui` — compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-db/src/database.rs crates/xray-tui/src/ops/subscriptions.rs
git commit -m "fix(subscriptions): delete_group purges orphaned profiles (was purge_expired(0) no-op)"
```

### Task 17: Edit profile uses current-view threshold (M4)

**Files:**
- Modify: `crates/xray-tui/src/ops/profiles.rs:210-214` (`start_edit_profile`)
- Test: logic fix — verify by review + compile; extract lookup if easy

- [ ] **Step 1: Write the failing test**

Extract a helper:

```rust
/// Resolve an endpoint row for editing: check the currently loaded view
/// first, then everything.
async fn find_editable_endpoint(
    state: &AppState,
    protocol_id: i64,
) -> Option<EndpointRow> {
    if let Some(r) = state.endpoints.iter().find(|r| r.endpoint.id == protocol_id) {
        return Some(r.clone());
    }
    state
        .db
        .get_active_endpoints(0)
        .await
        .ok()
        .and_then(|rows| rows.into_iter().find(|r| r.endpoint.id == protocol_id))
}
```

Test (in ops/profiles.rs tests if a test module exists; otherwise create one — AppState construction needs a lot; alternatively test the pure part): if AppState is hard to construct in tests, keep the fix minimal and verify by compile + manual reasoning, noting it in the commit. Prefer a test if a test harness exists (check for `#[cfg(test)] mod tests` in ops/profiles.rs).

- [ ] **Step 2: Run to verify it fails (if testable)**

- [ ] **Step 3: Implement**

Replace:

```rust
match state.db.get_active_endpoints(86400).await {
    Ok(rows) => {
        if let Some(_row) = rows.iter().find(|r| r.endpoint.id == protocol_id) {
```

with a lookup against the currently loaded view, falling back to all:

```rust
let found = state
    .endpoints
    .iter()
    .any(|r| r.endpoint.id == protocol_id)
    || state
        .db
        .get_active_endpoints(0)
        .await
        .is_ok_and(|rows| rows.iter().any(|r| r.endpoint.id == protocol_id));
if found {
    state.mode = AppMode::EditServer { ... };
} else {
    state.log_trace("error", "tui::ops::profiles", &format!("Profile {id} not found"));
}
```

- [ ] **Step 4: Verify**

`cargo check -p xray-tui` — compiles. If the test exists, run it.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui/src/ops/profiles.rs
git commit -m "fix(profiles): edit resolves against current view, not a 1-day threshold"
```

### Task 18: ProtocolRow endpoint_id index (L5)

**Files:**
- Modify: `crates/xray-tui-db/src/models_toasty.rs` (`ProtocolRow`)
- Verify: schema auto-migrates via toasty push_schema

- [ ] **Step 1: Write the failing test**

Performance — assert the index exists in the schema. Find how tests inspect schema (if there's a helper) or assert indirectly via EXPLAIN QUERY PLAN:

```rust
#[tokio::test]
async fn protocol_rows_are_indexed_by_endpoint_id() {
    let db = ...open...;
    let conn = db.db.connection().await.unwrap();
    let plan = toasty::sql::query(
        "EXPLAIN QUERY PLAN SELECT MAX(p.last_seen_at) FROM protocol_rows p WHERE p.endpoint_id = ?1",
    )
    .bind(1i64)
    .exec(&conn).await.unwrap();
    let text = format!("{plan:?}");
    assert!(text.contains("USING INDEX") || text.contains("endpoint_id"), "expected index use: {text}");
}
```

(Adapt to actual query API; if `db.db` is private, add the test inside the crate or use an existing accessor. If not feasible, verify via the toasty-generated schema SQL — check what `push_schema` emits and assert `CREATE INDEX ... endpoint_id`.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-db protocol_rows_are_indexed_by_endpoint_id`
Expected: FAIL — query plan uses a full scan.

- [ ] **Step 3: Implement**

Add to the `ProtocolRow` model:

```rust
#[index(endpoint_id)]
pub struct ProtocolRow { ... }
```

(Mirror the `#[index(batch_id, status, ping_type)]` syntax on PingSession.)

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-db protocol_rows_are_indexed_by_endpoint_id` — PASS. Full db suite — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-db/src/models_toasty.rs
git commit -m "perf(db): index protocol_rows.endpoint_id (profile list O(n) subqueries)"
```

---

## Phase 5: Enrichment & Utility Crates

### Task 19: host_features merge guard in EndpointInfoUpdated (M2)

**Files:**
- Modify: `crates/xray-tui-host-features/src/lib.rs` (derive PartialEq on `HostFeatures`)
- Modify: `crates/xray-tui/src/ops/events.rs:447` (guarded merge)
- Test: events.rs helper test or host-features derive test

- [ ] **Step 1: Write the failing test**

The guard logic: `only overwrite entry.host_features when the incoming value is non-default`. Extract into a pure fn in events.rs:

```rust
pub(crate) fn merge_host_features(
    current: xray_tui_host_features::HostFeatures,
    incoming: xray_tui_host_features::HostFeatures,
) -> xray_tui_host_features::HostFeatures {
    if incoming == xray_tui_host_features::HostFeatures::default() {
        current
    } else {
        incoming
    }
}
```

Test:

```rust
#[test]
fn merge_host_features_keeps_existing_when_incoming_is_default() {
    let real = xray_tui_host_features::HostFeatures {
        sni_whitelisted: true,
        ip_whitelisted: true,
        cidr_whitelisted: false,
    };
    assert_eq!(merge_host_features(real.clone(), Default::default()), real);
    let other = xray_tui_host_features::HostFeatures { sni_whitelisted: false, ip_whitelisted: false, cidr_whitelisted: true };
    assert_eq!(merge_host_features(real.clone(), other.clone()), other);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui merge_host_features_keeps_existing_when_incoming_is_default`
Expected: FAIL — `HostFeatures` has no `PartialEq` (or helper missing).

- [ ] **Step 3: Implement**

- host-features lib.rs: `#[derive(Debug, Clone, PartialEq, Eq)]` on `HostFeatures` (check current derives — add PartialEq/Eq).
- events.rs: replace `entry.host_features = info.host_features;` with `entry.host_features = merge_host_features(entry.host_features.clone(), info.host_features);` (if EndpointInfo.host_features is owned, clone appropriately).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui merge_host_features_keeps_existing_when_incoming_is_default` — PASS. `cargo test -p xray-tui-host-features` — PASS. `cargo check -p xray-tui` — compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-host-features/src/lib.rs crates/xray-tui/src/ops/events.rs
git commit -m "fix(enrich): don't clobber whitelist features with default from seed pass"
```

### Task 20: host-features ensure_file — timeout + atomic write + non-empty (M13)

**Files:**
- Modify: `crates/xray-tui-host-features/src/lib.rs:261-274` (`ensure_file`)
- Test: host-features tests

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn ensure_file_rejects_empty_download() {
    // Serve empty bytes; ensure_file must error instead of leaving a tombstone.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("whitelist.txt");
    let result = ensure_file(&path, "data:text/plain,").await; // empty body
    assert!(result.is_err(), "empty download must fail");
    assert!(!path.exists(), "no tombstone file left behind");
}

#[tokio::test]
async fn ensure_file_writes_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("whitelist.txt");
    let result = ensure_file(&path, "data:text/plain,ok-content").await;
    assert!(result.is_ok());
    assert_eq!(tokio::fs::read_to_string(&path).await.unwrap(), "ok-content");
}
```

(Check how the test module runs async tests — if tokio test is used elsewhere in the crate. Note: the real whitelist URLs are http(s); use a local test-only URL scheme or refactor `ensure_file` to take the bytes via a client abstraction. Simplest: add an internal `fn ensure_file_from_bytes(path, bytes)` and test that + a thin `ensure_file` that fetches with the client. The timeout itself is config, not unit-testable — assert `Client::builder().timeout(...)` usage by review.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-host-features ensure_file_` — FAIL (no tombstone check / non-atomic write).

- [ ] **Step 3: Implement**

```rust
/// Fetch `url` to `path` only if the file is not already present. Download is
/// bounded (30s) and written atomically (tmp + rename); empty downloads are
/// rejected so a rate-limit HTML page can never become a permanent tombstone.
async fn ensure_file(path: &Path, url: &str) -> anyhow::Result<()> {
    if path.is_file() {
        return Ok(());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let bytes = client.get(url).send().await?.error_for_status()?.bytes().await?;
    if bytes.is_empty() {
        anyhow::bail!("empty download from {url}");
    }
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        tokio::fs::create_dir_all(parent).await?;
    }
    let tmp = path.with_extension(format!("tmp{}", std::process::id()));
    tokio::fs::write(&tmp, &bytes).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-host-features ensure_file_` — PASS. Full host-features suite — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-host-features/src/lib.rs
git commit -m "fix(host-features): timeout + atomic write + non-empty check on whitelist download"
```

### Task 21: GeoIp heals corrupt mmdb, atomic download, serialized init (M14)

**Files:**
- Modify: `crates/xray-tui-geoip/src/lib.rs` (`GeoIp` + `ensure_db`)
- Test: geoip tests

- [ ] **Step 1: Write the failing test**

Refactor the reader-open into a testable piece. The healing behavior: on `Reader::open_readfile` failure, delete + re-download + retry once. Test with a corrupt file:

```rust
#[tokio::test]
async fn corrupt_db_is_healed_by_redownload() {
    // Can't hit the real 70MB URL — abstract the downloader.
    // After refactor: GeoIp::new_with_fetcher(path, fetcher) where fetcher is
    // a closure. Write garbage to path; call location_by_ip; assert the file
    // was replaced (fetcher called, content valid) or the error is surfaced
    // without leaving the corrupt file as a permanent tombstone.
}
```

Implementation plan for testability: add `fn open_reader(path: &Path) -> Result<Reader<Vec<u8>>, ...>` and `async fn fetch_bytes(url: &str) -> ...` as free fns; test `open_reader` on garbage returns Err and that `ensure_db` deletes the corrupt file on open failure before re-downloading. If full download mocking is too heavy, assert: (a) `ensure_db` skips download when file exists; (b) `location_by_ip` on a corrupt file triggers delete+redownload path (verified by the file being removed when the fetch then fails).

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-geoip corrupt_db_is_healed_by_redownload` — FAIL (current code: open error only logged/returned, file kept).

- [ ] **Step 3: Implement**

- Add `init_lock: tokio::sync::Mutex<()>` to `GeoIp` (serializes first-init download+open).
- In `location_by_ip`: hold the init lock while `ensure_db` + open run; on `Reader::open_readfile` error → `tokio::fs::remove_file(&self.db_path).await` (ignore missing) → `ensure_db().await?` (re-download) → open again.
- `ensure_db`: download to a temp path (e.g. `{db_path}.tmp{pid}`), then `tokio::fs::rename` (atomic); concurrent writers serialize via the init lock so no torn file.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-geoip corrupt_db_is_healed_by_redownload` — PASS. Full geoip suite — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-geoip/src/lib.rs
git commit -m "fix(geoip): heal corrupt mmdb, atomic download, serialize first init"
```

### Task 22: DNS resolver cache — empty/age validation, system fallback (M15)

**Files:**
- Modify: `crates/xray-tui-dns/src/lib.rs:88-131` (`get_dns_servers`)
- Test: dns tests

- [ ] **Step 1: Write the failing test**

Refactor `get_dns_servers` to separate load-from-cache and parse functions:

```rust
#[tokio::test]
async fn empty_cache_is_not_used() {
    // Write an empty cache file; get_dns_servers must re-download (or fall
    // back to system) rather than return a 0-server config.
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join(DNSCRYPT_RESOLVERS_CACHE);
    tokio::fs::create_dir_all(dir.path()).await.unwrap();
    tokio::fs::write(&cache, "").await.unwrap();
    let cfg = get_dns_servers(dir.path()).await.expect("resolver config");
    assert!(!cfg.name_servers().is_empty(), "empty cache must not yield 0 servers");
}
```

(Check the ResolverConfig API: `name_servers()` returns a slice. If the network is unavailable in CI, the re-download path must fall back to `ResolverConfig::from_system_conf()`; make the test tolerant by asserting non-empty OR that a system fallback was used. If neither is possible in the test env, gate the assertion on the fallback being present.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-dns empty_cache_is_not_used`
Expected: FAIL — empty cache yields 0 name servers.

- [ ] **Step 3: Implement**

- Extract `fn parse_dnscrypt_lines(text: &str) -> Vec<NameServerConfig>` (parses sdns:// lines → configs).
- Cache load: read file → parse; if the parsed list is EMPTY → treat as missing (re-download).
- Never write an empty cache (skip write when result empty).
- Age refresh: if cache file mtime is older than 7 days → re-download and rewrite (keep it simple: refresh inline before using the cache; failure falls back to the stale cache).
- Empty-after-download: fall back to `hickory_resolver::config::ResolverConfig::from_system_conf().ok()`.
- Keep the 10s download timeout.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-dns empty_cache_is_not_used` — PASS. Full dns suite — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-dns/src/lib.rs
git commit -m "fix(dns): empty/stale cache falls back to redownload + system resolvers"
```

---

## Phase 6: UI/UX

### Task 23: DataTable renders partially-visible rows (M8)

**Files:**
- Modify: `crates/xray-tui/src/ui/widgets/data_table.rs` (trait `render` + render loop)
- Modify: `crates/xray-tui/src/ui/profiles.rs` (`DisplayRowData::render` + `render_expansion_panel`)
- Modify: `crates/xray-tui/src/ui/logs.rs` (`LogRow::render`)
- Test: data_table.rs tests

**Interfaces:**
- Produces: `DataTableRow::render(&self, col_xs, col_widths, buf, y, clip_bottom: u16)` — rows must not write at `y_line >= clip_bottom`.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn rows_taller_than_viewport_do_not_blank_table() {
    // heights where the selected (tall) row exceeds the viewport:
    // old code produced offset == len → nothing rendered.
    let heights = vec![1u16, 1, 1, 1, 1, 10];
    let offset = compute_scroll_offset(&heights, 5, 8);
    assert!(offset < heights.len(), "offset must stay inside the row list");
}
```

(compute_scroll_offset lives in profiles.rs — test there. For the render loop clip: add a DataTable test with a fake row that records the clip_bottom it received and asserts it never exceeds the area bottom. Create a tiny test row type in data_table.rs tests implementing the trait.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui rows_taller_than_viewport_do_not_blank_table`
Expected: FAIL — offset == len.

- [ ] **Step 3: Implement**

- profiles.rs `compute_scroll_offset`: after computing `max_offset`, clamp: `let max_offset = max_offset.min(total_rows.saturating_sub(1));` (only when `!heights.is_empty()`).
- data_table.rs trait: add `clip_bottom: u16` to `DataTableRow::render`; in the render loop:
  - `if y >= content_inner.bottom() { break; }` (was `if y + rh > ...`).
  - Pass `content_inner.bottom()` as `clip_bottom`.
  - Bound the selection/multi highlight loops: `for row_y in y..y.saturating_add(rh).min(content_inner.bottom())`.
- profiles.rs `DisplayRowData::render`: guard `if y >= clip_bottom { return; }`; in `render_expansion_panel`, bound the panel block/rows to `clip_bottom` (skip lines `>= clip_bottom`; cap `Rect.height` to `clip_bottom.saturating_sub(y0)`).
- logs.rs `LogRow::render`: same guard (skip writes at `y_line >= clip_bottom`).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui rows_taller_than_viewport_do_not_blank_table` and the data_table test — PASS. `cargo check -p xray-tui` — compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui/src/ui/widgets/data_table.rs crates/xray-tui/src/ui/profiles.rs crates/xray-tui/src/ui/logs.rs
git commit -m "fix(ui): clip rows taller than viewport instead of blanking the table"
```

### Task 24: selected_index clamped after reload (M9)

**Files:**
- Modify: `crates/xray-tui/src/ops/profiles.rs` (`reload_profiles`)
- Test: extract + test clamp helper

- [ ] **Step 1: Write the failing test**

Extract:

```rust
/// Clamp the selection back into the loaded list after a reload/filter change.
pub(crate) fn clamp_selection(state: &mut AppState) {
    let len = state.filtered_profiles().count();
    if state.selected_index >= len && len > 0 {
        state.selected_index = len - 1;
    } else if len == 0 {
        state.selected_index = 0;
    }
    if state.selected_index >= len || len == 0 {
        state.selected_sub = None;
    }
}
```

Test (needs AppState construction — check if a test helper exists; if AppState is too heavy, test the pure arithmetic via a small fn `fn clamp_index(selected: usize, len: usize) -> usize`):

```rust
#[test]
fn clamp_index_stays_in_bounds() {
    assert_eq!(clamp_index(5, 3), 2);
    assert_eq!(clamp_index(2, 3), 2);
    assert_eq!(clamp_index(0, 0), 0);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui clamp_index_stays_in_bounds` — FAIL (missing).

- [ ] **Step 3: Implement**

Add `clamp_index`/`clamp_selection`; call `clamp_selection(state)` at the end of `reload_profiles` (after `state.endpoints = rows;` / `.clear();` and after `filter_cache_valid.set(false)`).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui clamp_index_stays_in_bounds` — PASS. `cargo check -p xray-tui` — compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui/src/ops/profiles.rs
git commit -m "fix(ui): clamp selected_index after reload (stale index made Enter/e/d/x no-ops)"
```

### Task 25: Footer resolves the filtered row (M10)

**Files:**
- Modify: `crates/xray-tui/src/ui/profiles.rs:637-640` (`render_footer`)
- Test: extract + test

- [ ] **Step 1: Write the failing test**

Extract a resolver:

```rust
/// The row the footer describes: the FILTERED row at `selected_index`.
pub(crate) fn footer_row<'a>(state: &'a AppState) -> Option<&'a EndpointRow> {
    state.filtered_profiles().nth(state.selected_index)
}
```

Test with AppState fixtures if feasible; otherwise test via `filtered_profiles().nth` semantics with a filter active. If AppState is hard to construct, mark this task verify-by-review (compile + manual) and note it.

- [ ] **Step 2: Run to verify it fails (if testable)**

- [ ] **Step 3: Implement**

```rust
let has_profile = state.filtered_profiles().nth(state.selected_index).is_some();

let line = if let Some(row) = footer_row(state) {
    ...
} else {
    ...
};
```

- [ ] **Step 4: Verify**

`cargo check -p xray-tui` — compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui/src/ui/profiles.rs
git commit -m "fix(ui): footer uses filtered row (wrong server shown with search filter)"
```

### Task 26: Esc closes settings from tree focus (M11)

**Files:**
- Modify: `crates/xray-tui/src/ui/settings.rs:400-443` (`handle_tree_key`)
- Test: key-handler logic — verify by review; check how Esc is handled for the right pane and mirror it

- [ ] **Step 1: (No unit test — key dispatch; verify by review + manual run)**

Check how `handle_form_key`/settings Esc works for the right pane (search for `KeyCode::Esc` in settings.rs) and mirror that exact exit behavior.

- [ ] **Step 2: Implement**

At the top of `handle_tree_key`, before the Enter handling:

```rust
if key.code == KeyCode::Esc {
    state.mode = AppMode::List;
    return;
}
```

(Confirm `AppMode::List` is the exit target — mirror the right-pane Esc handler.)

- [ ] **Step 3: Verify**

`cargo check -p xray-tui` — compiles. Manual: settings open with tree focus → Esc exits.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui/src/ui/settings.rs
git commit -m "fix(ui): Esc exits settings when the tree has focus"
```

### Task 27: Ctrl+Shift+S actually copies the share URL (L1)

**Files:**
- Modify: `crates/xray-tui/src/ui/mod.rs:743-771`
- Test: verify by review + compile (clipboard is a side effect)

- [ ] **Step 1: Implement**

Replace `let _url = state.selected_profile_id().and_then(|id| {...});` with a value-binding + clipboard write:

```rust
if let Some(url) = state.selected_profile_id().and_then(|id| {
    let row = state.filtered_profiles().find(|r| r.endpoint.id == id)?;
    let active = row.active_protocol();
    let parsed = xray_tui_config::import_export::ParsedProtocol { ... };
    xray_tui_config::import_export::format_share_url(&parsed).ok()
}) {
    match arboard::Clipboard::new() {
        Ok(mut cb) => {
            if let Err(e) = cb.set_text(url) {
                state.log_trace("error", "tui::ui", &format!("Copy failed: {e}"));
            }
        }
        Err(e) => state.log_trace("error", "tui::ui", &format!("Clipboard unavailable: {e}")),
    }
}
```

(Mirror the `arboard::Clipboard::new()` usage at :719. Note: arboard is sync and may block briefly — acceptable.)

- [ ] **Step 2: Verify**

`cargo check -p xray-tui` — compiles.

- [ ] **Step 3: Commit**

```bash
git add crates/xray-tui/src/ui/mod.rs
git commit -m "fix(ui): Ctrl+Shift+S writes share URL to clipboard (was dead binding)"
```

### Task 28: refresh_interval gates the draw (L6)

**Files:**
- Modify: `crates/xray-tui/src/ui/mod.rs:146-152` (event loop tail)
- Test: verify by review + compile (loop behavior)

- [ ] **Step 1: Implement**

```rust
let mut resize_seen = false; // declared before the loop; set true on Event::Resize in handle_event path
...
// in the loop, after poll_core_events/lazy-load/log-seek:
if resize_seen || last_tick.elapsed() >= refresh_interval {
    last_tick = std::time::Instant::now();
    resize_seen = false;
    terminal.draw(|f| render(f, &*state))?;
}
tokio::time::sleep(Duration::from_millis(16)).await;
```

- Set `resize_seen = true;` where the loop processes `Event::Resize` (the match at ~:99 — set it when `matches!(&ev, Event::Resize(_, _))`).
- Remove the old dead `if last_tick.elapsed() >= refresh_interval { last_tick = ... }` block.
- Keep the 16ms sleep (cheap wakeups; the draw was the expensive part). Update the comment to explain the gate.

- [ ] **Step 2: Verify**

`cargo check -p xray-tui` — compiles. Manual: idle CPU drops; key presses still render promptly (each loop iteration still processes events; draw happens at refresh cadence; Resize forces immediate redraw).

- [ ] **Step 3: Commit**

```bash
git add crates/xray-tui/src/ui/mod.rs
git commit -m "perf(ui): render at refresh_interval_secs instead of 60fps, force draw on resize"
```

---

## Phase 7: Core Plumbing

### Task 29: Log writer flushes on a deadline, not only on recv timeout (M12)

**Files:**
- Modify: `crates/xray-tui/src/main.rs:150-183` (writer loop)
- Test: verify by review + compile (channel timing is integration)

- [ ] **Step 1: Implement**

Restructure the writer loop so a batch flushes when its FIRST message is 500ms old regardless of recv outcome:

```rust
loop {
    // Wait for at least one message
    let msg = match log_rx.recv() {
        Ok(msg) => msg,
        Err(_) => {
            // Channel closed — flush and exit
            if !batch.is_empty() {
                let _ = writer_heed.write_log_batch(&batch);
            }
            return;
        }
    };
    let batch_deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
    batch.push(msg);
    // Drain until deadline or 100 entries
    while batch.len() < 100 {
        let remaining = batch_deadline.saturating_duration_since(std::time::Instant::now());
        match log_rx.recv_timeout(remaining) {
            Ok(msg) => batch.push(msg),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => break,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                let _ = writer_heed.write_log_batch(&batch);
                return;
            }
        }
    }
    let _ = writer_heed.write_log_batch(&batch);
    batch.clear();
}
```

This guarantees a flush at most 500ms after the first message of a batch.

- [ ] **Step 2: Verify**

`cargo check -p xray-tui` — compiles. Manual/observed: sustained log output appears in the Logs tab within ~600ms.

- [ ] **Step 3: Commit**

```bash
git add crates/xray-tui/src/main.rs
git commit -m "fix(logs): flush writer batch 500ms after first entry, not only on recv timeout"
```

### Task 30: aarch64 xray update asset URL (L2)

**Files:**
- Modify: `crates/xray-tui-core/src/updater.rs:324-336` (`release_asset_url`)
- Test: updater.rs tests

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn aarch64_xray_asset_uses_v8a_suffix() {
    let url = release_asset_url(CoreType::Xray, "1.8.0").expect("url");
    // arch is host-dependent; test the format string via a pure helper instead:
    assert_eq!(asset_name(CoreType::Xray, "aarch64"), "Xray-linux-arm64-v8a.zip");
    assert_eq!(asset_name(CoreType::SingBox, "aarch64"), "sing-box-1.8.0-linux-arm64.tar.gz");
}
```

(Extract `fn asset_name(core_type, arch) -> String` from `release_asset_url` if the current code builds names inline; mirror existing test style in updater.rs — check for a `mod tests`.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-core aarch64_xray_asset_uses_v8a_suffix`
Expected: FAIL — returns `Xray-linux-arm64.zip`.

- [ ] **Step 3: Implement**

In the arch match:

```rust
"aarch64" => match core_type {
    CoreType::Xray => "arm64-v8a", // XTLS publishes arm64-v8a, not arm64
    CoreType::SingBox => "arm64",
    CoreType::Auto => return Err(UpdateError::AutoCore),
},
```

(Verify the sing-box asset naming for aarch64 against `thirdparty/sing-box` release naming; keep the format string correct. If sing-box uses `linux-arm64`, the above is right.)

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-core aarch64_xray_asset_uses_v8a_suffix` — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-core/src/updater.rs
git commit -m "fix(updater): xray aarch64 asset is arm64-v8a (was 404)"
```

### Task 31: gRPC connect + call timeouts (L8)

**Files:**
- Modify: `crates/xray-tui-core/src/grpc_client.rs:74-103` (`GrpcStatsClient::connect`, `query_stats`, `get_sys_stats`)
- Test: grpc_client.rs tests (if any exist — check)

- [ ] **Step 1: Write the failing test (if a test harness exists)**

If the file has tests, add:

```rust
#[tokio::test]
async fn connect_to_unreachable_port_times_out_fast() {
    let t = std::time::Instant::now();
    let result = GrpcStatsClient::connect_timeout(
        "http://127.0.0.1:1", // nothing listens here
        std::time::Duration::from_millis(500),
    )
    .await;
    assert!(result.is_err());
    assert!(t.elapsed() < std::time::Duration::from_secs(5), "must fail fast");
}
```

(Requires a `connect_timeout(addr, dur)` constructor — see implementation step. If the client hardcodes API_ENDPOINT, add the addr param to the new constructor.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-core connect_to_unreachable_port_times_out_fast`
Expected: FAIL — constructor missing.

- [ ] **Step 3: Implement**

```rust
const GRPC_CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);
const GRPC_CALL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub async fn connect() -> Result<Self, GrpcError> {
    Self::connect_to(API_ENDPOINT).await
}

async fn connect_to(endpoint: &str) -> Result<Self, GrpcError> {
    let channel = tonic::transport::Endpoint::new(endpoint)
        .map_err(|e| GrpcError::Other(e.to_string()))?
        .connect_timeout(GRPC_CONNECT_TIMEOUT)
        .timeout(GRPC_CALL_TIMEOUT)
        .connect()
        .await?;
    Ok(Self { channel })
}
```

(Check the GrpcError variants — add `Other(String)` if missing, or map to an existing variant.)

Wrap the calls:

```rust
async fn query_stats(&self, pattern: &str, reset: bool) -> Result<Vec<proto::Stat>, GrpcError> {
    let mut client = ...;
    tokio::time::timeout(
        GRPC_CALL_TIMEOUT,
        client.query_stats(tonic::Request::new(proto::QueryStatsRequest { ... })),
    )
    .await
    .map_err(|_| GrpcError::Timeout(GRPC_CALL_TIMEOUT))?
    .map(|r| r.into_inner().stat)
    .await
}
```

(Adapt to the actual GrpcError enum — if no Timeout variant, add one or map to an existing variant. Same for `get_sys_stats`.)

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-core connect_to_unreachable_port_times_out_fast` — PASS (if test added). `cargo check -p xray-tui-core` — compiles. `cargo check -p xray-tui` — compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-core/src/grpc_client.rs
git commit -m "fix(grpc): timeouts on connect and per-call (wedged core no longer stalls stats loop)"
```

### Task 32: heed MapFull resize retries around active readers (L9)

**Files:**
- Modify: `crates/xray-tui-core/src/log_heed.rs:111-138` (`write_log_batch` MapFull path)
- Test: log_heed.rs tests (resize path)

- [ ] **Step 1: Write the failing test**

The current code: single resize attempt; if it fails (EBUSY because a reader holds a txn), the batch is dropped. Make the retry loop testable:

```rust
#[test]
fn resize_retries_until_success_or_exhaustion() {
    // Pure helper: attempt count / backoff decisions.
    // e.g. fn next_resize_attempt(attempts: u32) -> Option<Duration>
    assert_eq!(resize_backoff(0), Some(std::time::Duration::from_millis(50)));
    assert_eq!(resize_backoff(5), None); // give up after 5
}
```

(Extract `fn resize_backoff(attempt: u32) -> Option<Duration>`; the actual `env.resize` failure cannot be unit-tested without a full heed env — cover the loop control via the helper and review the loop.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-core resize_retries_until_success_or_exhaustion`
Expected: FAIL — helper missing.

- [ ] **Step 3: Implement**

```rust
const RESIZE_MAX_ATTEMPTS: u32 = 5;

fn resize_backoff(attempt: u32) -> Option<std::time::Duration> {
    if attempt >= RESIZE_MAX_ATTEMPTS {
        None
    } else {
        Some(std::time::Duration::from_millis(50 * (attempt as u64 + 1)))
    }
}
```

In `write_log_batch` MapFull path, replace the single resize + single retry with a loop:

```rust
Err(HeedError::MapFull) => {
    let current = self.env.info().map_size;
    let new_size = current.saturating_mul(2).min(8_589_934_592);
    if new_size <= current {
        self.mapsize_full_count.fetch_add(1, Ordering::Relaxed);
        return Ok(());
    }
    let mut attempt = 0u32;
    loop {
        match resize_backoff(attempt) {
            Some(delay) => {
                std::thread::sleep(delay); // readers are brief spawn_blocking reads
                match unsafe { self.env.resize(new_size) } {
                    Ok(()) => break,
                    Err(e) => {
                        if attempt + 1 >= RESIZE_MAX_ATTEMPTS {
                            self.mapsize_full_count.fetch_add(1, Ordering::Relaxed);
                            return Ok(()); // swallow after exhaustion (see doc comment)
                        }
                        attempt += 1;
                    }
                }
            }
            None => {
                self.mapsize_full_count.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
        }
    }
    // Retry once after resize; keep the batch if it still fails.
    if self.try_write_batch(messages).is_err() {
        self.mapsize_full_count.fetch_add(1, Ordering::Relaxed);
    }
    Ok(())
}
```

(Note: `std::thread::sleep` inside `write_log_batch` — this fn runs on spawn_blocking (async readers) or the TuiLogLayer? It is called from the background writer (spawn_blocking) and from `write_log` (spawn_blocking via async variants) — confirm callers are never on the async runtime thread; if `write_log` can be called synchronously on the TUI thread, keep the backoff short (≤250ms total). Keep the resize SAFETY comment accurate.)

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p xray-tui-core resize_retries_until_success_or_exhaustion` — PASS. `cargo test -p xray-tui-core log_heed` — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-core/src/log_heed.rs
git commit -m "fix(logs): retry heed resize with backoff (MapFull no longer drops batches)"
```

---

## Final Verification (run once after all phases)

```bash
cargo fmt --all -- --check || cargo fmt --all
cargo clippy --workspace --all-targets 2>&1 | tail -30
cargo test --workspace
cargo build --release
```

Expected: clippy warnings only pre-existing; all tests pass; release build succeeds.

## Self-Review Notes

- **Spec coverage:** All 31 findings mapped — C1..C5, M1..M19 (minus L4 which the user chose to keep), L1..L9. Tasks 1-32.
- **Dropped intentionally:** L4 (subscription_url_split) — user decision "keep current aho corasick behavior".
- **Type consistency:** `to_xray_stream_settings` (Task 2) consumed by proto mod.rs and xray.rs; `port_allocator()`/`batch_active_flag()` (Task 10) consumed by ops/ping.rs; `clamp_index`/`clamp_selection` (Task 24); `footer_row` (Task 25); `merge_host_features` (Task 19); `io_timeout`/`throughput_bps` (Tasks 12-13); `resize_backoff` (Task 32); `asset_name` (Task 30).
- **Known judgment calls** (deviations from the review, for the implementer to honor):
  - M19: rule_set_file/rule_set_url are NOT emitted into either core config (sing-box rule-set objects are out of scope); they count as matchers at save time and stay persisted. Reviewer suggested "emit or remove" — we emit protocols+domain_matcher, validate matchers, and leave rule_set fields for a future sing-box rule-set task.
  - Task 11 (M7) has no unit test seam (CorePool hardwires RealCoreManager) — verified by review + existing suite.
  - Tasks 17/25/26/27/28/29 have limited unit-testability (AppState/key-dispatch/clipboard/loop timing) — verified by compile + review + manual run where possible.
