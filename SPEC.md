# SPEC — sheaf-db requirements → specs

Each `REQ-###` maps to one `SPEC-###`. Acceptance criteria are the Turn-5/8 milestone tests; they become node `acceptance_criteria` in `SKELETON.md`. Status: `proposed` (no gate passed yet).

| REQ | Requirement (one line) | Source | SPEC |
|---|---|---|---|
| REQ-001 | Rust-only engine + language (layout control, zero-cost, concurrency) | T2 | SPEC-001 |
| REQ-002 | N-dim nodes/edges/relations as cellular sheaf over graded poset | T2/T3 | SPEC-002 |
| REQ-003 | Dual stalks: discrete exact payload + continuous R^d embedding | T4-V1 | SPEC-003 |
| REQ-004 | Typed morphisms `(π,W)` with zero-copy mask + SIMD matrix | T4-V1 | SPEC-004 |
| REQ-005 | Soft-sheaf energy relaxation `E<=ε` + async diffusion; discrete via lattice/CRDT | T4-V2 | SPEC-005 |
| REQ-006 | Pruned poset (maximal cells, lazy faces, sorted IDs, AVX intersect) | T4-V3 | SPEC-006 |
| REQ-007 | BCSF flat arrays, no HashMap in hot path, 64B aligned | T4-V4/T5-P1 | SPEC-007 |
| REQ-008 | LSP: MemComplex + frozen BaseComplex + background compactor | T4-V4/T5-P2 | SPEC-008 |
| REQ-009 | WAL + MVCC snapshot isolation (monotonic generations) | T5-P2 | SPEC-009 |
| REQ-010 | Coboundary `δ` (bitwise π + SIMD Wx) | T5-P3 | SPEC-010 |
| REQ-011 | Sheaf Laplacian `L_F=δ^Tδ` sparse block SpMV | T5-P3 | SPEC-011 |
| REQ-012 | Block-Jacobi PCG + Tikhonov `(L+λI)x=x0`, κ≤15, 8–18 iters | T13-G2 | SPEC-012 |
| REQ-013 | Eulerian orientation `[σ:τ]=(-1)^i`, `δ²=0` validator, PSD Laplacian | T13-G1 | SPEC-013 |
| REQ-014 | Upper-Star tombstone deletes (live_mask, no BCSF shift) | T13-G3 | SPEC-014 |
| REQ-015 | TIFI attribute index: values as (-1)-dim roots → CellId slice | T13-G4 | SPEC-015 |
| REQ-016 | NUMA partitions (64 MB L3) + Ghost Fiber replicas | T13-G5 | SPEC-016 |
| REQ-017 | TQL DDL/DML/DQL (DEFINE/INSERT/MATCH/RESTRICT/SOLVE/EMIT) | T8/T11 | SPEC-017 |
| REQ-018 | TIR lowering to 12-opcode vectorized plan | T11 | SPEC-018 |
| REQ-019 | E-graph optimizer (`∂∂=0`, π-pushdown, cost model) | T5-P5 | SPEC-019 |
| REQ-020 | `.section` YAML output with energy certificate | T8 | SPEC-020 |
| REQ-021 | Phase-6 head-to-head: N≥4 ≥5× QPS, P99 sub-ms | T5-P6/T8 | SPEC-021 |
| REQ-022 | Files ≤500 lines; 7 crates; math→file map, no monoliths | T7 | SPEC-022 |
| REQ-023 | Hardware-first order; dedicated kernel; batch-1024, no per-cell boundary cross | T10 | SPEC-023 |
| REQ-024 | Cross-paradigm contract (graph/tree 10, joins 9, doc 7, KV 6, OLAP 4) + sugar | T12 | SPEC-024 |
| REQ-025 | Mayer-Vietoris covers, no hash cuts | Dist-10/10 P1 | SPEC-025 |
| REQ-026 | Async Schwarz DDM, zero barriers | Dist-10/10 P2 | SPEC-026 |
| REQ-027 | Cohomology CRDT gluing, no 2PC | Dist-10/10 P3 | SPEC-027 |
| REQ-028 | Cech nerve zero-hop routing | Dist-10/10 P4 | SPEC-028 |
| REQ-029 | RDMA one-sided border reads | Dist-10/10 P5 | SPEC-029 |

## SPEC details (acceptance = gate)

