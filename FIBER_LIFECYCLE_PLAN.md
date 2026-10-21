# FIBER_LIFECYCLE_PLAN.md — runtime bundle lifecycle + write-time correctness
**Process: intention-engineering.** Branch: `fiber-bundle-replacement` (created at the plan tip; `main` reset to `884809b` so plan commits live only on the branch).
Scope decision: **Option A** (N1–N4 + N6). Axis B (N5) is optional and gated separately.

## 1. Why this plan exists (problem statement)

| Problem | Measured evidence | Consequence if unfixed |
|---|---|---|
| Fiber bundle build is O(dataset) per bench run | IMDb Q2+Q3 = 178.96 s/run (164% of a clean release build) | Every bench cycle pays >3 min of setup |
| Fibers are a frozen snapshot | No epoch/version stamp anywhere in `sheaf-storage` | Inserts after build → **silently stale answers** (correctness, not speed) |
| Compaction renumbers live cells | `compactor.rs` `remap_base` reassigns cell ids | Fiber keys dangle after compaction → wrong rows |
| Build can race a compaction | Bundle reads base files directly | Torn build → wrong aggregates that pass CRC (no version stamp) |

## 2. N0 baseline — compile-time axis (DONE, measurement only, zero code)

| Metric | Value | Method |
|---|---|---|
| Clean release build (`--release --bins --workspace`, 37 bins, fat LTO, `codegen-units=1`) | **109.3 s** | fresh `CARGO_TARGET_DIR`, cargo 1.98.1 / rustc 1.98.1 |
| Warm incremental build (no work) | 0.19 s | same tree |
| Incremental after 1 bin touched | 0.8 s incl. process spawn | measured earlier |
| Release-only artifacts size | 0.05 GB | main `target/` = 0.87 GB incl. debug+incremental |

| Cost | Value | Share of a clean CI/clean-checkout cycle |
|---|---|---|
| Fat-LTO clean release build | 109.3 s | 100% |
| Runtime fiber bundle build (TPC-H, per run) | 1.8 s | 1.6% |
| Runtime fiber bundle build (IMDb Q2+Q3, per run) | 178.96 s | **164%** |

## 3. Two axes, different magnitudes

| Axis | Target | Current | Achievable | Risk |
|---|---|---|---|---|
| **A — runtime bundle** | IMDb fiber bundle | 178.96 s/run | <1 s (cached, fingerprint-bound) | LOW — no measured-query change |
| **B — compile-time** | clean release build | 109.3 s | ~30–40 s (thin-LTO + `codegen-units=16`) | MED — LTO was a deliberate Phase-3 kernel choice |

Axis A is worth ~18x more wall time per bench cycle than Axis B and carries no perf-regression exposure.

## 4. Node plan (each node = one atomic commit, one gate)

