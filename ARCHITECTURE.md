# ARCHITECTURE + FILE DESIGN (Architecture → File Design gates)

DIP layers: `domain/` (pure math incl. slice arithmetic, depends on nothing) → `application/` (orchestration incl. operators/solvers, depends on domain traits) → `infrastructure/` (BCSF/WAL/executor impls) → `interfaces/` (TQL text, .section, client API).
Rule: import arrows point inward only (enforced by tests/dip_audit.sh — crate edges + folder arrows; violation = return to Planning).

## Folder tree (Phase 1 output; no file contents yet)

```
sheaf-db/
  Cargo.toml (workspace: sheaf-core, sheaf-storage, sheaf-kernel, sheaf-query, sheaf-engine — sheaf-distrib/sheaf-net deferred, see as-built delta A4)
  crates/sheaf-core/src/poset/          FOLDER-001  domain/TopologyCore
  crates/sheaf-core/src/stalk/          FOLDER-002  domain/TopologyCore
  crates/sheaf-core/src/morphism/       FOLDER-003  domain/TopologyCore
  crates/sheaf-storage/src/bcsf/         FOLDER-004  infrastructure/StorageEngine
  crates/sheaf-storage/src/lsp/          FOLDER-005  infrastructure/StorageEngine
  crates/sheaf-storage/src/index/        FOLDER-006  infrastructure/StorageEngine
  crates/sheaf-kernel/src/simd/          FOLDER-007  domain/SheafKernel (pure u64/f32 slice arithmetic, no storage/OS)
  crates/sheaf-kernel/src/operators/     FOLDER-008  application/SheafKernel (trait orchestration over domain primitives)
  crates/sheaf-kernel/src/solvers/       FOLDER-009  application/SheafKernel
  crates/sheaf-query/src/syntax/         FOLDER-010  interfaces+TQLCompiler
  crates/sheaf-query/src/ir/             FOLDER-011  application/TQLCompiler (TIR + validator + optimizer consolidated; FOLDER-012 merged here, see as-built delta A3)
  ~~crates/sheaf-engine/src/catalog/~~       FOLDER-013  DEFERRED — schema registry never built; TQL DEFINE statements serve as catalog (see as-built delta A5)
  ~~crates/sheaf-engine/src/executor/~~      FOLDER-014  DEFERRED — bench binaries serve as executor; api/result.rs serves as output (see as-built delta A5)
  crates/sheaf-engine/src/transaction/   FOLDER-015  application/EngineRuntime
  crates/sheaf-engine/src/api/           FOLDER-016  interfaces/EngineRuntime
  crates/sheaf-engine/src/corpus.rs      (bench-corpus helper, no FOLDER; E2E harness, see as-built delta A1)
  ~~crates/sheaf-distrib/src/~~               FOLDER-017  DEFERRED — implemented at storage level as bcsf/cover.rs + bcsf/partition.rs (see as-built delta A4)
  ~~crates/sheaf-net/src/~~                     FOLDER-018  DEFERRED — loopback proves protocol; no separate crate (see as-built delta A4)
  tests/e2e/phase1_bcsf/  phase2_lsp/  phase3_kernel/  phase4_tql/  phase5_opt/  phase6_split/  phase7_dist/
```

Each `tests/e2e/phaseN_<ctx>/` holds `fixtures/` (real inputs) + `expected/` (artifact next phase reads) + idempotent `run.sh` invoking `cargo run --bin <interface>`.

## File responsibilities + must-never (Phase 2 output; SRP: all statements unique)

