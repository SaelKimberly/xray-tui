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


# --- attribution (verbatim from gen.py) ---------------------------------------

from ua_parser import parse  # noqa: E402

BROWSER_FAMILIES = {
    "Edge": "edge", "Edge Mobile": "edge",
    "Samsung Internet": "samsung",
    "Mobile Safari": "safari",
    "Mobile Safari UI/WKWebView": "safari",
    "Safari": "safari",
    "Firefox": "firefox",
    "Opera": "opera",
    "OPR": "opera",
    "Brave": "brave",
}
OS_FAMILIES = {"mac os x": "macos", "windows": "windows", "linux": "linux",
               "android": "android", "ios": "ios"}
# Desktop Linux shows up in ua-parser as the distribution name.
LINUX_DISTROS = {"ubuntu", "debian", "fedora", "centos", "kubuntu", "gentoo",
                 "mint", "red hat", "suse", "opensuse", "arch"}
APPLICATION_RE = re.compile(r"^(Chrome|Firefox|Safari|Edge|Brave|Opera|Samsung Internet)"
                            r"(?: ([0-9]+)(?:\.[0-9]+)?)?\s*$")


def map_browser(family):
    if family is None:
        return None
    if family in BROWSER_FAMILIES:
        return BROWSER_FAMILIES[family]
    # Chrome + Mobile/CriOS/Webview variants; Firefox and Opera families too.
    if family.startswith("Chrome") or "CriOS" in family:
        return "chrome"
    if family.startswith("Firefox"):
        return "firefox"
    if family.startswith("Opera") or family == "OPR":
        return "opera"
    return None


def map_os(family):
    """family may be None or an unrecognized name -> drop the row."""
    if not family:
        return None
    key = family.lower()
    if key in OS_FAMILIES:
        return OS_FAMILIES[key]
    # ua-parser reports modern Windows as e.g. 'Windows 10'.
    if key.startswith("windows"):
        return "windows"
    if key in LINUX_DISTROS:
        return "linux"
    return None


def derive_device(ua, device_family):
    dev = device_family or ""
    if "iPhone" in dev or "iPhone" in ua or "Mobile" in dev:
        return "phone"
    if "iPad" in dev or "Tablet" in dev or "iPad" in ua:
        return "tablet"
    if "Android" in ua and "Mobile" not in ua:
        return "tablet"
    return "desktop"


def parse_ua(ua):
    """UA string -> (browser, browser_major, os, os_major, device); None when
    unidentifiable. Majors are 0 when unknown."""
    ua = ua.strip()
    if not ua:
        return None
    r = parse(ua)
    if r.user_agent is None or r.user_agent.family is None:
        return None
    browser = map_browser(r.user_agent.family)
    if browser is None:
        return None
    if r.os is None or r.os.family is None:
        return None
    os_name = map_os(r.os.family)
    if os_name is None:
        return None

    def major(v):
        return int(v) if v and v.isdigit() else 0

    return (browser, major(r.user_agent.major), os_name,
            major(r.os.major), derive_device(ua, r.device.family if r.device else ""))


def parse_application(application):
    """Direct application field, e.g. 'Chrome 94.0'; fallback for UA-less rows."""
    m = APPLICATION_RE.match(application.strip())
    if not m:
        return None
    name = {"Samsung Internet": "samsung"}.get(m.group(1), m.group(1).lower())
    return name, int(m.group(2)) if m.group(2) else 0, "", 0, ""


# --- manifest build ------------------------------------------------------------

MANIFEST_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                             "specs_manifest.json")


def _hexlist(ids):
    return ",".join(f"{v:04x}" for v in ids)