| Node | REQ | SPEC | Files (new/changed) | Gate (compiler + E2E evidence) | Risk |
|---|---|---|---|---|---|
| N1 / METHOD-030 | REQ-036 | SPEC-036 | NEW `crates/sheaf-engine/src/bin/shared/fiber.rs`; EDIT `fiber_join_bench.rs`, `imdb_fiber_bench.rs` | Extraction only: all existing fiber gates still PASS with identical numbers (bit-equal segment totals) | LOW |
| N2 / METHOD-031 | REQ-036 | SPEC-036 | NEW `crates/sheaf-storage/src/lsp/fiber_cache.rs` (fingerprint + bundle dir) | Same dataset twice → second bundle build **<1 s**; changed dataset → rebuild (cache MISS, never stale reuse); cached bundle answers == cold-build answers | LOW |
| N3 / METHOD-032 | REQ-037 | SPEC-037 | NEW `crates/sheaf-storage/src/lsp/fiber_epoch.rs` (epoch stamp, validate, fallback signal) | Fiber built at epoch E, loaded against epoch E+1 → **fails loud** (never answers); scan fallback returns the exact same aggregate | MED |
| N4 / METHOD-033 | REQ-038 | SPEC-038 | NEW `crates/sheaf-storage/src/bcsf/fiber_domain.rs` (per-64 MB-domain incremental update) | 1,000 writes → only the dirty domain re-sorts (measured re-sort count == dirty domains); post-write answers exact | MED |
| N5 / METHOD-034 | REQ-039 | SPEC-039 | EDIT `crates/sheaf-storage/src/lsp/compactor.rs` (expose frozen epoch) + `fiber_epoch.rs` | Build during concurrent compaction == build on a quiet box (byte-equal aggregates) | MED |
| N6 / METHOD-035 | REQ-040 | SPEC-040 | NEW `crates/sheaf-engine/src/bin/fiber_lifecycle_bench.rs`; NEW `tests/e2e/fiber_lifecycle/` | 10,000 mixed read/write ops: fiber answers == fallback answers every time; no deadlock, no corruption; lag bounded and reported | MED |
| N7 / METHOD-036 | REQ-041 | SPEC-041 | EDIT `Cargo.toml` release profile (thin-LTO, `codegen-units=16`) | **ALL THREE** required: (a) clean build ≤55 s (≥2x), (b) bit-identical Phase-3 kernel results, (c) identical measured-query timings across all 11 `SHELF_BINS`. Any miss ⇒ node closes as `recorded`, profile untouched | MED |
| N8 / METHOD-037 | REQ-042 | SPEC-042 | EDIT `COMPLEXITY_ANALYSIS.md` (compile-time axis row + A9 close-out), `RUNTIME_COMPARISON.md`, `FAIR_RERUN_PLAN.md` | (a) compile-time row present and consistent with the measured 109.3 s; (b) **fair-rerun disclosure**: `RUNTIME_COMPARISON.md` + `FAIR_RERUN_PLAN.md` state the bundle-cache amortization rule (what is counted: build, load, query) and state that a cached bundle is an **index build, not a query shortcut** — PG/DuckDB build their own indexes, so the comparison stays like-for-like | LOW |

## 5. Traceability (intention-engineering chain)

| REQ | SPEC | SOT | FOLDER | FILE | CLASS/METHOD | VERIFY |
|---|---|---|---|---|---|---|
| REQ-036 | SPEC-036 | Fiber bundle cache | `crates/sheaf-engine`, `crates/sheaf-storage` | FILE-063 `shared/fiber.rs`, FILE-067 `lsp/fiber_cache.rs` | `FiberCache::lookup_or_build`, `FiberBundle::fingerprint` | N1/N2 gates: <1 s rebuild, no stale reuse, equal answers |
| REQ-037 | SPEC-037 | Fiber epoch stamp | `crates/sheaf-storage` | FILE-068 `lsp/fiber_epoch.rs` | `FiberEpoch::stamp`, `FiberEpoch::validate`, `FiberFallback` | N3 gate: stale fiber fails loud, fallback exact |
| REQ-038 | SPEC-038 | Per-domain incremental fiber | `crates/sheaf-storage` | FILE-069 `bcsf/fiber_domain.rs` | `DomainFibers::apply_delta`, `dirty_domains` | N4 gate: only dirty domain re-sorts, answers exact |
| REQ-039 | SPEC-039 | Frozen-epoch build | `crates/sheaf-storage` | FILE-062 `lsp/compactor.rs`, FILE-068 | `Compactor::frozen_epoch` | N5 gate: concurrent build == quiet build |
| REQ-040 | SPEC-040 | Concurrent stress gate | `crates/sheaf-engine`, `tests/e2e` | FILE-070 `bin/fiber_lifecycle_bench.rs`, FILE-071 `tests/e2e/fiber_lifecycle/` | `mixed_rw_stress` | N6 gate: fiber == fallback on 10k ops |
| REQ-041 | SPEC-041 | Release-profile restructure | repo root | FILE-072 `Cargo.toml` | `[profile.release]` | N7 gate: 3-part bar |
| REQ-042 | SPEC-042 | Complexity-model row | docs | FILE-073 `COMPLEXITY_ANALYSIS.md` | compile-time axis row | N8 gate: doc consistent |

