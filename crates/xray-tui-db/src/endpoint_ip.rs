//! `endpoint_ip` support: the sortable address key and the table's accessors.
//!
//! The table ([`crate::models_toasty::EndpointIp`]) is the single owner of a
//! DNS endpoint's resolved addresses — the JSON-array `endpoints.resolved_as`
//! column it replaces could not be indexed, joined, or ordered, and stored an
//! address list as opaque text.
//!
//! # Why not the engine's `inet` type
//!
//! Turso 0.7.2 ships `inet` as a built-in custom type,
//! `CREATE TYPE inet(value text) BASE text ENCODE validate_ipaddr(value)
//! DECODE value` (`turso_core-0.7.2/schema.rs:840`). It validates, and does
//! nothing else: with no `OPERATOR '<'` the engine refuses to order or index a
//! column of that type (`cannot ORDER BY column 'ip' of type 'inet': type does
//! not declare OPERATOR '<'`), and it is inert outside a STRICT table, which
//! `toasty` cannot emit. So the ordering lives in the *stored key* instead —
//! [`key_of`] / [`ip_of`], byte order == numeric order.
//!
//! # The key
//!
//! `[family, address-bytes…]`: `4` + 4 bytes for IPv4, `6` + 16 bytes for
//! IPv6. The family byte leads so IPv4 (`0x04…`) sorts before every IPv6
//! (`0x06…`), and within a family the big-endian bytes compare as the
//! addresses do. B-tree comparison of blobs is `memcmp`, so `ORDER BY ip_key`
//! (and an index on it) is exactly the address order.
//!
//! [`key_of`] is total over `IpAddr` and the column stores nothing else, so
//! the address and its order cannot disagree; the text form exists only for a
//! display, rendered from the bytes on read.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::error::Result;
use crate::models_toasty::{EndpointId, EndpointIp};
use toasty_core::stmt::Value;

/// Family byte of an IPv4 key: every IPv4 address sorts before every IPv6.
const FAMILY_V4: u8 = 4;
/// Family byte of an IPv6 key.
const FAMILY_V6: u8 = 6;
/// Longest key: the family byte plus 16 address octets.
const MAX_KEY_LEN: usize = 17;

/// Index the ordering key: a covering index, because toasty's `#[index]` is
/// single-column and a page's IP sort wants `(ip_key, endpoint_id)` without a
/// row lookup. Additive (`IF NOT EXISTS`), like the `endpoint_rank` indexes.
const BY_KEY_INDEX: &str =
    "CREATE INDEX IF NOT EXISTS endpoint_ip_by_key ON endpoint_ip(ip_key, endpoint_id)";

/// The sortable key of an address. Byte order is address order, IPv4 first.
#[must_use]
pub fn key_of(ip: IpAddr) -> Vec<u8> {
    let mut out = Vec::with_capacity(MAX_KEY_LEN);
    match ip {
        IpAddr::V4(v4) => {
            out.push(FAMILY_V4);
            out.extend_from_slice(&v4.octets());
        }
        IpAddr::V6(v6) => {
            out.push(FAMILY_V6);
            out.extend_from_slice(&v6.octets());
        }
    }
    out
}

/// The sortable key of a textual address, `None` when the text is not one.
///
/// The stored column is the binary key, so no write path parses text any
/// more; this stays for callers that start from a string (fixtures, a URL's
/// host field) and as the codec's textual entry point.
#[must_use]
pub fn key_of_str(text: &str) -> Option<Vec<u8>> {
    text.parse::<IpAddr>().ok().map(key_of)
}

/// The address a key encodes, in its canonical text form.
#[must_use]
pub fn ip_of(key: &[u8]) -> Option<IpAddr> {
    match key {
        [FAMILY_V4, a, b, c, d] => Some(IpAddr::V4(Ipv4Addr::new(*a, *b, *c, *d))),
        [FAMILY_V6, rest @ ..] if rest.len() == 16 => {
            let mut octets = [0u8; 16];
            octets.copy_from_slice(rest);
            Some(IpAddr::V6(Ipv6Addr::from(octets)))
        }
        _ => None,
    }
}