def build_manifest(csv_dir: str) -> dict:
    """Attributed, joined, deduplicated fingerprint manifest.

    Joins the raw-string export to the label export by recomputed JA4,
    attributes each joined fingerprint to a browser/OS/device identity,
    merges identical (ja4, identity) rows and resolves same-identity
    collisions across different JA4s deterministically.
    """
    # Raw side: recomputed JA4 -> (components, observation_count).
    raw: dict[str, tuple[RawComponents, int]] = {}
    rows_in = dropped_t12 = dropped_no_ua = 0
    with open(os.path.join(csv_dir, "ja4_fingerprint_string.csv"),
              newline="", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            rows_in += 1
            s = (row.get("ja4_fingerprint_string") or "").strip()
            if not s.startswith("t13d"):
                continue  # non-TLS1.3 strings are out of scope entirely
            rc = parse_raw(s)
            if rc is None:
                continue
            try:
                n = int(row.get("observation_count") or 1)
            except ValueError:
                n = 1
            k = ja4_hash(rc)
            prev = raw.get(k)
            raw[k] = (rc, n + prev[1] if prev else n)

    # Label side: attribute + join.
    entries: dict[tuple, dict] = {}  # (ja4, identity) -> entry
    with open(os.path.join(csv_dir, "ja4_fingerprint.csv"),
              newline="", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            rows_in += 1
            ja4 = (row.get("ja4_fingerprint") or "").strip()
            if not ja4.startswith("t13d"):
                dropped_t12 += 1
                continue
            ua_field = (row.get("user_agent_string") or "").strip()
            parsed = parse_ua(ua_field) if ua_field else \
                parse_application(row.get("application") or "")
            if parsed is None or not parsed[2]:
                # application-only rows carry no OS evidence -> unattributable
                dropped_no_ua += 1
                continue
            browser, bmaj, os_name, omaj, device = parsed
            try:
                n = int(row.get("observation_count") or 1)
            except ValueError:
                n = 1
            hit = raw.get(ja4)
            if hit is None:
                continue  # no raw components on the string side; cannot join
            rc, raw_n = hit
            verified = (row.get("verified") or "").strip() == "true"
            ident = (browser, bmaj, os_name, omaj, device)
            key = (ja4, ident)
            e = entries.get(key)
            if e is None:
                entries[key] = {
                    "name": None, "browser": browser, "browser_major": bmaj,
                    "os": os_name, "os_major": omaj, "device": device,
                    "ja4": ja4, "raw": rc, "family": browser,
                    "observation_count": n + raw_n, "verified": verified,
                    "fallback": False,
                }
            else:
                e["observation_count"] += n
                e["verified"] = e["verified"] or verified

    # Collision resolution: one winner per identity across differing JA4s.
    by_ident: dict[tuple, list[dict]] = {}
    for e in entries.values():
        by_ident.setdefault((e["browser"], e["browser_major"], e["os"],
                             e["os_major"], e["device"]), []).append(e)
    winners: list[dict] = []
    collisions_dropped = 0
    for group in by_ident.values():
        group.sort(key=lambda e: (-e["observation_count"], not e["verified"],
                                  e["ja4"]))
        winners.append(group[0])
        collisions_dropped += len(group) - 1

    # Names: {browser}_{major}[_{os}_{device}], snake_case; `_2`, `_3` ...
    # disambiguate entries sharing a base name (uniqueness asserted below).
    winners.sort(key=lambda e: e["ja4"])
    used: dict[str, int] = {}
    for e in winners:
        base = f"{e['browser']}_{e['browser_major']}"
        if e["os"]:
            base += f"_{e['os']}_{e['device']}"
        seen = used.get(base, 0)
        used[base] = seen + 1
        e["name"] = base if seen == 0 else f"{base}_{seen + 1}"

    stats = {
        "rows_in": rows_in,
        "kept": len(winners),
        "dropped_no_ua": dropped_no_ua,
        "dropped_t12": dropped_t12,
        "collisions_dropped": collisions_dropped,
    }
    return {"entries": winners, "stats": stats}


def _manifest_entry_json(e: dict) -> dict:
    rc = e["raw"]
    return {
        "name": e["name"], "browser": e["browser"],
        "browser_major": e["browser_major"], "os": e["os"],
        "os_major": e["os_major"], "device": e["device"], "ja4": e["ja4"],
        "raw": {
            "ja4_a": rc.ja4_a, "ciphers": _hexlist(rc.ciphers),
            "exts_sorted": _hexlist(rc.exts_sorted),
            "sigalgs_ordered": _hexlist(rc.sigalgs_ordered),
            "alpn_first": rc.alpn_first,
        },
        "family": e["family"], "observation_count": e["observation_count"],
        "verified": e["verified"], "fallback": e["fallback"],
    }

def _write_manifest(csv_dir: str) -> int:
    import json
    manifest = build_manifest(csv_dir)
    out = {
        "entries": [_manifest_entry_json(e) for e in manifest["entries"]],
        "stats": manifest["stats"],
    }
    names = [e["name"] for e in out["entries"]]
    assert len(names) == len(set(names)), "duplicate entry names"
    assert all(e["family"] for e in out["entries"]), "entry without family"
    with open(MANIFEST_PATH, "w", encoding="utf-8") as fh:
        json.dump(out, fh, indent=1)
        fh.write("\n")
    s = out["stats"]
    print(json.dumps(s))
    print(f"wrote {MANIFEST_PATH} ({len(names)} entries)")
    assert s["kept"] >= 50, f"kept {s['kept']} < 50"
    return 0


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
    ap.add_argument("--manifest", action="store_true",
                    help="build specs_manifest.json from the ja4db export")
    args = ap.parse_args()
    if args.selftest:
        sys.exit(_selftest())
    if args.manifest:
        here = os.path.dirname(os.path.abspath(__file__))
        csv_dir = os.path.join(here, "..", "..", "..", "..", "..",
                               "thirdparty", "ja4db-export", "csv")
        sys.exit(_write_manifest(csv_dir))
    ap.error("nothing to do; pass --selftest or --manifest")