## 6. State-machine gates per node (execution discipline)

```
Planning (this doc) -> Architecture (section 4-5) -> File Design (FILE ids)
   -> Code Skeleton (signatures only) -> Implementation (one node)
   -> Compilation (cargo clippy -D warnings, zero warnings)
   -> Execution (real binary, real data) -> Verification (gate table)
   -> Accepted (one atomic commit) -> next node
```
- Every node closes only with compiler + runtime evidence attached to its commit message.
- Correction Scouting: a failing gate scopes to the node's files only; sibling nodes on the same root-cause chain are re-verified before closing.

## 7. Risks and pre-agreed outcomes

| Risk | Pre-agreed outcome |
|---|---|
| Cache serves a stale bundle | Fingerprint includes dataset epoch + row counts + schema version; any mismatch ⇒ MISS (never stale hit). Gate N2 proves it |
| Epoch stamping breaks existing fiber callers | Epoch lives in the bundle header only; absent epoch ⇒ treated as "unknown" ⇒ fallback path, never a wrong answer |
| Incremental update diverges from full rebuild | N4 gate compares incremental answers against a full rebuild on the same data; any diff ⇒ node stays open |
| Release-profile change regresses the kernel | N7's 3-part bar; any miss ⇒ profile untouched, node closed as `recorded` |
| Scope creep into a distributed fiber protocol | Out of scope for this plan; distributed fibers belong to SPEC-025..029 |

## 8. Explicit non-goals

| Non-goal | Reason |
|---|---|
| Client/server protocol | Decided: in-process only, scale by partitioning into copies (client-server plan deleted) |
| Cross-machine fiber replication | Belongs to SPEC-025..029 (distributed phase) |
| GPU fiber build | Not on the critical path; memory wall binds anyway |
| Replacing the naive scan path | It stays as the always-correct fallback and the correctness oracle for every gate |

## 9. Definition of done for this plan

| Done when | Evidence |
|---|---|
| Bundle rebuild <1 s on unchanged data | N2 gate output in commit message |
| Stale fiber never answers | N3 gate: stale load fails loud, fallback exact |
| Writes update fibers per domain, not by full rebuild | N4 gate: re-sort count == dirty domains |
| Builds are snapshot-consistent | N5 gate: concurrent == quiet byte-equal |
| Read+write at volume stays correct | N6 gate: 10k mixed ops, fiber == fallback |
| Compile-time claim either met or recorded | N7 gate: 3-part bar, outcome stated |
| Docs/model consistent | N8 row present |

---

# Part B — parallel-session additions (appended 2026-10-20; nothing in Part A discarded)

| Field | Value |
|---|---|
| Node inventory | **23 slots** = N0–N8 (9) + B1–B14 (14) ⇒ **22 planned + 1 done** (N0) |
| Refuted (not built) | **6** (B.6 rows) |
| Source | Parallel session `ses_ea721b69dffd2u1VbzMJskNncf` ("Fiber Bundle Build Avoidance Plan") |
| Merge rule | **Append only.** Part A (N0–N8) stands unchanged. |
| ID rule | Part A already consumes REQ-036..042 / SPEC-036..042 / METHOD-030..037 / FILE-063..073. The parallel session's own REQ-036..041 carry *different* meanings, so its items are appended below with **fresh** ids: REQ-043..048 / SPEC-043..048 / METHOD-038..043 / FILE-074..079. |
| Conflict note | The two plans' `N1`–`N5` labels are **not** the same nodes. The mapping table below is the reconciliation; both are kept, nothing overwritten. |

## B.1 Reconciliation of the two node sets

