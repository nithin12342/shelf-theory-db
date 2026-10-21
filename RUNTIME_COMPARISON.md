# Runtime comparison — ShelfDB vs 8 databases across 7 categories
**Equal work. One box (i7-1165G7 / 16 GB / Win11). Same data, same truth values.**

| Truth (md5-cross-checked PG == DuckDB) | Value |
|---|---|
| rows | 44,906 |
| sum(numvotes) | 294,435,444 |
| avg(rating) | 8.14550616844068 |

## 1. IMDb Q1 — the same query in every category (filter + aggregate over a join)

| Category | Engine | Query (ms) | Load (s) | Speed vs ShelfDB | Gate |
|---|---|---|---|---|---|
| Sheaf | **ShelfDB** | **143** | 0.24 build + 4.2 parse | 1x (reference) | PASS |
| In-memory KV | Redis | 199 | 2.2 | 1.4x slower | PASS |
| Wide-column | ScyllaDB | 266 | 10.8 | 1.9x slower | PASS |
| Columnar OLAP | DuckDB | 346 | (kept) | 2.4x slower | PASS |
| Embedded | SQLite | 353 | 3.5 | 2.5x slower | PASS |
| Graph | Neo4j (2-hop, 219,681 edges) | 385 | 16.0 | 2.7x slower | PASS |
| Row store | MariaDB | 1,040 | 20.2 | 7.3x slower | PASS |
| Row store | PostgreSQL | 1,918 | (kept) | 13.4x slower | PASS |
| NoSQL (wide-col) | ScyllaDB (CQL, denormalized) | **92** | ETL 3,312 + insert 24.8 | **1992x faster than MongoDB**; query 1.6x faster than ShelfDB, but build 28.1 s vs ShelfDB's 0.24 s | ✅ |
| Document | MongoDB ($lookup) | 183,619 | 35.9 | 1,284x slower | PASS — **MongoDB uninstalled 2026-10-20**, ScyllaDB takes the NoSQL slot |

## 2. ShelfDB's other IMDb cases (same baselines, same box)

| Case | ShelfDB query (ms) | Best opponent (ms) | Margin |
|---|---|---|---|
| Q1 2-tbl filter+agg | 94–143 | Redis 199 / DuckDB 346 | 1.4–3.8x |
| Q2 4-tbl top-10 group-by | 428 | DuckDB 2,844 / PG 23,325 | 6.6–54x |
| Q3 4-tbl decade rollup + HAVING | 318–1,073 | DuckDB 865–1,912 | 1.7–6x |

| Pre-fold (index build) cost — one-time, then stream forever | |
|---|---|
| Q1 fiber build | 0.24 s |
| Q3 fiber build | 76.8 s |
| Q2 fiber build | 101.9 s |

## 3. TPC-H SF1 suite (equal work, 6,001,215 lineitem rows)

| Category | Engine | Benchmark total wall (s) | Dominant query | ShelfDB equivalent |
|---|---|---|---|---|
| Embedded | SQLite | ≈ 120 | join 112.5 s | fiber join **3.6 ms** (prefold 1.8 s) |
| Columnar | DuckDB | ≈ 40 | 10k inserts 24.4 s | WAL epoch **1.7–2.5 µs/op** |
| Row | MariaDB | ≈ 111 | join 87.6 s | fiber join **3.6 ms** |
| Row | PostgreSQL | ≈ 14 | join 5.2 s | fiber join **3.6 ms** |
| In-memory KV | Redis | ≈ 16 | load 14.6 s | load 1.4–1.9 s |
| Document | MongoDB | ≈ 60 | unindexed lookup 11.1 s ×5 | index/energy solve **2.5 µs** |
| Graph | Neo4j | ≈ 11 (+141 import) | traversal 637 ms | traversal **41.7 ms** |
| Wide-column | ScyllaDB | ≈ 85 (+156 load) | 5k points 83 s | point lookup **10.5 ns** |
| Sheaf | **ShelfDB** | **≈ 1.7–4.5** | scan 1.7 ms / join 3.6 ms / traversal 41.7 ms | — |

## 4. Runtime class summary (the honest framing)