/// The key behind a hex string — the form a SQL aggregate can carry.
///
/// `hex()` on a BLOB is the only way the engine can hand a stored key back
/// through `group_concat` (it has no address renderer, and a raw-blob
/// `group_concat` coerces the bytes through TEXT), so the page's decoder walks
/// hex → key → [`IpAddr`]. The key is built in a stack buffer: the only
/// allocation on this path is the driver's own `Vec<u8>` for the hex string.
/// A malformed string yields `None` rather than a guess.
#[must_use]
pub fn key_from_hex(hex: &str) -> Option<IpAddr> {
    let bytes = hex.as_bytes();
    if !bytes.len().is_multiple_of(2) || bytes.len() > 2 * MAX_KEY_LEN {
        return None;
    }
    let mut key = [0u8; MAX_KEY_LEN];
    for (slot, pair) in key.iter_mut().zip(bytes.as_chunks::<2>().0) {
        let hi = (pair[0] as char).to_digit(16)?;
        let lo = (pair[1] as char).to_digit(16)?;
        *slot = u8::try_from(hi * 16 + lo).ok()?;
    }
    ip_of(&key[..bytes.len() / 2])
}

/// Create the table's covering index. Idempotent; runs at every open.
pub(crate) async fn ensure(conn: &mut impl toasty::Executor) -> Result<()> {
    toasty::sql::query(BY_KEY_INDEX).exec(conn).await?;
    Ok(())
}

/// Replace the resolved address set of `endpoint_id` with `ips`.
///
/// One transaction's worth of work — the caller owns the transaction — and a
/// full replacement, because the set is what the resolver just returned:
/// prune-what-is-gone would need the old set, and a re-resolution is rare
/// enough (TTL-gated) that rewriting one to three rows is the cheap path.
pub(crate) async fn replace(
    conn: &mut impl toasty::Executor,
    endpoint_id: EndpointId,
    ips: &[IpAddr],
) -> Result<()> {
    EndpointIp::filter_by_endpoint_id(endpoint_id)
        .delete()
        .exec(conn)
        .await?;
    // Dedup by key: the PK is (endpoint_id, ip_key), and a duplicated address
    // in one DNS answer must not be a constraint violation.
    let mut keys: Vec<Vec<u8>> = ips.iter().map(|ip| key_of(*ip)).collect();
    keys.sort_unstable();
    keys.dedup();
    for key in keys {
        // The set was just emptied, so this is a plain insert — there is no
        // value to update on conflict (`upsert` refuses a key-only model).
        toasty::create!(EndpointIp {
            endpoint_id,
            ip_key: key,
        })
        .exec(conn)
        .await?;
    }
    Ok(())
}

/// The resolved addresses of `ids`, key-ordered (the display order).
///
/// Inlined ids and one statement for the whole set: the page's typed oracle
/// and the enrich seed both read a page at a time, and the engine charges
/// ~0.8 ms per *bound* parameter — the same reason the page's hydration
/// inlines its ids (`profiles_query`).
pub(crate) async fn load(
    conn: &mut impl toasty::Executor,
    ids: &[EndpointId],
) -> Result<HashMap<EndpointId, Vec<IpAddr>>> {
    let mut out: HashMap<EndpointId, Vec<IpAddr>> = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let id_list = ids
        .iter()
        .map(|id| id.get().to_string())
        .collect::<Vec<_>>()
        .join(",");
    let rows = toasty::sql::query(format!(
        "SELECT endpoint_id, ip_key FROM endpoint_ip WHERE endpoint_id IN ({id_list}) \
         ORDER BY endpoint_id, ip_key"
    ))
    .exec(conn)
    .await?;
    for row in &rows {
        let Value::Record(record) = row else {
            continue;
        };
        let id = record.fields.first().and_then(|v| match v {
            Value::I64(n) => Some(*n),
            _ => None,
        });
        let ip = record
            .fields
            .get(1)
            .and_then(|v| match v {
                Value::Bytes(bytes) => Some(bytes.as_slice()),
                _ => None,
            })
            .and_then(ip_of);
        if let (Some(id), Some(ip)) = (id, ip) {
            out.entry(EndpointId::new(id)).or_default().push(ip);
        }
    }
    Ok(out)
}