| Parallel node | Parallel ids | This plan's counterpart | Relationship |
|---|---|---|---|
| N1 shared fiber pre-fold module | REQ-036 / METHOD-030 | **N1 / METHOD-030** (Part A) | **Identical** — dedupe the 3 copies of the pre-fold into `shared/fiber.rs` |
| N2 snapshot codec + source fingerprint | REQ-037 / METHOD-031 | **N2 / METHOD-031** (Part A: fingerprint-bound cache) | **Same intent, Part A is a superset** (fingerprint + epoch stamp + CRC) |
| N3 TPCH bins wired to the cached bundle | REQ-038 / METHOD-032 | — | **Appended as B1** (new METHOD-038) |
| N4 IMDb fiber bundle (Q1/Q2/Q3) | REQ-039 / METHOD-033 | — | **Appended as B2** (new METHOD-039) |
| N5 release-profile restructure | REQ-040 / METHOD-034 | **N7 / METHOD-036** (Part A) | **Same goal, different id** — Part A's bar is stricter (3-part: ≤55 s + bit-identical kernel + identical timings). Part A wins; B keeps a cross-reference. |
| N6 docs + disclosure | REQ-041 / METHOD-035 | **N6 + N8 / METHOD-035 + METHOD-037** | Same — Part A covers it. |
| *(new)* zero-copy (mmap) snapshot load | — | — | **Appended as B3** (new METHOD-040) |
| *(new)* versioned tombstone table | — | — | **Appended as B4** (new METHOD-041) |
| *(new)* shadow-paging compaction (reader chain) | — | — | **Appended as B5** (new METHOD-042) |
| *(new)* SoA as primary stalk layout | — | — | **Appended as B6** (new METHOD-043) |
| *(new)* SKELETON drift fix (7 untracked library files) | — | — | **Appended as B7** (new METHOD-044) |

## B.2 Appended nodes (Part B)