- SPEC-007: 1M 0-cells + 200k 4-cells; mem <1.5× raw; boundary lookup sub-15 ns. E2E: `tests/e2e/phase1_bcsf/`. Host qualification (2026-10-08): gate is median-of-11 steady-state true-fetch; marginal on 12 MB laptop LLC (i7-1165G7, hot set ~11.2 MB) — ~50% process PASS, excursions to ~30ns worst-round recorded in evidence; target-class 32–96 MB LLC passes with headroom. Worst-round/cold-stride series is informational Phase-2 P99 input, not gated.
- SPEC-008/009: 4 writers @50k hyperedges/s + 8 readers; zero corruption/deadlock; P99 write <200 µs (staging path; periodic-seal P99 reported separately); full commit path (begin→commit, incl. snapshot copy) P99 <2 ms over 200 sampled commits. Merge-window reader throughput ≥95% of the thermally-adjacent post-merge baseline (P1 is warmup only — cool-chip turbo made cross-window comparison measure thermals; per-op P99 series reported alongside, never median). Compactor merges via `BcsfBuilder::extend_from_reader`/`extend_range` bulk slice copies (32k-cell chunks with inter-chunk yields) with offset shifts + tombstone-block skips, pinned to a compaction core, so merges never evict reader L3 lines. WAL: header-word (valid_len + CRC) + group-commit buffering + pre-allocation; replay exact, short-file truncation semantics, garbage-past-valid ignored. Phase-2 memory footprint reported (star/live/WAL estimate ~23 MB peak) — informational, gated by Phase 6. Adversarial suite `tests/e2e/phase2_adv/` (A1 remap identity, A2 delete×merge races, A3 isolation, A4 WAL bit-flips, A5 txn matrix, A6 shadow-model fuzz, A7 prune churn, A8 degenerates) — all PASS, exit-gated. E2E: `tests/e2e/phase2_lsp/`.
- SPEC-010/011/012: 10k noisy nodes → `E≤1e-4`, ≤25 iters (BJ-PCG 8–18 typical), <8 ms Rayon/SIMD. E2E: `tests/e2e/phase3_kernel/`. Part-2 watchpoint (2026-10-08): sparse-morph `binary_search` (~18 branches/cell) is free for boundary/txn paths but billed per face in Laplacian assembly — Phase-3 bench must isolate morph-fetch cost; explicit trigger: replace with direct stride offsets inside boundary arrays on branch-stall evidence.
- SPEC-017/018: 50 valid TQL compile <50 µs each; 20 malformed pinpoint errors; lowering matches 12-opcode shapes. E2E: `tests/e2e/phase4_tql/`.
- SPEC-019: N=6 query `O(2⁶)` → single BCSF slice + vectorized check per `EXPLAIN TOPOLOGY`. E2E: `tests/e2e/phase5_opt/`.
- SPEC-020: every DQL emits `.section` with all fields; `dirichlet_energy`, `iterations_to_converge`, `cohomology_state:HARMONIC_BOUNDED` present; `execution_time_us` recorded.
- SPEC-021: fraud net 10k checks / 10M records; N≥4 ≥5× QPS vs PG 5-join + Neo4j intermediate-node; P99 harmonic retrieval sub-ms.
- SPEC-013: Eulerian validator rejects non-`δ²=0` posets; Laplacian PSD on all fixtures.
- SPEC-014: delete 0-cell → dependents tombstoned via `Star(σ)`; readers skip via `live_mask`; no BCSF rebuild on delete path.
- SPEC-015: `p.name=="Alice"` → TIFI down-set slice; no full scan; indexed + unindexed parity test.
- SPEC-016: cross-socket hypercell executes from Ghost replica; no remote per-cell fetch in kernel trace.
- SPEC-022: `wc -l` gate: no file >500 lines; responsibility uniqueness check passes (see ARCHITECTURE.md); 7 crates (`sheaf-distrib`, `sheaf-net` added for Phase 7).
- SPEC-003/004/005/006: covered by SPEC-007–012 fixtures (dual payloads, typed morphisms, strain recording, lazy faces).
- SPEC-025: KaHyPar cover fixture: ≥95% N-ary relations interior to one subcomplex; zero split hyperedges; border overlap minimal vs hash-cut baseline. E2E: `tests/e2e/phase7_distrib/`.
- SPEC-026: 2-node RAS fixture converges with zero barrier collectives in trace; local PCG <10µs per subdomain; border residuals exchanged async. E2E: `tests/e2e/phase7_distrib/`.
- SPEC-027: concurrent border edits converge deterministically to the min-Dirichlet-energy section; interior deletes emit zero network messages; no 2PC locks/latches in trace. E2E: `tests/e2e/phase7_distrib/`.
- SPEC-028: 100-query nerve fixture: single-subcomplex queries route with zero coordinator hops; spanning queries slice on nerve edges with correctly merged sections. E2E: `tests/e2e/phase7_distrib/`.
- SPEC-029: one-sided border reads functionally correct over loopback transport; 1.2µs gate HW-qualified (RoCEv2 NIC required — dev laptop proves protocol only, same precedent as SPEC-007 host qualification). Head-to-head N=5 fraud check <35µs vs 45–120ms Spanner-class cited as HW-qualified target. E2E: `tests/e2e/phase7_distrib/`.
