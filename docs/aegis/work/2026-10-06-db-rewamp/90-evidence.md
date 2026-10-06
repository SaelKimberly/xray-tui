# Evidence — DB rewamp

## S1 (baseline, harness guard, turso gate)

### T0 — baseline at HEAD (pre-DDL)

**Index inventory** (`/tmp/measure.db`, `user_version=14`, 74,723 endpoints /
146,744 links / 77,395 protocols / 7,721 addresses; file 121.8 MB incl. WAL):
- secondary/raw: `index_endpoint_groups_by_endpoint_id`,
  `index_endpoint_groups_by_group_id`, `endpoint_ip_by_key`,
  `endpoint_rank_band_host`, `endpoint_rank_band_window`, `endpoint_rank_test`,
  `endpoint_rank_test_v2`, `endpoint_rank_window`,
  `index_profile_stats_by_endpoint_id`, `index_profile_stats_by_last_seen_at`,
  `index_profile_stats_by_protocol_id`.
- + 10 PK autoindexes (one per table).

**App-level page baseline** (`flow_cost_report`, real feed 74,395 endpoints,
ns/op):
| path | ms |
| --- | --- |
| `profiles_page` Active **Test** @0 | 108.4 |
| … @37197 | 112.0 |
| … @74195 | 117.1 |
| `profiles_page` All Test @0 / @74368 / @148537 | 9.8 / 12.7 / 15.5 |
| `profiles_page` Active Address @0 / @74195 | 4.9 / 10.3 |
| `reload_profiles_preserving_selection` | 10.0 |
| `load_page_projection` | 3.1 |
| `refresh_endpoint_ranks` (per endpoint) | 40.8 µs |
| `band A/B`: Address filesort | 174.3 ms |
| `band A/B`: `band=0` seek (`rank_host`) | 0.16 ms |

### T1 — harness guard
`PageSort::Id` and `profiles_walk_page` have **no production caller** (grep:
`flow_cost.rs` lab + `tests/profiles_query.rs` only). Production uses
`PageSort::Test` (`ops/ping.rs` `FEED_SORT`, stream/subscription page loads) and
`PageSort::Address` (UI). The lab port folds into the T10 slice.

### T2 — turso planner gate (**PASSED**)
`crates/xray-tui-db/tests/turso_planner.rs`, direct turso 0.7.2, proposed
`endpoint_rank` shape at 74,723 rows:
| query | turso plan |
| --- | --- |
| Active `band=0 ORDER BY key` | `SEARCH … USING INDEX endpoint_rank_key (band=?)` — no sorter |
| Purgatory `band=1` | same |
| All `ORDER BY band, key` | `SCAN … USING COVERING INDEX endpoint_rank_key` — no sorter |
| All `band IN (0,1) ORDER BY key` (**trap**) | `USE SORTER FOR ORDER BY` — filesorts, confirmed |
| scope `rank_bin IN (…) ORDER BY band, key` | `SCAN … USING COVERING INDEX` |
| reband sweep | `SEARCH endpoint_rank_window (band=? AND rank_newest_seen<?)` |

The proposed index selection is confirmed on the **production engine**; only the
turso *timings* remain (to be captured after S2 with the real schema).

### Advisory
A claimed workspace/hakari self-cycle blocker is **false**: `cargo metadata`
RC=0, `cargo check -p xray-tui-db` RC=0.