| Node | REQ | SPEC | Files | Change | Gate (evidence, not claim) |
|---|---|---|---|---|---|
| **B1** / METHOD-038 | REQ-043 | SPEC-043 | EDIT `fiber_join_bench.rs` (+ `shared/fiber_snapshot.rs` codec) | **TPCH fiber bench uses the cached bundle.** `fiber_join_bench` re-sorted 6M+1.5M+150k rows (4.2 s) on EVERY run. It now opens the bundle written by the B3 codec (same payload layout: [nf][(cust,rev)*nf][ns][(cust,seg)*ns]), walks it with the same streaming merge-walk, and writes the bundle when it is missing (so the first run builds it, later runs skip the build). | **GREEN (2026-10-21):** cached path **24.8–54.5 ms** total (load 21–51 ms + query 3.5 ms) vs the 4,200 ms pre-fold ⇒ **77–169x**; totals exact (93,574,920,897, rel 4.38e-12); gate total ≤ 500 ms; clippy clean. |
| **B2** / METHOD-039 | REQ-044 | SPEC-044 | NEW `crates/sheaf-engine/src/bin/shared/imdb_fiber.rs`; REWRITE `imdb_fiber_bench.rs` | **IMDb fiber bundle, sectioned.** The 178.96 s build (principals 102M + names 15.7M rows) is written ONCE into a sectioned container: header (magic, version, n_sections, index_len, index CRC32C) + index (name, offset, len, CRC32C per section) + payload. Sections: `basics` (id u64, year i32, flags u8 — 12,866,375 rows), `ratings` (id, avg f32, votes i64 — 1,721,477), `princount` (id, n u32 — 11,626,391 titles), `names` (id, birthyear) + `nameblob` (NUL-terminated), `q2edges` (name_id, title_id — 42,453,755 actor/actress credits). A run fetches ONLY the sections its query needs (offset + own CRC32C); a corrupt section fails loud instead of half-loading. | **GREEN (2026-10-21, 2/2 runs, clippy clean):** build **51.1 s ONCE** (was 178.96 s per run); Q1 **244 ms** (17-18x), Q2 **10658 ms** (10-11x, gate >=5x), Q3 **1253 ms** (61-65x); Q1 exact (count 44,906, votes 294,435,444, avg rel 8.78e-10); Q2 top-10 names+counts identical to the SQL baseline. **Five bugs the gates caught:** the pre-1995 filter was lost in a rewrite (6,001,215 rows instead of 2,574,528 → total wrong 2.3x); the reader shifted section offsets a SECOND time (the writer already stores final offsets) → false 'crc mismatch'; the index record-size guard said 24 bytes instead of 20 → the last section was dropped → index CRC mismatch; packed records (13/20/12 B) read with an 8-byte-stride accessor → wrong fields read silently; the last TSV field has no trailing tab (numVotes all zero). |
| **B3** / METHOD-040 | REQ-045 | SPEC-045 | EDIT `fiber_snap_bench.rs`; NEW `crates/sheaf-engine/src/bin/shared/fiber_snapshot.rs` | **Zero-copy load**: read the bundle ONCE and BORROW the bytes — no per-element `Vec::push` deserialization; the merge-walk decodes on the fly. Integrity primitive switched from the byte-at-a-time IEEE CRC32 (measured 150-250 ms per 42.5 MB) to the repo's existing SSE4.2 **CRC32C** (8.5-23 ms; software fallback 110-220 ms) with a format-version bump so old bundles rebuild instead of mis-reading. No new crate. | **GREEN (2026-10-20, 3/3 runs, same-run comparison):** post-read work (the part B3 changed) old 167-301 ms -> new 44-80 ms = **5.4-10.4x, median 6.2x**; total path (adds the file read, identical work in both) **2.8-4.2x**; totals exact (rel 4.38e-12); corrupt copy rejected. **Gate:** post-read median ratio >= 2.0x (measured 6.2x) AND exactness AND corrupt-reject. **Measurement discipline (two bugs found, both recorded because both flattered the result):** (1) the reference path's CRC result was unused, so the compiler DELETED the whole integrity loop — the 'old path' stopped paying for its checksum (ratio read 1.1-1.6x instead of the true 5-8x); fixed with `black_box`. (2) the reference re-read the file and got a warm-cached read while the new path paid the cold one; fixed by sharing one read. A third error — the earlier '4.7-5.1x' figure compared against a CROSS-RUN baseline measured on a cooler box — was an artifact of box state; the same-run ratio gate is what exposed it. **Next bottleneck:** the file read (22-44 ms, now dominant) — removable only by mmap (new crate, out of scope). |
| **B4** / METHOD-041 | REQ-046 | SPEC-046 | NEW `crates/sheaf-storage/src/lsp/tombstone_table.rs`; EDIT `lsp/snapshot.rs`, `lsp/mem_complex.rs` | **Versioned tombstone table**: replace the per-snapshot copied live bitmap (`mem_complex.rs:249,258`) with a shared `(cell_id, deletion_gen)` table; a snapshot at gen G sees cell alive iff `deletion_gen > G` | snapshot creation O(1) instead of O(ncells); Phase-2 repeatable-read E2E still PASS; no leaked generation pins (F1 discipline holds) |
| **B5** / METHOD-042 | REQ-047 | SPEC-047 | EDIT `bcsf/reader.rs` (FILE-015), `bcsf/builder.rs` (FILE-048), `lsp/compactor.rs` (FILE-019) | **Shadow paging**: `BcsfReader` gains an optional chain `base ⊕ remap ⊕ delta`; compactor emits a chain instead of copying the whole dataset when `dropped == 0` (remap is then an identity prefix — the hot path stays indirection-free) | merge cost scales with Δ not n; **Phase-1 lookup gate re-run: median true-fetch ≤15 ns** (SPEC-007) — a miss closes the node as `recorded`; merge-window reader rate ≥95% of post-merge baseline; totals exact |
| **B6** / METHOD-043 | REQ-048 | SPEC-048 | EDIT `bcsf/arrow_layout.rs` | **SoA as primary stalk layout** (column descriptor decides physical layout at write time) so `transpose_soa` / `compact_valid` no longer materialize 2× memory at read time | `fused_revenue_sum` exact; memory spike eliminated; no regression in `fused_scan` / `mt_scan` gates |
| **B7** / METHOD-044 | REQ-049 | SPEC-049 | EDIT `SKELETON.md` | **SKELETON drift fix**: register FILE ids for the 7 load-bearing library files that have none — `bcsf/arrow_layout.rs`, `bcsf/coord.rs`, `lsp/graded_log.rs`, `lsp/ring_wal.rs`, `lsp/colimit.rs`, `sheaf-kernel/src/components.rs`, `sheaf-kernel/src/solvers/chebyshev.rs` | every `src/**` **library** file has a FILE id; `src/bin/**` is EXEMPT per as-built delta A1 ("Bench binaries + harness helpers carry no FILE IDs"); DIP audit still clean; no orphan REQ/SPEC/METHOD ids |

