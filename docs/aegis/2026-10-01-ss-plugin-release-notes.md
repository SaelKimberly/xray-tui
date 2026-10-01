# Release notes — SIP003 plugin support, VLESS mux, and the stored-schema reset

**Unreleased.** These three changes are independent enough to bisect separately: the plugin
feature (where mux is a hard compatibility requirement), the VLESS mux framing, and the
stored-schema reset.

## Behavior changes

- *"Shadowsocks SIP003 plugin rows (`v2ray-plugin`, `obfs-local`) now connect natively"*.

  `mode=websocket`, `obfs=http` and `obfs=tls` are served. **There is no `obfs=plain`** — the mode
  table maps only `websocket`/`ws`, `quic`, `http` and `tls`, so a row spelling anything else
  (including `plain`) is stored verbatim and refused by name; the "plain" in this project's
  obfs vocabulary belongs to ShadowsocksR, a different protocol. Plugin rows whose
  spelling we do not implement are **kept and named**, not dropped and not silently rewritten to
  a default that would dial a server neither spelling describes — such a row imports, exports,
  carries `[untestable]`, and is safe from the failed-server sweep. Plugin **UDP** is refused per
  association; there is no in-tunnel UDP for these modes.

- *"VLESS profiles that request mux are now sent over a mux-framed connection — one session per
  connection, no connection reuse yet"*.

  **Not** a restored optimization and **not** a bug fix: the row connected before and still
  connects, it simply gains framing, and the reuse that would make it cheaper is a follow-up.
  A VLESS row that carries a vision flow is **refused by name** rather than dialed, because Xray
  accepts vision only with its XUDP (port 666) multiplexer.

- *"the stored database is recreated on first launch"* (the schema wipe).

  This is the one user-visible cost in the release. The tag moves 13 → 14, which carries **no
  column change** — it exists because the SIP003 `plugin` field and the VLESS mux value are an
  identity change (`IDENTITY_VERSION` 1 → 2), which re-keys every Shadowsocks and VLESS-mux uid.
  Subscriptions and groups are config-driven and are not affected; re-importing a subscription
  rebuilds the feed.

## Not in this release

- **`mode=quic`** is stored and refused by name. Its client wire could not be pinned from
  evidence: the reference server is v2ray-plugin v1.3.2 embedding V2Ray 4.38.3, whose source is
  not in this repository, and three *mutually incompatible* QUIC wires are (v2ray-core v5.53.0,
  v2ray-core v4.31.0, and sing-box's own — which its own documentation states is not
  v2ray-core-compatible here). Shipping a guess would fail as an opaque handshake timeout.
  Previously such a row opened a TCP connection to a server answering on UDP/443; it is now
  refused before any dial, and its marker says why.
- **Plugin connection reuse.** As above: one session per connection for now.