/// Delete the addresses of `endpoint_ids`. The deletion owners call this in
/// the same transaction that removes the endpoint rows.
pub(crate) async fn delete_for(
    conn: &mut impl toasty::Executor,
    endpoint_ids: &[EndpointId],
) -> Result<()> {
    if endpoint_ids.is_empty() {
        return Ok(());
    }
    let list = endpoint_ids
        .iter()
        .map(|id| id.get().to_string())
        .collect::<Vec<_>>()
        .join(",");
    toasty::sql::query(format!(
        "DELETE FROM endpoint_ip WHERE endpoint_id IN ({list})"
    ))
    .exec(conn)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The canonical text of a stored key — what the display column holds.
    fn text_of(key: &[u8]) -> String {
        ip_of(key)
            .expect("a key this module wrote decodes")
            .to_string()
    }

    #[test]
    fn key_round_trips_every_family() {
        for text in [
            "0.0.0.0",
            "10.0.0.1",
            "255.255.255.255",
            "::",
            "::1",
            "2001:db8::1",
            "fe80::1",
            "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff",
        ] {
            let key = key_of_str(text).expect("parses");
            assert_eq!(text_of(&key), text);
        }
    }

    #[test]
    fn byte_order_is_numeric_order() {
        // The property the whole design rests on: sorting keys as bytes gives
        // the address order, with IPv4 ahead of IPv6.
        let mut ips = vec![
            "9.9.9.9",
            "10.0.0.1",
            "10.0.0.2",
            "100.0.0.1",
            "2001:db8::1",
            "::1",
            "1.2.3.4",
        ];
        ips.sort_by_key(|t| key_of_str(t).expect("parses"));
        assert_eq!(
            ips,
            vec![
                "1.2.3.4",
                "9.9.9.9",
                "10.0.0.1",
                "10.0.0.2",
                "100.0.0.1",
                "::1",
                "2001:db8::1",
            ],
            "10.0.0.1 must precede 9.9.9.9 (numeric, not lexicographic), and IPv4 must precede IPv6"
        );
    }

    #[test]
    fn bad_input_has_no_key() {
        assert!(key_of_str("").is_none());
        assert!(key_of_str("not-an-ip").is_none());
        assert!(key_of_str("10.0.0.256").is_none());
        assert!(key_of_str("10.0.0.1/24").is_none());
        assert_eq!(ip_of(&[4, 1, 2]), None);
        assert_eq!(ip_of(&[]), None);
        assert_eq!(ip_of(&[7, 1, 2, 3, 4]), None);
        assert_eq!(
            ip_of(&[6, 1, 2]),
            None,
            "a short IPv6 key is not an address"
        );
    }

    #[test]
    fn hex_round_trips_and_rejects_junk() {
        // The page's decoder reads the key back out of a `group_concat(hex())`
        // column, so this is the contract the projection depends on.
        for text in ["10.0.0.1", "2001:db8::1", "::1"] {
            let key = key_of_str(text).expect("parses");
            let mut hex = String::new();
            for byte in &key {
                use std::fmt::Write as _;
                let _ = write!(hex, "{byte:02x}");
            }
            assert_eq!(key_from_hex(&hex), Some(text.parse().expect("addr")));
            assert_eq!(
                key_from_hex(&hex.to_uppercase()),
                Some(text.parse().expect("addr")),
                "the engine's hex() is uppercase"
            );
        }
        assert_eq!(key_from_hex(""), None);
        assert_eq!(key_from_hex("abc"), None, "odd length");
        assert_eq!(key_from_hex("zz"), None, "not hex");
        // `0400000001` is NOT junk: it is the v4 key of 0.0.0.1.
        assert_eq!(
            key_from_hex("0400000001"),
            Some("0.0.0.1".parse().expect("addr"))
        );
        assert_eq!(key_from_hex("04000000"), None, "a short v4 payload");
        assert_eq!(key_from_hex("040000000000"), None, "a long v4 payload");
        assert_eq!(
            key_from_hex("0400000000000000000000000000000000000001"),
            None,
            "beyond the longest key"
        );
    }

    #[test]
    fn keys_of_the_same_address_are_identical() {
        assert_eq!(key_of_str("10.0.0.1"), key_of_str("10.0.0.1"));
        assert_eq!(
            key_of_str("2001:DB8::1"),
            key_of_str("2001:db8:0:0:0:0:0:1"),
            "text spelling must not reach the key"
        );
    }
}
