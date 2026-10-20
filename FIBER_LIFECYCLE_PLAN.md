# FIBER_LIFECYCLE_PLAN.md — runtime bundle lifecycle + write-time correctness
**Process: intention-engineering.** Branch: `fiber-bundle-replacement` (off `884809b`).
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
| N8 / METHOD-037 | REQ-042 | SPEC-042 | EDIT `COMPLEXITY_ANALYSIS.md` (compile-time axis row + A9 close-out) | Doc row present and consistent with measured 109.3 s | LOW |

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
