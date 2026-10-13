# Benchmark Run Log — final full evaluation

## This run

- **Completed:** 2026-10-13 21:01:34 +05:30
- **Head commit:** `3165b19` — realtime-bench: final fair-rerun verdict table (all 5 tests closed)
- **Scope:** entire evaluation per `FAIR_RERUN_PLAN.md` — 7 engines live + sheaf-db fair-rerun Tests 1–5 + real datasets
- **Machine:** i7-1165G7 / 16 GB / Windows 11, D: 159 GB free at close

## Run sequence (all timestamps IST, 2026-10-13/14)

| Time | Event |
|------|-------|
| 12:39 | `COMPARISON_MATRIX.md` written (internet numbers + first sheaf-db runs) |
| 14:33 | `PLAN.md` written (8-engine live plan) |
| 15:35–17:19 | Engine JSONs recorded: sqlite, duckdb, redis, mongodb, mariadb, pg, neo4j, scylladb |
| 17:19 | `REALTIME_RESULTS.md` first synthesis |
| 17:44 | `FAIR_RERUN_PLAN.md` created (apples-to-oranges admission) |
| ~19:00 | TPC-H `dbgen.exe` + `qgen.exe` built (MinGW); SF1 generated, counts verified exact |
| ~19:30 | SQLite live (30.8µs point / 127s join / 2.6s scan / 2.1µs insert) |
| ~19:45 | DuckDB live (410ms join / 17.9ms scan / 0.2µs batched point) |
| ~20:00 | Redis live (430µs point / 491µs Lua / 121 MB per 1M keys), removed per old lifecycle |
| ~20:10 | MongoDB live (indexed $lookup 80ms; 12s unindexed catch), removed |
| ~20:15 | MariaDB live (214µs point / 29.8s join), removed |
| ~20:20 | PostgreSQL live (108µs point / 3.04s join / 334ms scan), removed |
| ~20:30 | Neo4j live (2.69ms point / 3.3ms 2-hop / 878ms 600k-path), removed |
| ~20:35 | ScyllaDB live (11.7ms settled point / 65ms range slice), removed |
| ~20:40 | C: crisis (124 MB free) → freed 34 GB (TEMP + logs) → Docker resurrected |
| evening | Fair rerun Tests 1+4 built+run (scan 7.9ms WIN / WAL 6.9µs LOSS) |
| evening | All engines reinstalled on D: (kept installed per new rule); rematches preserved as quiet+loaded pairs |
| evening | Tests 2 (join 2.5s), 3 (traversal 41.7ms WIN), 5 (217k ops/s WIN) built+run |
| 21:01:34 | **Close: final verdict table committed (`3165b19`), tree clean** |

## Commits in this run (oldest → newest)

`7f8a804` → `8679c30` → `7b5e68c` → `640fdb1` → `ab6ec37` → `2ec89a5` → `9fbd4f0` → `18de4ec` → `4ce751b` → `3165b19`

## Final verdicts (unchanged from close)

Test 1: DuckDB wins (6.4 vs 7.9 ms) · Test 2: middle (DuckDB 158ms >> 2.5s >> PG 3.04s) · Test 3: sheaf-db wins 21× · Test 4: SQLite wins 3.3× · Test 5: sheaf-db wins 87×
