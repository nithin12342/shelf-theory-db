# ARCHITECTURE + FILE DESIGN (Architecture → File Design gates)

DIP layers: `domain/` (pure math incl. slice arithmetic, depends on nothing) → `application/` (orchestration incl. operators/solvers, depends on domain traits) → `infrastructure/` (BCSF/WAL/executor impls) → `interfaces/` (TQL text, .section, client API).
Rule: import arrows point inward only (enforced by `pub(crate)` + `cargo check`; violation = return to Planning).

## Folder tree (Phase 1 output; no file contents yet)

```
sheaf-db/
  Cargo.toml (workspace: sheaf-core, sheaf-storage, sheaf-kernel, sheaf-query, sheaf-engine, sheaf-distrib, sheaf-net)
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
  crates/sheaf-query/src/ir/             FOLDER-011  application/TQLCompiler
  crates/sheaf-query/src/optimizer/      FOLDER-012  application/TQLCompiler
  crates/sheaf-engine/src/catalog/       FOLDER-013  application/EngineRuntime
  crates/sheaf-engine/src/executor/      FOLDER-014  application+infrastructure/EngineRuntime
  crates/sheaf-engine/src/transaction/   FOLDER-015  application/EngineRuntime
  crates/sheaf-engine/src/api/           FOLDER-016  interfaces/EngineRuntime
  crates/sheaf-distrib/src/               FOLDER-017  application+infrastructure/DistributedEngine (cover/border fuse orchestration with storage-impl types; waivers W-D1/W-D2 below)
  crates/sheaf-net/src/                     FOLDER-018  infrastructure/DistributedEngine
  tests/e2e/phase1_bcsf/  phase2_lsp/  phase3_kernel/  phase4_tql/  phase5_opt/  phase6_bench/  phase7_distrib/
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
| lsp/mem_complex.rs | append lock-free live poset | serve frozen reads |
| lsp/wal.rs | log mutations with CRC | execute queries |
| lsp/snapshot.rs | isolate MVCC read snapshots | compact segments |
| lsp/compactor.rs | merge MemComplex into BCSF | accept client traffic |
| index/tifi.rs | map attribute values to cells | scan full BCSF |
| index/partition.rs | partition hypergraph NUMA domains | solve linear systems |
| simd/intersection.rs | intersect u64 slices via SIMD | know TQL syntax |
| simd/matvec.rs | multiply W times stalk vectors | touch discrete masks |
| operators/coboundary.rs | compute delta residuals per cell | own solver iteration |
| operators/laplacian.rs | assemble sparse sheaf Laplacian | parse queries |
| operators/energy.rs | score Dirichlet energy coherence | emit client output |
| solvers/cg.rs | solve preconditioned conjugate gradient | define schema |
| solvers/tikhonov.rs | regularize Laplacian linear solves | manage transactions |
| solvers/harmonic.rs | drive harmonic section relaxation | tokenize text |
| syntax/lexer.rs | tokenize TQL topological keywords | typecheck dims |
| syntax/parser.rs | parse TQL into AST | execute vectors |
| syntax/ast.rs | represent match restrict emit | optimize plans |
| ir/tir.rs | lower AST to TIR graph | run SIMD |
| ir/validator.rs | reject dim cycles mismatches | choose plan cost |
| optimizer/egraph.rs | hold equivalence plan classes | change semantics |
| optimizer/rules_boundary.rs | prune via partial-partial zero | push filters |
| optimizer/rules_pushdown.rs | push discrete filters early | prune boundaries |
| optimizer/cost.rs | choose scan versus solve | parse text |
| catalog/schema.rs | register cell type schemas | execute pipelines |
| catalog/index_meta.rs | track segment offset metadata | serve clients |
| executor/physical_plan.rs | represent executable plan steps | run threads |
| executor/pipeline.rs | run batched vectorized pipeline | parse TQL |
| executor/context.rs | hold scratch execution buffers | allocate per cell |
| transaction/manager.rs | manage txn begin commit | evaluate energy |
| transaction/lock.rs | lock overlapping lattice subcomplexes | store stalks |
| api/client.rs | accept client query sessions | compute Laplacian |
| api/result.rs | serialize section plus energy | open sockets |
| distrib/cover.rs | partition poset into overlapping subcomplexes | cut hyperedges by hash |
| distrib/border.rs | represent replicated border fiber slices | own interior cells |
| distrib/schwarz.rs | diffuse border residuals without barriers | call global allreduce |
| distrib/crdt.rs | merge border edits by energy | take distributed locks |
| net/nerve.rs | route queries along nerve graph | use central coordinator |
| net/rdma.rs | read remote border arenas one-sided | wake remote CPU per read |
| net/gossip.rs | exchange border residuals asynchronously | block on barriers |
| net/protocol.rs | frame cross-node sync bytes | parse TQL text |

SRP check: all 56 statements distinct. Line gate: `wc -l` must show every file ≤500 lines before Code Skeleton entry.

## Part-2 forward deltas (2026-10-08, from Phase-1 adversarial findings)

1. Compactor merges via `BcsfBuilder::extend_from_reader` bulk slice copies with SIMD offset shifts + tombstone-block skips — never cell-by-cell `push` (METHOD-017, parent FILE-048).
2. Phase-2 gates are P99-gated (writes <200 µs, cold-read degradation <5% during merges); medians are informational only.
3. Compactor uses cache-isolated streaming writes (dedicated cores and/or non-temporal stores) to protect reader L3 lines.
4. Phase-3 watchpoint: sparse-morph `binary_search` (~18 branches) isolated in Laplacian bench; stride-offset replacement triggers on branch-stall evidence only.
5. Registry discipline (F1): snapshot registers pin+gen atomically under one lock; install holds the same lock across min+prune — the prune set can never cover a live snapshot.
6. Merge-window gating (F5): reader rate inside the merge call vs a thermally-adjacent post baseline (P1 is warmup); 32k-cell `extend_range` chunks with inter-chunk yields bound burst bandwidth theft.

## Distributed plan deltas (Phase-7, from 10/10 review)

1. Covers, not cuts: `distrib/cover.rs` partitions by open subcomplexes (KaHyPar-minimized borders); ≥95% hyperedges interior (METHOD-019).
2. No barriers: `distrib/schwarz.rs` runs local PCG <10µs and diffuses border residuals async; an `MPI_Allreduce` anywhere fails the gate (METHOD-020).
3. No 2PC: `distrib/crdt.rs` merges border edits to min-Dirichlet-energy sections; interior deletes stay zero-network (METHOD-021).
4. No coordinator: `net/nerve.rs` routes local queries zero-hop and slices spanning queries on nerve edges (METHOD-022).
5. No RPC on hot path: `net/rdma.rs` reads border arenas one-sided; 1.2µs gate is HW-qualified (RoCEv2 required, loopback proves protocol) — same host-qualification precedent as SPEC-007 (METHOD-023).

## Documented DIP waivers (accepted deviations, Phase-7)

Strict inward-only import would force a new domain-trait layer for two arrows whose cross-layer coupling is inherent, not accidental. Both waived explicitly (FOLDER-014 `application+infrastructure` sets the precedent); the audit script enforces that every outward arrow has a matching entry here, so waivers cannot accumulate silently.

- W-D1: `distrib/cover.rs` (FILE-049) → `index/partition.rs` (FILE-021). Rationale: cover construction consumes KaHyPar partition-map output types directly; interposing a domain trait would add a file to launder a genuine data dependency.
- W-D2: `distrib/border.rs` (FILE-050) → `bcsf/reader.rs` (FILE-015). Rationale: border slices are views over reader snapshots; re-wrapping the reader in an application trait duplicates a zero-cost view for layer purity alone.
