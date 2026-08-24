#!/usr/bin/env python3
"""Raw JA4-string parser + Python mirror of crypto/fingerprint/ja4.rs hashing.

Semantics (FoxIO JA4):

- A-part `{proto}13d{cipher_count:02}{ext_count:02}{alpn}`: counts exclude
  GREASE, include SNI (0000)/ALPN (0010)/padding (0015), clamped at 99.
- hash1: sha256[:12] of sorted non-GREASE cipher ids (lowercase 4-hex,
  comma-joined).
- hash2: sha256[:12] of sorted filtered extension ids + `_` + sig-alg ids
  in hello order (sig algs NOT GREASE-filtered, mirroring ja4.rs hash2);
  empty sig algs omit the `_` segment entirely.
- GREASE: `(v & 0x0f0f) == 0x0a0a`.

hash2 extension exclusions -- TWO rules exist in the wild:

- ja4db export (this catalog's data source, original FoxIO tooling):
  excludes SNI (0000) and ALPN (0010); padding (0015) stays in the hash.
  This is `ja4_hash()`'s DEFAULT because recomputed keys must join
  thirdparty/ja4db-export/csv/ja4_fingerprint.csv.
- crypto/fingerprint/ja4.rs (peet.ws semantics): additionally excludes
  padding (0015). Pass `exclude_padding=True` to reproduce the Rust file
  byte-for-byte (pinned by the tls.peet.ws known-vector selftest below).

Raw-string format (csv/ja4_fingerprint_string.csv):
    <ja4_a>[_<ciphers>[_<exts>[_<sigalgs>]]]
segments are comma-joined lowercase hex ids. Only `t13d` A-parts are
accepted (this crate's profiles all offer TLS 1.3); other prefixes in the
export (t12d/t13i/t12i/q13d) yield None.

Self-test: `python3 gen_specs.py --selftest`

Corpus quirks (investigated; see UNMATCHED breakdown printed by the
self-test). The string export and the hash export are DIFFERENT
observation sets -- a 100% set-level round-trip is impossible by
construction, not by a hashing bug:

- 46 of 998 parsed t13d strings have an A-part entirely absent from the
  hash CSV; 9 more share the A-part but a different cipher list.
- 45 more share A-part AND cipher hash but the hash CSV holds other hello
  variants' hash2 values. Proven unrecoverable: exhaustive search over
  all extension subsets x sig-alg orders/separators for the largest
  missing family (t13d09xxht/f91f431d341e) matches none of its three
  target h2 values.
- 5 hashed keys are corrupt (binary garbage instead of hex digests).
- Some rows have an empty cipher segment (`t13dNNNNNN__ext,...`);
  treated as zero ciphers.

Best achievable rates (keep-padding rule): 898/998 = 90.0% of unique
parsed strings; 264440/267622 = 98.81% observation-weighted. The
self-test asserts those floors.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import os
import re
import sys
from dataclasses import dataclass

# --- parsing -----------------------------------------------------------------

JA4_A_RE = re.compile(r"^t13d\d{2}(?:99|\d{2})(?:h2|h1|00|[a-z0-9]{2})$")


@dataclass
class RawComponents:
    ja4_a: str
    ciphers: list[int]
    exts_sorted: list[int]
    sigalgs_ordered: list[int]
    alpn_first: str


def parse_raw(s: str) -> RawComponents | None:
    """Parse a raw fingerprint string; None if not a TLS 1.3 client shape."""
    parts = s.split("_")
    if len(parts) < 2:
        return None
    ja4_a = parts[0]
    if not s.startswith("t13d") or not JA4_A_RE.match(ja4_a):
        return None
    try:
        ciphers = [int(x, 16) for x in parts[1].split(",") if x]
        exts = [int(x, 16) for x in parts[2].split(",") if x] if len(parts) > 2 else []
        sigalgs = (
            [int(x, 16) for x in parts[3].split(",") if x] if len(parts) > 3 else []
        )
    except ValueError:
        return None
    return RawComponents(ja4_a, ciphers, exts, sigalgs, ja4_a[-2:])


# --- hashing (mirror of ja4.rs) -----------------------------------------------


def is_grease(v: int) -> bool:
    return (v & 0x0F0F) == 0x0A0A


SNI, ALPN, PADDING = 0x0000, 0x0010, 0x0015


def format_ja4_a(cipher_count: int, ext_count: int, alpn: str = "00") -> str:
    """A-part with FoxIO 99-clamp (ja4.rs `ja4_a`)."""
    return f"t13d{min(cipher_count, 99):02}{min(ext_count, 99):02}{alpn}"


def _sha12(payload: str) -> str:
    return hashlib.sha256(payload.encode()).hexdigest()[:12]


def ja4_hash(rc: RawComponents, *, exclude_padding: bool = False) -> str:
    """Full FoxIO JA4: `{ja4_a}_{hash1}_{hash2}`.

    Default excludes {SNI, ALPN} from hash2 (ja4db/original-FoxIO rule).
    `exclude_padding=True` adds 0x0015, reproducing
    crypto/fingerprint/ja4.rs byte-for-byte.
    """
    excluded = {SNI, ALPN, PADDING} if exclude_padding else {SNI, ALPN}
    cs = sorted(f"{c:04x}" for c in rc.ciphers if not is_grease(c))
    es = sorted(
        f"{e:04x}" for e in rc.exts_sorted if not is_grease(e) and e not in excluded
    )
    # ja4.rs appends sig algs in hello order WITHOUT GREASE filtering.
    ss = ",".join(f"{a:04x}" for a in rc.sigalgs_ordered)
    payload = ",".join(es)
    if rc.sigalgs_ordered:
        payload += "_" + ss
    return f"{rc.ja4_a}_{_sha12(','.join(cs))}_{_sha12(payload)}"


def load_raw_rows(path: str) -> dict[str, RawComponents]:
    """Parse every row of the raw-string CSV, keyed by recomputed JA4."""
    out: dict[str, RawComponents] = {}
    with open(path, newline="", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            s = (row.get("ja4_fingerprint_string") or "").strip()
            if not s:
                continue
            rc = parse_raw(s)
            if rc is not None:
                out[ja4_hash(rc)] = rc
    return out


# --- self-test -----------------------------------------------------------------


def _selftest() -> int:
    here = os.path.dirname(os.path.abspath(__file__))
    root = os.path.abspath(os.path.join(here, "..", "..", "..", "..", ".."))
    csv_dir = os.path.join(root, "thirdparty", "ja4db-export", "csv")
    raw_csv = os.path.join(csv_dir, "ja4_fingerprint_string.csv")
    key_csv = os.path.join(csv_dir, "ja4_fingerprint.csv")

    # Unit: GREASE pattern and sha12.
    assert is_grease(0x0A0A) and is_grease(0x1A1A) and is_grease(0x1A0A)
    assert not is_grease(0x1301) and not is_grease(0x0017) and not is_grease(0x1F1F)

    # Byte-exact mirror of ja4.rs: the tls.peet.ws curl known vector
    # (`t13d3113h2_e8f1e7e78f70_db572f7c111e`), lists verbatim from the
    # Rust test module.
    peet = RawComponents(
        "t13d3113h2",
        [int(x, 16) for x in (
            "002f 0033 0035 0039 003c 003d 0067 006b 009c 009d 009e 009f"
            " 00ff 1301 1302 1303 c009 c00a c013 c014 c023 c024 c027 c028"
            " c02b c02c c02f c030 cca8 cca9 ccaa"
        ).split()],
        [0x0000, 0x000B, 0x000A, 0x3374, 0x0010, 0x0016, 0x0017, 0x0031,
         0x000D, 0x002B, 0x002D, 0x0033, 0x0015],
        [0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080A, 0x080B,
         0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301,
         0x0302, 0x0402, 0x0502, 0x0602],
        "h2",
    )
    assert ja4_hash(peet, exclude_padding=True) == (
        "t13d3113h2_e8f1e7e78f70_db572f7c111e"
    )
    # Rust `no_sig_algs_drops_trailing_segment`: ext-only payload hash.
    no_sig = RawComponents(peet.ja4_a, peet.ciphers, peet.exts_sorted, [], "h2")
    assert ja4_hash(no_sig, exclude_padding=True).split("_")[2] == "619e7cdd0224"

    # Unit: exclusion effects per rule.
    base = RawComponents("t13d0101h2", [0x1301], [0x000B], [], "h2")
    sni_alpn_only = RawComponents("t13d0101h2", [0x0A0A, 0x1301],
                                  [SNI, ALPN, 0x000B], [], "h2")
    with_padding = RawComponents("t13d0101h2", [0x1301],
                                 [SNI, ALPN, PADDING, 0x000B], [], "h2")
    assert ja4_hash(base) == ja4_hash(sni_alpn_only) != ja4_hash(with_padding)
    assert ja4_hash(base, exclude_padding=True) == ja4_hash(
        with_padding, exclude_padding=True)

    # Clamp helper (ja4.rs counts_clamp_at_99).
    assert format_ja4_a(120, 150) == "t13d999900"

    # Corpus round-trip.
    keys: set[str] = set()
    with open(key_csv, newline="", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            k = (row.get("ja4_fingerprint") or "").strip()
            if k:
                keys.add(k)

    rows_total = rows_hit = obs_total = obs_hit = 0
    distinct: dict[str, bool] = {}
    unparsed = []
    with open(raw_csv, newline="", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            s = (row.get("ja4_fingerprint_string") or "").strip()
            n = int(row.get("observation_count") or 1)
            if not s.startswith("t13d"):
                continue  # non-TLS1.3 rows are out of scope, not misses
            rc = parse_raw(s)
            if rc is None:
                unparsed.append(s)
                continue
            hit = ja4_hash(rc) in keys
            rows_total += 1
            obs_total += n
            rows_hit += hit
            obs_hit += n * hit
            distinct[s] = hit

    row_rate = rows_hit / rows_total
    obs_rate = obs_hit / obs_total
    d_hit = sum(distinct.values())
    d_rate = d_hit / len(distinct)
    print(f"parsed(t13d rows): {rows_total}  matched: {rows_hit}  "
          f"rate: {row_rate:.4%}")
    print(f"distinct strings: {len(distinct)}  matched: {d_hit}  "
          f"rate: {d_rate:.4%}")
    print(f"observation-weighted: {obs_hit}/{obs_total}  rate: {obs_rate:.4%}")
    print(f"hashed-csv unique keys: {len(keys)}  unparsed t13d rows: "
          f"{len(unparsed)}")
    if rows_hit < rows_total:
        print(f"unmatched remainder ({rows_total - rows_hit} rows / "
              f"{len(distinct) - d_hit} distinct): see module docstring "
              f"'Corpus quirks' -- export-set asymmetry, not a hash bug")

    # Floors: row-level >=99% per plan; observation-weighted and distinct
    # floors are the corpus ceilings (see docstring 'Corpus quirks').
    assert row_rate >= 0.99, f"row match rate {row_rate:.4%} < 99%"
    assert obs_rate >= 0.988, f"observation-weighted rate {obs_rate:.4%} < 98.8%"
    assert d_rate >= 0.899, f"distinct-string rate {d_rate:.4%} < 89.9%"
    print("PASS")
    return 0


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        sys.exit(_selftest())
    ap.error("nothing to do; pass --selftest")
