# CONTEXT — Cellular Sheaf Database (sheaf-db)

Source: `7 Database Paradigms Breakdown - Gemini Conversation (2026-10-08_12-09-49).md` (14 turns, saved `Thu Oct 08 2026 12:09:49 GMT+0530`).
Method: `@intention-engineering` — Planning output. No implementation detail below is actionable until `SKELETON.md` gates pass.

## 1. Business goal (one paragraph)

Build a personal-project Rust database + query language that natively expresses N-dimensional nodes/edges/relations/hyper-relations via cellular sheaf theory, beating SQL/graph/document/wide-column/KV on higher-order (N>=4) queries — not by out-tuning their home turf (B-Trees, Redis hashes, columnar scans) but by changing the computation to topological consistency + sparse linear algebra ("zigzag on land", Turn 3).

## 2. Scope: in / out

In: relational, graph (1-D complex), tree (poset ideal), document (atomic payload), wide-column-sparse, KV (0-D cells) unified under one poset + stalk model; native harmonic/sheaflet search (`min x^T L_F x`) instead of Lucene/HNSW add-on.
Out: multimodal/file-blob store (Turn 2 — separate file storage); generic full-text (BM25/tokenizer/fuzzy automata) as Phase-1 target (Turn 3 — deferred, then rebuilt natively as harmonic search, not inverted index).

## 3. Hard constraints (from conversation)