| FILE | Responsibility (≤7 words) | Must never |
|---|---|---|
| poset/id.rs | encode CellId dimension tagged identifier | parse queries or touch floats |
| poset/cell.rs | define graded cell descriptors | store stalks or matrices |
| poset/incidence.rs | store codim-1 boundaries virtualize composites | materialize full simplex closure |
| poset/ordering.rs | enforce poset order acyclicity | execute linear algebra |
| stalk/discrete.rs | store exact discrete attribute payloads | do float matvec |
| stalk/continuous.rs | store aligned continuous embedding vectors | do string matching |
| stalk/aligned_buf.rs | allocate 64B aligned float buffers | define schema semantics |
| stalk/bundle.rs | unify discrete plus continuous handle | persist or compact |
| morphism/projection.rs | evaluate discrete projection masks | multiply float matrices |
| morphism/matrix.rs | store restriction weight matrices | filter strings |
| morphism/compound.rs | combine pi plus W morphism | own storage layout |
| bcsf/offsets.rs | index cell to incidence slice | mutate on write path |
| bcsf/indices.rs | store sorted boundary identifiers | hold floats |
| bcsf/arena.rs | pack stalks plus morphism weights | parse or plan |
| bcsf/reader.rs | serve zero-copy snapshot views | write or lock |
| bcsf/builder.rs | assemble immutable BCSF segments | serve reads or compact |
| bcsf/name_lookup.rs | find cells by discrete value | scan full BCSF |
| bcsf/partition.rs | split hypergraph L3 domains | solve linear systems |
| bcsf/cover.rs | assign hyperedges to domains | cut hyperedges by hash |
| lsp/mem_complex.rs | append lock-free live poset | serve frozen reads |
| lsp/wal.rs | log mutations with CRC | execute queries |
| lsp/snapshot.rs | isolate MVCC read snapshots | compact segments |
| lsp/compactor.rs | merge MemComplex into BCSF | accept client traffic |
| lsp/preagg.rs | maintain O(1) group accumulators | serve ad-hoc aggregates |
| lsp/epoch_wal.rs | epoch-validated append-only headers | rewrite headers in place |
| ~~index/tifi.rs~~ | ~~map attribute values to cells~~ | ~~scan full BCSF~~ |
| ~~index/partition.rs~~ | ~~partition hypergraph NUMA domains~~ | ~~solve linear systems~~ |
| simd/intersection.rs | intersect u64 slices via SIMD | know TQL syntax |
| simd/mt_scan.rs | partitioned multi-threaded scan reduce | allocate per scan |
| simd/matvec.rs | multiply W times stalk vectors | touch discrete masks |
| operators/coboundary.rs | compute delta residuals per cell | own solver iteration |
| operators/laplacian.rs | assemble sparse sheaf Laplacian | parse queries |
| operators/energy.rs | score Dirichlet energy coherence | emit client output |
| solvers/cg.rs | solve preconditioned conjugate gradient | define schema |
| solvers/tikhonov.rs | regularize Laplacian linear solves | manage transactions |
| solvers/harmonic.rs | drive harmonic section relaxation | tokenize text |
| syntax/lexer.rs | tokenize TQL topological keywords | typecheck dims |
| syntax/parser.rs | parse TQL into AST | execute vectors |
| syntax/expr_parser.rs | parse WHERE expressions | touch DDL syntax |
| syntax/define_parser.rs | parse DEFINE statements | touch query syntax |
| syntax/ast.rs | represent match restrict emit | optimize plans |
| ir/tir.rs | lower AST to TIR graph | run SIMD |
| ir/validator.rs | reject dim cycles mismatches | choose plan cost |
| ir/optimizer.rs | prune TIR via poset structure | invent new opcodes (tripwire: a 13th opcode requires a SPEC-018 amendment + METHOD node, never a silent addition) |
| ~~catalog/schema.rs~~ | ~~register cell type schemas~~ | ~~execute pipelines~~ |
| ~~catalog/index_meta.rs~~ | ~~track segment offset metadata~~ | ~~serve clients~~ |
| ~~executor/physical_plan.rs~~ | ~~represent executable plan steps~~ | ~~run threads~~ |
| ~~executor/pipeline.rs~~ | ~~run batched vectorized pipeline~~ | ~~parse TQL~~ |
| ~~executor/context.rs~~ | ~~hold scratch execution buffers~~ | ~~allocate per cell~~ |
| transaction/manager.rs | manage txn begin commit | evaluate energy |
| transaction/lock.rs | lock overlapping lattice subcomplexes | store stalks |
| ~~api/client.rs~~ | ~~accept client query sessions~~ | ~~compute Laplacian~~ |
| api/result.rs | serialize section plus energy | open sockets |
| engine/bin-shared/corpus.rs | hold shared E2E bench corpus | link into library |
| ~~distrib/cover.rs~~ | ~~partition poset into overlapping subcomplexes~~ | ~~cut hyperedges by hash~~ |
| ~~distrib/border.rs~~ | ~~represent replicated border fiber slices~~ | ~~own interior cells~~ |
| ~~distrib/schwarz.rs~~ | ~~diffuse border residuals without barriers~~ | ~~call global allreduce~~ |
| ~~distrib/crdt.rs~~ | ~~merge border edits by energy~~ | ~~take distributed locks~~ |
| ~~net/nerve.rs~~ | ~~route queries along nerve graph~~ | ~~use central coordinator~~ |
| ~~net/rdma.rs~~ | ~~read remote border arenas one-sided~~ | ~~wake remote CPU per read~~ |
| ~~net/gossip.rs~~ | ~~exchange border residuals asynchronously~~ | ~~block on barriers~~ |
| ~~net/protocol.rs~~ | ~~frame cross-node sync bytes~~ | ~~parse TQL text~~ |