## B.3 Measured evidence behind the appended nodes

| Measurement (row) | Value (column) | Source (column) |
|---|---|---|
| TPCH bundle build | 4.2 s | `fiber_join_bench` live run (this box, loaded) |
| TPCH bundle load (cached) | 158.1 ms | `fiber_snap_bench` live run |
| Of which copy/deserialize | ~153 ms (97%) | 40.6 MB file vs 5.1 ms BW floor (8.4 GB/s measured) |
| IMDb bundle build | 178.96 s | `results/imdb_shelf.json` (q2 101.9 s + q3 76.8 s) |
| Merge (compaction) cost | 71–764 ms for 1.2M cells | METHOD-003 evidence (Phase-2 E2E) |
| Snapshot live-bitmap copy | O(ncells) per snapshot, on the read path | `mem_complex.rs:249,258` (`.map(...).collect()`) |
| Thread scaling ceiling | 4 threads only 1.25× (controller-capped) | W1 in `MAX_PERF_PLAN.md` |
| Already Kafka-native in-repo | `lsp/ring_wal.rs` — Disruptor-lite lock-free MPSC, fixed 64 B slots, HW CRC | file header |
| **B3 delivered (2026-10-20)** | post-read work **167-301 ms -> 44-80 ms (5.4-10.4x, median 6.2x)**; total path 2.8-4.2x; per-element copy removed; byte-wise IEEE CRC32 (150-250 ms) -> SSE4.2 CRC32C (8.5-23 ms); next bottleneck = the 22-44 ms file read | `fiber_snap_bench` 3/3 green runs, same-run ratio gate |

## B.4 Ordering (Part B only; Part A ordering unchanged)

```
B3 (zero-copy load) ──► B2 (IMDb cache) ──► B1 (TPCH wiring)   [bench spine]
B4 (tombstones) ──► B5 (shadow paging) ──► B6 (SoA layout)     [storage spine]
B7 (skeleton drift)  independent — do anytime
```

**Scope label — Part A N3 / N4(partial) / N5 / N6 are "future-live-fiber" nodes.**
They defend a *future* design where fibers are built over live BCSF cells. Today
the bench fibers are derived from `.tbl`/`.tsv` files and never from BCSF cell
ids, so compaction cannot dangle a fiber key and the epoch stamp has nothing to
stamp. They are legitimate to plan, but they are 4 high-effort MED-risk nodes
sitting at P1 **ahead of** the 179 s win. Re-prioritised: build **B3 → B2 → B1**
first, then **B4**, and treat N3/N4/N5/N6 as a second wave to be scheduled only
when a live-fiber design exists.

## B.5 Remaining identified items (listed, not scheduled — append-only, nothing dropped)