- Rust only (memory layout, zero-cost, fearless concurrency). Turn 2.
- Every file <=500 lines; ~60 files across 5 crates (sheaf-distrib/sheaf-net deferred to storage-level modules per ARCHITECTURE as-built delta A4). Turn 7.
- E2E-only verification: real binary + real fixture → `.section` artifact; no mocked unit tests (skill `language-profiles/rust.md`, Turn 5/8).
- Hardware-first build order: BCSF → SIMD kernel → hand-written API + benchmark vs PG/Neo4j → TQL compiler. Never compile to an imaginary target. Turn 10.
- Dedicated vectorized kernel mandatory (not Volcano `next()->Tuple`); 1024-cell batches, L1/L2 residency; CPU→GPU offload only at subcomplex granularity. Turn 10.
- Idempotent scripts, one node = one commit with compiler + runtime evidence (skill principles #8/#9).

## 4. Math foundation (agreed, Turns 3-4 + Turn 13 fixes)

- Base: graded poset `(P, <=)`; 0-cells entities, 1-cells binary edges, k-cells N-ary hyper-relations; `σ<=τ` = incidence. Regular CW + locally Eulerian + canonical bit-parity `[σ:τ]=(-1)^i` so `δ²=0` (Turn 13 Gap 1 fix; pairwise `Δx` alone is insufficient for N>=3). `δ`/`∂` are defined only between adjacent dimensions (codim 1): a k-cell pointing directly at 0-cells is evaluated as a composite chain map through virtual codim-1 faces generated on the fly via canonical sorted-subset indexing — no intermediate cells stored, `δ²=0` preserved algebraically.
- Stalk: dual `F(σ) = (F_disc, F_cont)`; discrete = byte/bitmask/enum exact payload; continuous ≅ R^d SIMD f32 embeddings. Turn 4 Vuln 1.
- Morphism: typed `F_{σ<=τ} = (π_{σ<=τ}, W_{σ<=τ})`; `π` zero-copy mask, `W` dense/sparse col-major matrix.
- Operators: coboundary `(δx)(τ) = Σ_{σ◁τ} [σ:τ]·F_{σ<=τ}(x_σ)`; Laplacian `L_F = δ^T δ` (PSD only with Gap-1 fix); energy `E(x) = x^T L_F x + λ||x-x0||² <= ε` (soft sheaf, Turn 4 Vuln 2); cohomology `H^0 = ker δ`; solver = Block-Jacobi PCG `M=diag(L_F)`, `κ(M⁻¹L)<=15`, 8–18 iters (Turn 13 Gap 2 fix; unconditioned CG stalls).
- Storage invariant: pruned poset (maximal + demanded cells only; lazy sub-faces; sorted boundary IDs, AVX-512 intersect) — never full `O(2^N)` simplicial closure (Turn 4 Vuln 3). Down-closure `τ∈P,σ<=τ⇒σ∈P` enforced via Upper-Star tombstones, not BCSF shifts (Turn 13 Gap 3).
- Lookup invariant: attribute queries go through TIFI — indexed values as (-1)-dim virtual roots → contiguous `CellId` slice (Turn 13 Gap 4; BCSF alone is O(N) scan).
- Partition invariant: 64 MB L3 domains via KaHyPar/METIS k-way hypergraph partition + Ghost Fiber replicas; kernel never does cross-socket per-cell fetch (Turn 13 Gap 5).

## 5. Hardened architecture (Turn 4 + Turn 13)

```
TQL Compiler (parse→Eulerian-validate→TIR→E-graph π-pushdown+∂∂=0→12-opcode plan)
LSP (MemComplex lock-free/star-cascade/TIFI/MVCC + BCSFv2 64B blocks/live-masks/M⁻¹ + NUMA compactor via extend_from_reader bulk merges with streaming writes)
SIMD Kernel (FMA coboundary + BJ-PCG + certified .section emit)
Distributed (Phase-7 planned: Mayer-Vietoris covers → RAS async diffusion → cohomology-CRDT gluing → Čech-nerve routing → RDMA border reads; no hash cuts, no barriers, no 2PC, no coordinator, no hot-path RPC)
```

## 6. Hardware mapping (Turn 14, summary)

NVMe: WAL + frozen BCSF (10–50 µs, 7 GB/s seq; no traversal) → RAM dual-buffer MemComplex/BaseComplex flat arrays + NUMA/ghosts (60–100 ns) → L3 32–96 MB subcomplex → L2 ~1 MB W+M⁻¹ (~3 ns) → L1 32–48 KB boundary+bitmask (~1 ns) → AVX-512 zmm FMA coboundary/energy/PCG (~0.5–12 ns) → GPU only for whole-subcomplex sparse linalg. Budget ~30–150 ns/subcomplex vs PG 5–50 µs / Neo4j 2–10 µs.

## 7. Six-phase plan with gates (Turn 5; details in SPEC.md)

1. BCSF in-memory (<1.5× bytes, sub-15 ns lookup) → 2. LSP+ACID (4W@50k/s + 8R bulk-merged, P99 write <200 µs, cold-read P99 degradation <5% during merges — tail-gated per Part-2, never median) → 3. Kernel (10k noisy nodes → E≤1e-4, ≤25 iters, <8 ms; morph-fetch watchpoint armed) → 4. TQL frontend (50 valid <50 µs, 20 malformed pinpointed) → 5. E-graph optimizer (N=6 O(2⁶)→single slice per EXPLAIN) → 6. Head-to-head (N≥4 ≥5× QPS, P99 sub-ms; e.g. 1.8 ms vs PG 38 ms / Neo4j 52 ms — analytic comparison in PERFORMANCE.md stands in, live harness environment-blocked) → 7. Distributed (planned, SPEC-025..029: covers → Schwarz → CRDT gluing → nerve routing → RDMA; E2E `tests/e2e/phase7_dist/`).

## 8. Language + verifiable output (Turns 8/9/11)

TQL: DDL `DEFINE CELL/HYPERCELL TYPE + MORPHISM`, DML `INSERT CELL/HYPERCELL OVER BOUNDARY`, DQL `MATCH ... WHERE ... RESTRICT TO BOUNDARY ... SOLVE HARMONIC(λ,MAX_ITER,ENERGY_THRESHOLD) EMIT SECTION`. Lowers to 12-opcode plan (`BCSF_SCAN→DISCRETE_FILTER→GATHER→APPLY_MASK→VEC_GATHER→LOAD_MORPHISMS→BUILD_LAPLACIAN→SOLVE_PCG→EVAL_ENERGY→FILTER_THRESHOLD→MATERIALIZE→YIELD`); discrete bitmask chain can skip `SOLVE_PCG`.
`.section` YAML: `status, execution_time_us, subcomplex{dim,cell_count,cells}, discrete_payload, spectral_metrics{dirichlet_energy,iterations,cohomology_state}, harmonic_section{stalk_projections}`.

## 9. Top risks

R1 Orientation PSD breakage → mitigated by Eulerian validator (Gap 1). R2 Solver stall on hubs → BJ-PCG (Gap 2). R3 Delete corruption → tombstones (Gap 3). R4 O(N) attribute scans → TIFI (Gap 4). R5 NUMA stall → partition+ghosts (Gap 5). R6 Scope creep into per-cell GPU/JIT before Phase 3 gates pass → blocked by build-order gate (Turn 10). R7 Compactor evicts reader L3 → bulk merges with cache-isolated streaming writes only (Part-2). R8 Sparse-morph branches stall SpMV → watchpoint armed, stride-offset trigger on evidence (Part-2). R9 No RDMA HW on dev laptop → loopback proves protocol, 1.2µs gate HW-qualified (SPEC-029). R10 Async Schwarz divergence under partition → bounded residual exchange + energy-min gluing contains it (SPEC-026/027).

## 10. Theoretical-maximum program (2026-10-14, planned)

Exhaustive bottleneck probing (bottleneck_probe, results in results/bottleneck.json) fixed three walls: memory bandwidth (~8.4 GB/s true read; 4-thread scaling only ~1.25x, controller-capped), fsync floor (~1.2 ms; neither side syncs per op), checksum materiality (~2% of durable ops, Amdahl). Refuted two hypotheses: bounds-check cost (all variants within noise) and thread-scaling gains. Response: six proposals P1-P6 (parallel partitioned scan; validity-free nulls; incremental pre-aggregation; append-only epoch WAL headers; MPHF/branchless index; bitmask adjacency), each with hard parity gates + stretch targets, ordered P1+P3, P4, P2/P6, P5. Full proposal + falsification criteria: tests/e2e/realtime_bench/MAX_PERF_PLAN.md. New traceability: REQ-030..035 / SPEC-030..035 / METHOD-024..029 (stubs). Business goal unchanged: wins must come from structural advantages (pre-fold/stream/pre-aggregate), never from contesting physics floors.

## 11. Fiber lifecycle + write-time correctness (2026-10-20, planned)

**Trigger.** Production-readiness review of the fiber (pre-fold) layer. Measured facts: bundle build is O(dataset) per run (TPC-H 1.8 s; IMDb Q2+Q3 178.96 s = 164% of a clean release build); fibers are a frozen snapshot with **no epoch stamp anywhere in storage**; `compactor.rs` renumbers live cells (`remap_base`), so fiber keys dangle after compaction; a build racing a compaction can produce a torn fiber whose CRC still validates (no version stamp).

**Decision.** Scope A (N1-N4 + N6 + N8). Axis B (release-profile restructure) is separate and gated by a 3-part bar. Plan of record: `FIBER_LIFECYCLE_PLAN.md`. Traceability: REQ-036..042 / SPEC-036..042 / METHOD-030..037.

**Axis A (runtime bundle).** Extract the pre-fold into `crates/sheaf-engine/src/bin/shared/fiber.rs` (METHOD-030), then add a fingerprint-bound bundle cache (epoch + row counts + schema version + bundle kind): unchanged data rebuilds in <1 s, any fingerprint change is a MISS (never a stale hit), cached answers must equal cold-build answers.

**Correctness under writes.** Epoch-stamped bundles with fail-loud validation and an always-correct scan fallback (METHOD-032); per-64 MB-domain incremental maintenance so a write re-sorts only its dirty domain (METHOD-033); frozen-epoch builds under the compactor's LSM fencing (METHOD-034); a concurrent read/write stress gate of 10,000 mixed ops where the fiber answer must equal the fallback answer every time (METHOD-035). The naive scan path stays permanently as the correctness oracle.

**Axis B (compile-time, gated).** Fat-LTO clean release build measured at 109.3 s (fresh `CARGO_TARGET_DIR`; warm 0.19 s; incremental 0.8 s; release artifacts 0.05 GB). thin-LTO + `codegen-units=16` is allowed only with all three of: clean build <=55 s, bit-identical Phase-3 kernel results, identical measured-query timings across all 11 `SHELF_BINS`. Any miss closes the node as `recorded` with the profile untouched.

**Production shape (decided).** In-process only; scale horizontally by partitioning into multiple copies (Phase-6/7 cover partitioning, 64 MB L3 domains, ghost replicas, CRDT energy-merge, zero-hop nerve routing). The client/server plan was deleted; remote access would erase the ns-class wins (10.5 ns -> ~70 us over loopback) while leaving ms-class wins intact.

**Conversation record.** IMDb complex suite (Q1 filter+agg, Q2 4-table top-10, Q3 4-table rollup+HAVING) built with md5-cross-checked PG==DuckDB baselines; two silent SQL dialect divergences found by that gate (DuckDB `/` true division vs PG integer division; DuckDB `CAST(double->INT)` rounds vs PG truncates); ShelfDB fiber counterpart gates all PASS (Q1 count/votes/avg exact; Q3 all 8 decade groups exact; Q2 all 10 names+counts exact); 9-category IMDb comparison PASS (ShelfDB 143 ms; Redis 199; ScyllaDB 266; DuckDB 346; SQLite 353; Neo4j 385; MariaDB 1040; PostgreSQL 1918; MongoDB 183,619). Engineering fixes found: psycopg3 text COPY is per-line CPU-bound (474 s/GB) -> DuckDB postgres extension loader (195k rows/s); PG WAL checkpoint storm -> UNLOGGED tables; Redis cannot join (needs denormalization); Neo4j duplicate credit rows inflate pair counts 3.2x; Neo4j `count(*)` over 2-hop expansion exhausts the 4.2 GiB heap -> degree-form aggregation.