SRP check: all responsibility statements distinct (recounted post as-built deltas; struck rows excluded). Line gate: `wc -l` must show every file ≤500 lines before Code Skeleton entry — enforced; parser.rs and phase4_bench.rs were split (expr_parser.rs, define_parser.rs, engine/corpus.rs) to comply.

## As-built deltas (2026-10-13 audit: text reconciled to code, no code deleted)

A1. Bench binaries + harness helpers carry no FILE IDs (precedent: phase2_bench, adversarial). `engine/corpus.rs` holds the shared Phase-4 corpus extracted from phase4_bench.rs for the SPEC-022 line gate.
A2. `syntax/parser.rs` exceeded 500 lines → split into parser.rs (374) + expr_parser.rs (WHERE logic) + define_parser.rs (DEFINE logic). Same code, same gates.
A3. Optimizer consolidated: the planned 4-file e-graph (egraph/rules_boundary/rules_pushdown/cost) was never needed — poset pruning is a single-pass rewrite. Implemented as `ir/optimizer.rs` (FILE-038 covers it; METHOD-010 verified). FOLDER-012 merged into FOLDER-011.
A4. Distributed implemented at storage level, not as separate crates: `bcsf/partition.rs` (FILE-021, L3 domains + ghosts) + `bcsf/cover.rs` (FILE-049, greedy hyperedge cover). sheaf-distrib/sheaf-net crates, border/schwarz/crdt/nerve/rdma/gossip/protocol files: DEFERRED. Waivers W-D1/W-D2 are moot (no cross-crate arrows exist); retained for history.
A5. EngineRuntime catalog/executor/client never built: TQL DEFINE statements serve as catalog, bench binaries serve as executor, api/result.rs serves as output. METHOD-016 (head-to-head benchmark) remains stub — analytic comparison in PERFORMANCE.md/PERFORMANCE_PARTITIONED.md (local sheaf-db runs + internet benchmarks) stands in until a PG/Neo4j harness exists.
ROADMAP (no code yet): a deployable service needs, in order: (1) executor pipeline over TIR (un-defer FILE-041/042), (2) client sessions (FILE-046), (3) catalog registry (FILE-039/040), (4) multi-node fabric (FOLDER-017/018). Each step re-enters the state machine at File Design with its own METHOD node; the shipped kernel/compiler/storage are the stable foundation, not the finished database.
A6. `index/` holds only mod.rs (stub). TIFI (FILE-020) implemented as `bcsf/name_lookup.rs` (METHOD-011 verified); `index/partition.rs` implemented as `bcsf/partition.rs` (METHOD-015 verified).