| Item | REQ | SPEC | METHOD | Source | What | Gate | Priority |
|---|---|---|---|---|---|---|---|
| **B8** | REQ-050 | SPEC-050 | METHOD-045 | parallel C1 | **Durable segment descriptor**: on-disk BCSF segment header `[MAGIC][fmt_ver][generation][src_fingerprint][ncells][offsets_off/len][...][crc32]` + `open_segment()` so a segment opens without running the builder | opens without builder; totals exact; torn/corrupt rejected, never half-loaded | 🟠 P2 (unblocks B9/B10) |
| **B9** | REQ-051 | SPEC-051 | METHOD-046 | parallel C2 | **Persisted TIFI index sidecar**: value→cells index survives process exit (today: RAM-only, rebuilt per process) | after restart, lookup returns the same contiguous slice; parity with scan | 🟡 P3 |
| **B10** | REQ-052 | SPEC-052 | METHOD-047 | parallel C3 | **Persisted partition-map sidecar**: L3-domain assignment + ghost map survive restart | after restart, `domains_fit_l3` + `ghosts_cover_boundaries` still hold | 🟡 P3 |
| **B11** | REQ-053 | SPEC-053 | METHOD-048 | parallel C4 | **Table/catalog-level meta map** (table → segment set + column offsets + generation). Already on the ROADMAP (`ARCHITECTURE.md:110` step 3) | table opens by name; schema version gates every plan | 🟡 P3 (ROADMAP) |
| **B12** | REQ-054 | SPEC-054 | METHOD-049 | parallel A6 | **Per-section generation table** for the bundle cache: one changed input rewrites only the affected section, not all 40.6 MB | only the changed section rewrites; totals exact | 🟢 P4 |
| **B13** | REQ-055 | SPEC-055 | METHOD-050 | parallel A7 | **Consistent gate amortization policy**: `fiber_join_bench` times only the query while `fiber_snap_bench` counts its load — one rule for both, stated in the gate output | both gates use the same rule; README/RUNTIME_COMPARISON updated | 🟢 P4 |
| **B14** | REQ-056 | SPEC-056 | METHOD-051 | parallel B5 | **Zero-copy WAL replay** (keep the mapped buffer, return slices instead of `payload.to_vec()` per record) | replay exact; no per-record copy | 🟢 P4 (negligible) |

| Note | Detail |
|---|---|
| B13/B14 are policy/hygiene, not code-heavy | B13 is a rule + doc change; B14 is a small optimisation |
| B8 is the enabling node for B9/B10 | the sidecars need a durable segment format to hang off |
| Nothing from the parallel session's 22-item list is dropped | A1→N1, A2→N2, A3→B1, A4→B2, A5→B3, A6→B12, A7→B13, B1→B4, B2/B3→B5, B4→B6, B5→B14, C1→B8, C2→B9, C3→B10, C4→B11, D1→N0, D2→N7, E1→B7, E2→N6/N8, E3→N6/N8 |

## B.6 Explicitly not building (measured and refuted — retained for the record)

| Idea | Why rejected | Measurement |
|---|---|---|
| L1/L2/SRAM tier manager | layout tuning already closed | R1: bounds checks within noise (12.3 / 14.8 / 10.9 ms) |
| More thread scaling | bandwidth-capped | W1: 4 threads only 1.25x; 48 MB floor ~5.7 ms |
| Kafka-style no-fsync-per-op | breaks durability (REQ-009/REQ-032) | W2: fsync 1.2 ms vs write-only 9 us |
| Per-tier checksums | capped ~2% | W3: checksum = 2% of a durable op |
| Persisting raw memory addresses | breaks restart, compaction, MVCC pinning, cross-node reads | offsets vs addresses table (SHEAF_THEORY_REVIEW) |
| Explicit L2 pinning / prefetch hints | hardware prefetcher wins | R2: iterator-is-faster refuted |
| **B2 delivered (2026-10-21)** | IMDb build 178.96 s/run -> 51.1 s once; Q1 244 ms, Q2 10658 ms, Q3 1253 ms (10-65x); exact on all three | `imdb_fiber_bench` ALL PASS 2/2 |
| **B1 delivered (2026-10-21)** | TPCH fiber bench: 4,200 ms pre-fold per run -> 24.8-54.5 ms cached (77-169x), totals exact | `fiber_join_bench` cached path PASS |
| **B14 delivered (2026-10-21)** | WAL replay: copy 4.8 ms -> zero-copy 3.5 ms (20,000 frames identical); integrity unchanged | `wal_insert_bench` PASS |
| **N3 delivered (2026-10-21)** | fiber bundle bound to a source fingerprint; stale source => rebuild, never a stale answer | `fiber_snap_bench` + `fiber_join_bench` PASS |
