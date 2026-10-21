# CONVERSATION_LOG_2026-10-21.md — session record: comparison repair, NoSQL challenger, fiber-bundle cache

| # | Event | Evidence | Outcome |
|---|---|---|---|
| 1 | Full 8-engine matrix run (run_all.py) | `results/RUN_SUMMARY_20261020_2116.json` | 8/8 engines OK, 10/11 shelf bins OK, 1 FAIL (fiber_snap) |
| 2 | Neo4j traversal showed 8.6 ms with paths 0 | `results/neo4j.json` run9 (counts 0/0/0) | **INVALID** — my IMDb bench had wiped the graph (`MATCH (n) DETACH DELETE n`) |
| 3 | Neo4j re-import + re-bench | 2.25M nodes / 2.1M rels, traversal 1,398.5 ms, paths 600,000 | ✅ valid number restored |
| 4 | Root cause of (2) fixed | `imdb_multi_engine_bench.py` delete now scoped to `Title`/`Person` labels | future runs cannot wipe the bench graph |
| 5 | B3 (zero-copy bundle load) built | `shared/fiber_snapshot.rs` | per-element copy removed; CRC32C instead of byte-wise IEEE CRC32 |
| 6 | B3 gate had 3 measurement bugs | see below | gate rewritten to a same-run ratio |
| 7 | MongoDB uninstalled | `scoop uninstall mongodb` + data dir deleted | ScyllaDB takes the NoSQL slot |
| 8 | ScyllaDB IMDb case built + run | `imdb_scylla_prep.rs` (Rust) + `imdb_scylla_bench.py` (CQL transport) | query **90 ms**, exact PASS, **1,991x** faster than MongoDB's 183,618.6 ms |
| 9 | Cross-dataset comparison error found | MongoDB 29.7 ms was the synthetic corpus; IMDb is 183,618.6 ms | table corrected; rule: every row names its dataset |
| 10 | B2 (IMDb sectioned bundle) built | `shared/imdb_fiber.rs` + `imdb_fiber_bench.rs` | 178.96 s build → 51.1 s once; Q1 240 ms, Q2 ~10 s, Q3 ~1.2 s; exact |
| 11 | B1 (TPCH bench wired to the cached bundle) | `fiber_join_bench.rs` cached path | 4,200 ms pre-fold → 24.8–54.5 ms (77–169x), totals exact |
| 12 | B13 gate policy + N8 model row | headers in both fiber benches; `COMPLEXITY_ANALYSIS.md` | build/query always separate; compile-time axis 109.3 s recorded; A9 closed |

## Adversarial findings (all found by gates, all recorded)

| Finding | How it surfaced | Correction |
|---|---|---|
| Reference path's CRC was **dead-code eliminated** (result unused) | ratio read 1.1–1.6x, implausible vs the earlier 158 ms load | `black_box` the CRC result; true ratio 5.4–10.4x |
| Reference re-read the file (warm cache) vs the new path (cold) | ratio swung 0.9x–1.9x between runs | share ONE read between both paths |
| Cross-run baseline (cooler box) inflated the win to "4.7–5.1x" | same-run ratio disagreed | same-run median-of-3 ratio gate; the 4.7x claim retracted |
| Pre-1995 row filter lost in a rewrite | bundle had 6,001,215 fibers instead of 2,574,528; total wrong by 2.3x | filter restored; exactness gate caught it |
| The 42.5 MB buffer **clone** was inside the timed region | verify 24.2 ms vs raw CRC 8.5 ms | clone removed; `Bundle::as_bytes` borrow |
| Last TSV field has **no trailing tab** | all `numVotes` came out 0 (Scylla case) | parse the remainder after the last separator |
| Packed records (13/20/12 B) read with an 8-byte-stride accessor | would read wrong fields silently | per-record-type accessors |
| Neo4j benchmark wiped by another bench | traversal 8.6 ms, paths 0 | label-scoped delete |
| CQL rejects range predicates on the partition key | "Only EQ and IN relation are supported" | single partition + clustering on `avg` |

## Measured results this session (B1 done, B2 done, B3 done)

| Case | Result | Gate |
|---|---|---|
| ScyllaDB IMDb Q1 | 90 ms query, 24.8 s insert, 3.3 s ETL, exact 44,906 / 294,435,444 / 8.145506 | ✅ PASS (1,991x vs MongoDB) |
| B3 bundle load | post-read work 167–301 ms → 44–80 ms (5.4–10.4x); total path 2.8–4.2x | ✅ PASS 3/3 |
| B2 IMDb bundle | sectioned, per-section CRC32C, demand-loaded | see results/imdb_shelf.json |

## Decision recorded

| Decision | Value |
|---|---|
| Deployment model | **In-process only** (no client/server) — the ns-class wins (10.5 ns) die over a socket |
| Scale-out | Partition + shard into multiple in-process copies (Phase-6/7 cover, ghosts, CRDT, zero-hop routing) |
| NoSQL challenger | ScyllaDB (CQL), MongoDB uninstalled |
| Rust-first | Heavy work in Rust; Python only for CQL transport (no Rust driver without a new crate) |