## Part-2 forward deltas (2026-10-08, from Phase-1 adversarial findings)

1. Compactor merges via `BcsfBuilder::extend_from_reader` bulk slice copies with SIMD offset shifts + tombstone-block skips — never cell-by-cell `push` (METHOD-017, parent FILE-048).
2. Phase-2 gates are P99-gated (writes <200 µs, cold-read degradation <5% during merges); medians are informational only.
3. Compactor uses cache-isolated streaming writes (dedicated cores and/or non-temporal stores) to protect reader L3 lines.
4. Phase-3 watchpoint: sparse-morph `binary_search` (~18 branches) isolated in Laplacian bench; stride-offset replacement triggers on branch-stall evidence only.
5. Registry discipline (F1): snapshot registers pin+gen atomically under one lock; install holds the same lock across min+prune — the prune set can never cover a live snapshot.
6. Merge-window gating (F5): reader rate inside the merge call vs a thermally-adjacent post baseline (P1 is warmup); 32k-cell `extend_range` chunks with inter-chunk yields bound burst bandwidth theft.

## Distributed plan deltas (Phase-7, from 10/10 review — HISTORICAL: all items below deferred per as-built delta A4; retained to show intent, not state)

1. Covers, not cuts: `distrib/cover.rs` partitions by open subcomplexes (KaHyPar-minimized borders); ≥95% hyperedges interior (METHOD-019).
2. No barriers: `distrib/schwarz.rs` runs local PCG <10µs and diffuses border residuals async; an `MPI_Allreduce` anywhere fails the gate (METHOD-020).
3. No 2PC: `distrib/crdt.rs` merges border edits to min-Dirichlet-energy sections; interior deletes stay zero-network (METHOD-021).
4. No coordinator: `net/nerve.rs` routes local queries zero-hop and slices spanning queries on nerve edges (METHOD-022).
5. No RPC on hot path: `net/rdma.rs` reads border arenas one-sided; 1.2µs gate is HW-qualified (RoCEv2 required, loopback proves protocol) — same host-qualification precedent as SPEC-007 (METHOD-023).

## Documented DIP waivers (SUPERSEDED by as-built delta A4 — no cross-crate arrows exist; retained for history)

- W-D3 (live): `sheaf-storage` crate declares `sheaf-kernel` in [dependencies], but only `src/bin/phase1_bench.rs` (harness) imports it (sorted-scan check); lib code under bcsf/lsp/index never does — enforced by tests/dip_audit.sh. Rationale: cargo has no per-bin dependencies; splitting the bench into its own crate for one import is churn without layer benefit.

Strict inward-only import would force a new domain-trait layer for two arrows whose cross-layer coupling is inherent, not accidental. Both waived explicitly (FOLDER-014 `application+infrastructure` sets the precedent); the audit script enforces that every outward arrow has a matching entry here, so waivers cannot accumulate silently.

- W-D1: `distrib/cover.rs` (FILE-049) → `index/partition.rs` (FILE-021). Rationale: cover construction consumes KaHyPar partition-map output types directly; interposing a domain trait would add a file to launder a genuine data dependency.
- W-D2: `distrib/border.rs` (FILE-050) → `bcsf/reader.rs` (FILE-015). Rationale: border slices are views over reader snapshots; re-wrapping the reader in an application trait duplicates a zero-cost view for layer purity alone.

## As-built deltas (continued)

A7. Theoretical-maximum program (2026-10-14, planned, unbuilt): bottleneck-probe measurements (bandwidth ~8.4 GB/s read, fsync 1.2 ms, 4-thread scaling ~1.25x, checksum ~2% of ops) motivate FILE-060/061/062 (P1/P3/P4 builds). P2/P5/P6 modify existing files (name_lookup, partition) with no new FILE IDs. Traceability: REQ-030..035 / SPEC-030..035 / METHOD-024..029 (stubs). Predictions are stretch targets with hard-gate parity floors; see MAX_PERF_PLAN.md.