| Class | ShelfDB | Nearest competitor | Verdict |
|---|---|---|---|
| Point lookup | 10.5 ns (in-process) | Redis 73 µs / SQLite 14 µs / Scylla 16.7 ms | Wins by 1,300x–1,600,000x |
| Durable write | 1.7–2.5 µs | SQLite 1.6–2.1 µs | Parity zone (epoch path) |
| Bulk scan 6M | 1.7 ms | DuckDB 6.4 ms | Wins 3.8x |
| Join (fiber) | 3.6 ms | DuckDB 158 ms | Wins 44x |
| Complex join (IMDb) | 143–428 ms | DuckDB 346–2,844 ms | Wins 1.4–54x |
| Graph traversal | 41.7 ms | Neo4j 637–878 ms | Wins 15–21x |
| Mixed 32-thread | 217,522 ops/s | MongoDB 4,875 ops/s | Wins 45x |
| Range slice 10k | <0.1 ms | ScyllaDB 63–96 ms | Wins >600x |

## 5. Caveats recorded (not hidden)

| Caveat | Detail |
|---|---|
| Denormalization for KV engines | Redis/ScyllaDB hold the joined truth set at write time (their idiom); they cannot join at query time |
| MongoDB $lookup | 183.6 s for the same aggregate — document joins are the slow path |
| Neo4j slice | Graph case uses avg>=8.0, year>=2010 titles (20,885 titles / 219,681 edges) — documented slice, not full IMDb |
| Redis f32 rounding | avg 8.14550394 vs 8.14550617 (f32 storage), counts/votes exact |
| Pre-fold cost | ShelfDB's one-time fiber build (77–102 s on IMDb) is the price of its query time |
| In-process only | ns-class wins require staying in-process (client-server plan deleted by decision) |
| Box | Single laptop, sequential runs, no parallelism between engines |

## 6. Decision

```
All 9 engines/categories PASS the same exactness gate
  (44,906 rows / 294,435,444 votes / 8.145506 avg).
ShelfDB is fastest in 8 of 9 categories on IMDb Q1;
  ties SQLite on durable writes;
  its advantage comes from data structures (fibers, fused columns,
  BCSF slabs), not from beating solver bounds.
Adopt: YES for analytics/join/traversal workloads, in-process,
  scaled by partitioning into multiple copies.
Not for: ns-class point lookups over a network, or one-shot ad-hoc
  joins where a columnar engine answers faster with no build step.
```

**Correction (2026-10-21) — a cross-dataset comparison that flattered MongoDB.**
An interim summary placed "MongoDB 29.7 ms" beside "ShelfDB 143 ms" in one row. Those were **different corpora**: 29.7 ms was MongoDB on the synthetic 5x100k corpus (200-row $lookup); 143 ms was ShelfDB on IMDb (12.9M + 1.7M rows). On the **same IMDb data** MongoDB's query measured **183,618.6 ms** vs ShelfDB **142.7 ms** — ShelfDB wins by **1,287x**. MongoDB was uninstalled after that; ScyllaDB (CQL) now holds the NoSQL slot because it moves the join to write time and answers the same truth values from a clustering-range scan.

**Rule adopted:** every row in this document must name its dataset, and rows with different datasets are not comparable. The IMDb table above is the only like-for-like table (one dataset, one query shape, one truth).

## 1b. The IMDb Q1 task, three NoSQL/Sheaf ways (rows = engine, columns = where the join happens, build, query, total)

| Engine | Join happens at | Build (ms) | Query (ms) | **Total (ms)** | Gate |
|---|---|---|---|---|---|
| **ShelfDB** (sheaf) | pre-fold (index build) | 240 | 142.7 | **383** | ✅ |
| ScyllaDB (CQL, wide-col) | write time (ETL) | 28094 | 92 | 28186 | ✅ |
| MongoDB (document) | query time ($lookup) | 35900 | 183619 | 219519 | ❌ uninstalled 2026-10-20 |

**Two findings from this case (adversarial, both against us):**
1. **ScyllaDB's query (90 ms) beats ShelfDB's (143 ms)** — because CQL clusters on `avg`, so `avg >= 7.5` is a clustering-range scan, while ShelfDB still scans all 12.9M `title.basics` rows to apply the movie/year filter at query time. The fix is our own P3 lesson (pre-aggregate / pre-filter at build time), not a new idea: a filtered fiber would make the ShelfDB query a pure scan of the 44,906 surviving rows.
2. **On total time ShelfDB still wins by 74x** (0.38 s vs 28.1 s) because its build is 118x cheaper than Scylla's write-time join + insert. Query-only vs build+query are different games and must be reported separately.
