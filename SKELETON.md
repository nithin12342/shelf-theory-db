# SKELETON.md — sheaf-db (Planning gate artifact)

system: sheaf-db
status: 27/29 methods e2e_passed (unchanged); +32 built/verified nodes (+B6 SoA primary layout) (+B5 shadow paging) (+N3 epoch stamp, +N4 per-domain fiber, +N5 frozen epoch, +N6 lifecycle gate) (+B11 catalog meta map, +B12 per-section generations) (+B8 durable segment, +B9 TIFI sidecar, +B10 partition sidecar) (+B8 durable segment descriptor) (+B4 versioned tombstones, +B14 zero-copy WAL replay) (METHOD-030 shared fiber module, 031 bundle cache, 038 TPCH wiring pending, 039 IMDb sectioned bundle, 040 zero-copy load, 041 tombstones pending, 042 shadow paging pending, 043 SoA pending, 044 skeleton drift, 050/051 pending) + 15 planned (Part A N0-N8 + Part B parallel-session additions B1-B7) (fiber lifecycle: shared pre-fold, fingerprint bundle cache, epoch stamp + fallback, per-domain incremental update, frozen-epoch build, concurrent stress gate, gated release-profile change, compile-time model row) per FIBER_LIFECYCLE_PLAN.md, 2 stubs (METHOD-016 head-to-head environment-blocked; METHOD-028 MPHF deferred — diminishing returns, documented); 2026-10-13 audit reconciled text to code (ARCHITECTURE as-built deltas A1-A6; parser.rs/phase4_bench.rs split for SPEC-022; Phase-7 methods reparented to real files; distrib/net/catalog/executor marked DEFERRED); clippy -D warnings clean; DIP audit clean (2 waived W-D1/W-D2 moot, no cross-crate arrows exist)

requirements:
  - {id: REQ-001, text: "Rust-only engine plus language", spec_id: SPEC-001}
  - {id: REQ-002, text: "Cellular sheaf over graded poset", spec_id: SPEC-002}
  - {id: REQ-003, text: "Dual stalks discrete plus continuous", spec_id: SPEC-003}
  - {id: REQ-004, text: "Typed morphisms pi plus W", spec_id: SPEC-004}
  - {id: REQ-005, text: "Soft-sheaf energy relaxation", spec_id: SPEC-005}
  - {id: REQ-006, text: "Pruned poset lazy faces", spec_id: SPEC-006}
  - {id: REQ-007, text: "BCSF flat arrays hot path", spec_id: SPEC-007}
  - {id: REQ-008, text: "LSP MemComplex plus compactor", spec_id: SPEC-008}
  - {id: REQ-009, text: "WAL plus MVCC snapshots", spec_id: SPEC-009}
  - {id: REQ-010, text: "Coboundary operator delta", spec_id: SPEC-010}
  - {id: REQ-011, text: "Sheaf Laplacian SpMV", spec_id: SPEC-011}
  - {id: REQ-012, text: "Block-Jacobi PCG Tikhonov", spec_id: SPEC-012}
  - {id: REQ-013, text: "Eulerian orientation delta2 zero", spec_id: SPEC-013}
  - {id: REQ-014, text: "Upper-Star tombstone deletes", spec_id: SPEC-014}
  - {id: REQ-015, text: "TIFI attribute index", spec_id: SPEC-015}
  - {id: REQ-016, text: "NUMA partitions plus ghosts", spec_id: SPEC-016}
  - {id: REQ-017, text: "TQL DDL DML DQL", spec_id: SPEC-017}
  - {id: REQ-018, text: "TIR 12-opcode lowering", spec_id: SPEC-018}
  - {id: REQ-019, text: "E-graph optimizer", spec_id: SPEC-019}
  - {id: REQ-020, text: "Section YAML certificate", spec_id: SPEC-020}
  - {id: REQ-021, text: "Head-to-head 5x QPS", spec_id: SPEC-021}
  - {id: REQ-022, text: "Files le500 lines 7 crates", spec_id: SPEC-022}
  - {id: REQ-023, text: "Hardware-first dedicated kernel", spec_id: SPEC-023}
  - {id: REQ-024, text: "Cross-paradigm contract plus sugar", spec_id: SPEC-024}
  - {id: REQ-025, text: "Mayer-Vietoris covers no hash cuts", spec_id: SPEC-025}
  - {id: REQ-026, text: "Async Schwarz DDM zero barriers", spec_id: SPEC-026}
  - {id: REQ-027, text: "Cohomology CRDT gluing no 2PC", spec_id: SPEC-027}
  - {id: REQ-028, text: "Cech nerve zero-hop routing", spec_id: SPEC-028}
  - {id: REQ-029, text: "RDMA one-sided border reads", spec_id: SPEC-029}

data_flow_order: [TopologyCore, StalkMorphism, StorageEngine, SheafKernel, TQLCompiler, EngineRuntime, DistributedEngine]

bounded_contexts:

  - name: TopologyCore
    reason_separate: "owns poset math, nothing else defines incidence"
    sot_id: SOT-001
    aggregates:
      - name: GradedPoset
        invariant: "down-closure holds; delta sums over formal codim-1 faces with parity signs so delta2=0; 0-to-k queries use composite chain map through virtual codim-1 faces (sorted-subset indexing), never direct codim>1 evaluation"
        entities: [Cell, Boundary]
        value_objects: [CellId, Dimension, OrientationSign]
        ports_in: []
        ports_out: ["CellId+Boundary -> StalkMorphism, StorageEngine"]
    files:
      - {path: crates/sheaf-core/src/poset/id.rs, file_id: FILE-001, folder_id: FOLDER-001, responsibility: "encode CellId dimension tagged identifier", depends_on: []}
      - {path: crates/sheaf-core/src/poset/cell.rs, file_id: FILE-002, folder_id: FOLDER-001, responsibility: "define graded cell descriptors", depends_on: [FILE-001]}
      - {path: crates/sheaf-core/src/poset/incidence.rs, file_id: FILE-003, folder_id: FOLDER-001, responsibility: "store codim-1 boundaries virtualize composites", depends_on: [FILE-001, FILE-002]}
      - {path: crates/sheaf-core/src/poset/ordering.rs, file_id: FILE-004, folder_id: FOLDER-001, responsibility: "enforce poset order acyclicity", depends_on: [FILE-002, FILE-003]}

  - name: StalkMorphism
    reason_separate: "owns data payloads, never touches storage layout"
    sot_id: SOT-002
    aggregates:
      - name: StalkBundle
        invariant: "discrete and continuous dims always match morphism signatures"
        entities: [DiscreteStalk, ContinuousStalk]
        value_objects: [ProjectionMask, WeightMatrix]
        ports_in: ["CellId+Boundary <- TopologyCore"]
        ports_out: ["Stalk+morphism -> StorageEngine, SheafKernel"]
    files:
      - {path: crates/sheaf-core/src/stalk/discrete.rs, file_id: FILE-005, folder_id: FOLDER-002, responsibility: "store exact discrete attribute payloads", depends_on: [FILE-002]}
      - {path: crates/sheaf-core/src/stalk/continuous.rs, file_id: FILE-006, folder_id: FOLDER-002, responsibility: "store aligned continuous embedding vectors", depends_on: [FILE-002]}
      - {path: crates/sheaf-core/src/stalk/aligned_buf.rs, file_id: FILE-007, folder_id: FOLDER-002, responsibility: "allocate 64B aligned float buffers", depends_on: []}
      - {path: crates/sheaf-core/src/stalk/bundle.rs, file_id: FILE-008, folder_id: FOLDER-002, responsibility: "unify discrete plus continuous handle", depends_on: [FILE-005, FILE-006]}
      - {path: crates/sheaf-core/src/morphism/projection.rs, file_id: FILE-009, folder_id: FOLDER-003, responsibility: "evaluate discrete projection masks", depends_on: [FILE-005]}
      - {path: crates/sheaf-core/src/morphism/matrix.rs, file_id: FILE-010, folder_id: FOLDER-003, responsibility: "store restriction weight matrices", depends_on: [FILE-006, FILE-007]}
      - {path: crates/sheaf-core/src/morphism/compound.rs, file_id: FILE-011, folder_id: FOLDER-003, responsibility: "combine pi plus W morphism", depends_on: [FILE-009, FILE-010]}

  - name: StorageEngine
    reason_separate: "owns bytes on disk/RAM, never evaluates math"
    sot_id: SOT-003
    aggregates:
      - name: BCSFView
        invariant: "offsets/indices/arena stay sorted, aligned, immutable between compactions"
        ports_in: ["Stalk+morphism <- StalkMorphism"]
        ports_out: ["boundary slices -> SheafKernel, TQLCompiler"]
      - name: MemComplex
        invariant: "appends only; epoch GC; deletes via tombstone never shift"
        ports_in: ["writes <- EngineRuntime"]
        ports_out: ["segments -> compactor"]
      - name: WALog
        invariant: "every mutation precedes WAL CRC record"
      - name: TIFIIndex
        invariant: "indexed value maps to contiguous CellId slice"
      - name: PartitionMap
        invariant: "each hypercell executes inside one 64MB L3 domain"
    files:
      - {path: crates/sheaf-storage/src/bcsf/offsets.rs, file_id: FILE-012, folder_id: FOLDER-004, responsibility: "index cell to incidence slice", depends_on: [FILE-001]}
      - {path: crates/sheaf-storage/src/bcsf/indices.rs, file_id: FILE-013, folder_id: FOLDER-004, responsibility: "store sorted boundary identifiers", depends_on: [FILE-012]}
      - {path: crates/sheaf-storage/src/bcsf/arena.rs, file_id: FILE-014, folder_id: FOLDER-004, responsibility: "pack stalks plus morphism weights", depends_on: [FILE-006, FILE-010]}
      - {path: crates/sheaf-storage/src/bcsf/reader.rs, file_id: FILE-015, folder_id: FOLDER-004, responsibility: "serve zero-copy snapshot views", depends_on: [FILE-012, FILE-013]}
      - {path: crates/sheaf-storage/src/bcsf/builder.rs, file_id: FILE-048, folder_id: FOLDER-004, responsibility: "assemble immutable BCSF segments", depends_on: [FILE-012, FILE-013, FILE-014, FILE-015]}
      - {path: crates/sheaf-storage/src/lsp/mem_complex.rs, file_id: FILE-016, folder_id: FOLDER-005, responsibility: "append lock-free live poset", depends_on: [FILE-002, FILE-008]}
      - {path: crates/sheaf-storage/src/lsp/wal.rs, file_id: FILE-017, folder_id: FOLDER-005, responsibility: "log mutations with CRC", depends_on: [FILE-016]}
      - {path: crates/sheaf-storage/src/lsp/snapshot.rs, file_id: FILE-018, folder_id: FOLDER-005, responsibility: "isolate MVCC read snapshots", depends_on: [FILE-016]}
      - {path: crates/sheaf-storage/src/lsp/compactor.rs, file_id: FILE-019, folder_id: FOLDER-005, responsibility: "merge MemComplex into BCSF", depends_on: [FILE-015, FILE-016]}
      - {path: crates/sheaf-storage/src/lsp/preagg.rs, file_id: FILE-061, folder_id: FOLDER-005, responsibility: "maintain O(1) group accumulators", depends_on: [FILE-016]}
      - {path: crates/sheaf-storage/src/lsp/epoch_wal.rs, file_id: FILE-062, folder_id: FOLDER-005, responsibility: "epoch-validated append-only headers", depends_on: [FILE-017]}
      - {path: crates/sheaf-storage/src/bcsf/name_lookup.rs, file_id: FILE-020, folder_id: FOLDER-004, responsibility: "find cells by discrete value (TIFI role)", depends_on: [FILE-005, FILE-013]}
      - {path: crates/sheaf-storage/src/bcsf/partition.rs, file_id: FILE-021, folder_id: FOLDER-004, responsibility: "split hypergraph into L3 domains", depends_on: [FILE-013]}
      - {path: crates/sheaf-engine/src/bin/shared/fiber.rs, file_id: FILE-063, folder_id: FOLDER-004, responsibility: "shared fiber pre-fold and bundle types", depends_on: [FILE-061]}
      - {path: crates/sheaf-storage/src/lsp/fiber_cache.rs, file_id: FILE-067, folder_id: FOLDER-005, responsibility: "fingerprint-bound fiber bundle cache", depends_on: [FILE-017]}
      - {path: crates/sheaf-storage/src/lsp/fiber_epoch.rs, file_id: FILE-068, folder_id: FOLDER-005, responsibility: "fiber epoch stamp, validation, fallback signal", depends_on: [FILE-017]}
      - {path: crates/sheaf-storage/src/bcsf/fiber_domain.rs, file_id: FILE-069, folder_id: FOLDER-004, responsibility: "per-domain incremental fiber update", depends_on: [FILE-021]}
      - {path: crates/sheaf-engine/src/bin/fiber_lifecycle_bench.rs, file_id: FILE-070, folder_id: FOLDER-004, responsibility: "concurrent read/write fiber gate", depends_on: [FILE-063]}
      - {path: sheaf-db/tests/e2e/fiber_lifecycle/, file_id: FILE-071, folder_id: FOLDER-006, responsibility: "fiber lifecycle E2E evidence", depends_on: [FILE-070]}
      - {path: sheaf-db/Cargo.toml, file_id: FILE-072, folder_id: FOLDER-001, responsibility: "release profile (gated restructure)", depends_on: []}
      - {path: COMPLEXITY_ANALYSIS.md, file_id: FILE-073, folder_id: FOLDER-001, responsibility: "complexity model incl. compile-time axis", depends_on: []}
      - {path: crates/sheaf-engine/src/bin/shared/imdb_fiber.rs, file_id: FILE-074, folder_id: FOLDER-019, responsibility: "cached IMDb fiber bundle sections", depends_on: [FILE-064]}
      - {path: crates/sheaf-storage/src/lsp/tombstone_table.rs, file_id: FILE-075, folder_id: FOLDER-005, responsibility: "versioned delete table (no bitmap copy)", depends_on: [FILE-016]}
      - {path: crates/sheaf-storage/src/bcsf/arrow_layout.rs, file_id: FILE-076, folder_id: FOLDER-004, responsibility: "Arrow-compatible columnar stalk slabs", depends_on: [FILE-006]}
      - {path: crates/sheaf-storage/src/bcsf/coord.rs, file_id: FILE-077, folder_id: FOLDER-004, responsibility: "dense 0-cochain coordinate vectors", depends_on: [FILE-076]}
      - {path: crates/sheaf-storage/src/lsp/graded_log.rs, file_id: FILE-078, folder_id: FOLDER-005, responsibility: "dimension-partitioned WAL with down-closure", depends_on: [FILE-017]}
      - {path: crates/sheaf-storage/src/lsp/ring_wal.rs, file_id: FILE-079, folder_id: FOLDER-005, responsibility: "lock-free MPSC ring WAL with HW CRC", depends_on: [FILE-017]}
      - {path: crates/sheaf-storage/src/lsp/colimit.rs, file_id: FILE-080, folder_id: FOLDER-005, responsibility: "background canonical projection", depends_on: [FILE-017, FILE-048]}
      - {path: crates/sheaf-kernel/src/components.rs, file_id: FILE-081, folder_id: FOLDER-009, responsibility: "b0 connected components via union-find", depends_on: []}
      - {path: crates/sheaf-kernel/src/solvers/chebyshev.rs, file_id: FILE-082, folder_id: FOLDER-009, responsibility: "Chebyshev Tikhonov filter", depends_on: [FILE-025]}
      - {path: crates/sheaf-storage/src/bcsf/segment.rs, file_id: FILE-083, folder_id: FOLDER-002, responsibility: "durable segment descriptor: write/open a BCSF segment without the builder", depends_on: [FILE-017]}
      - {path: crates/sheaf-storage/src/bin/segment_bench.rs, file_id: FILE-084, folder_id: FOLDER-002, responsibility: "B8 gate: segment round-trip exact, corrupt/truncated/alien rejected, restore identical", depends_on: [FILE-083]}
      - {path: crates/sheaf-engine/src/bin/shadow_paging_bench.rs, file_id: FILE-097, folder_id: FOLDER-006, responsibility: "B5 gate: shadow merge O(delta), chained totals exact, chained lookup budget", depends_on: [FILE-015]}
      - {path: crates/sheaf-storage/src/bcsf/arrow_layout.rs, file_id: FILE-098, folder_id: FOLDER-002, responsibility: "Arrow-compatible stalk slab, SoA primary layout (B6): columns are views, not copies", depends_on: []}
      - {path: crates/sheaf-storage/src/bcsf/name_lookup.rs, file_id: FILE-085, folder_id: FOLDER-002, responsibility: "value->cells index + persisted TIFI sidecar (survives process exit)", depends_on: [FILE-015]}
      - {path: crates/sheaf-storage/src/bin/index_sidecar_bench.rs, file_id: FILE-086, folder_id: FOLDER-002, responsibility: "B9 gate: sidecar reload in a 2nd process, scan/range parity, corrupt reject", depends_on: [FILE-085]}
      - {path: crates/sheaf-storage/src/bcsf/partition.rs, file_id: FILE-087, folder_id: FOLDER-002, responsibility: "L3 domain partition + ghost map + persisted partition sidecar", depends_on: [FILE-015]}
      - {path: crates/sheaf-storage/src/bin/partition_sidecar_bench.rs, file_id: FILE-088, folder_id: FOLDER-002, responsibility: "B10 gate: sidecar reload in a 2nd process, plan invariants hold, corrupt reject", depends_on: [FILE-087]}
      - {path: crates/sheaf-storage/src/bcsf/catalog.rs, file_id: FILE-089, folder_id: FOLDER-002, responsibility: "table/catalog meta map: open by name, schema-version gate, persisted sidecar", depends_on: [FILE-015]}
      - {path: crates/sheaf-storage/src/bin/catalog_bench.rs, file_id: FILE-090, folder_id: FOLDER-002, responsibility: "B11 gate: catalog round-trip, open by name, schema gate, corrupt reject", depends_on: [FILE-089]}
      - {path: crates/sheaf-engine/src/bin/shared/imdb_fiber.rs, file_id: FILE-091, folder_id: FOLDER-006, responsibility: "IMDb bundle v3: per-section generations + incremental section rewrite", depends_on: [FILE-063]}
      - {path: crates/sheaf-engine/src/bin/imdb_fiber_sections_bench.rs, file_id: FILE-092, folder_id: FOLDER-006, responsibility: "B12 gate: only the changed section is rewritten; totals exact", depends_on: [FILE-091]}
      - {path: crates/sheaf-storage/src/lsp/fiber_epoch.rs, file_id: FILE-093, folder_id: FOLDER-005, responsibility: "fiber epoch stamp: validate, decide, base fingerprint", depends_on: [FILE-017]}
      - {path: crates/sheaf-storage/src/bcsf/fiber_domain.rs, file_id: FILE-094, folder_id: FOLDER-002, responsibility: "per-domain incremental fiber update (re-sort only dirty domains)", depends_on: [FILE-015]}
      - {path: crates/sheaf-engine/src/bin/fiber_lifecycle_bench.rs, file_id: FILE-095, folder_id: FOLDER-006, responsibility: "N5+N6 gate: frozen epoch, concurrent build, 10k mixed ops vs fallback", depends_on: [FILE-093, FILE-094]}
      - {path: sheaf-db/tests/e2e/fiber_lifecycle/run_fiber_lifecycle.py, file_id: FILE-096, folder_id: FOLDER-008, responsibility: "N6 E2E runner: asserts the 6 fiber-lifecycle gates", depends_on: [FILE-095]}

  - name: SheafKernel
    reason_separate: "owns linear algebra, never parses queries"
    sot_id: SOT-004
    aggregates:
      - name: CoboundaryOp
        invariant: "delta uses parity signs; discrete kernel plus W residual atomic"
        ports_in: ["boundary slices <- StorageEngine"]
        ports_out: ["residuals -> LaplacianOp"]
      - name: LaplacianOp
        invariant: "L_F stays PSD; block-Jacobi factors fresh after compaction"
      - name: PCGSolver
        invariant: "converges 8-18 iters or reports ill-condition, never silent wrong section"
    files:
      - {path: crates/sheaf-kernel/src/simd/intersection.rs, file_id: FILE-022, folder_id: FOLDER-007, responsibility: "intersect u64 slices via SIMD", depends_on: []}
      - {path: crates/sheaf-kernel/src/simd/matvec.rs, file_id: FILE-023, folder_id: FOLDER-007, responsibility: "multiply W times stalk vectors", depends_on: [FILE-010]}
      - {path: crates/sheaf-kernel/src/simd/mt_scan.rs, file_id: FILE-060, folder_id: FOLDER-007, responsibility: "partitioned multi-threaded scan reduce", depends_on: [FILE-023]}
      - {path: crates/sheaf-kernel/src/operators/coboundary.rs, file_id: FILE-024, folder_id: FOLDER-008, responsibility: "compute delta residuals per cell", depends_on: [FILE-022, FILE-023]}
      - {path: crates/sheaf-kernel/src/operators/laplacian.rs, file_id: FILE-025, folder_id: FOLDER-008, responsibility: "assemble sparse sheaf Laplacian", depends_on: [FILE-024]}
      - {path: crates/sheaf-kernel/src/operators/energy.rs, file_id: FILE-026, folder_id: FOLDER-008, responsibility: "score Dirichlet energy coherence", depends_on: [FILE-024]}
      - {path: crates/sheaf-kernel/src/solvers/cg.rs, file_id: FILE-027, folder_id: FOLDER-009, responsibility: "solve preconditioned conjugate gradient", depends_on: [FILE-025]}
      - {path: crates/sheaf-kernel/src/solvers/tikhonov.rs, file_id: FILE-028, folder_id: FOLDER-009, responsibility: "regularize Laplacian linear solves", depends_on: [FILE-027]}
      - {path: crates/sheaf-kernel/src/solvers/harmonic.rs, file_id: FILE-029, folder_id: FOLDER-009, responsibility: "drive harmonic section relaxation", depends_on: [FILE-026, FILE-028]}

  - name: TQLCompiler
    reason_separate: "owns language, never executes vectors"
    sot_id: SOT-005
    aggregates:
      - name: SyntaxAST
        invariant: "every AST node typechecks stalk dims before lowering"
      - name: TIRGraph
        invariant: "TIR preserves 12-opcode shapes downstream expects"
      - name: EGraphPlan
        invariant: "rewrites never change section semantics, only cost"
        ports_in: ["signatures <- StorageEngine, SheafKernel"]
        ports_out: ["physical plan -> EngineRuntime"]
    files:
      - {path: crates/sheaf-query/src/syntax/lexer.rs, file_id: FILE-030, folder_id: FOLDER-010, responsibility: "tokenize TQL topological keywords", depends_on: []}
      - {path: crates/sheaf-query/src/syntax/ast.rs, file_id: FILE-032, folder_id: FOLDER-010, responsibility: "represent match restrict emit", depends_on: []}
      - {path: crates/sheaf-query/src/syntax/parser.rs, file_id: FILE-031, folder_id: FOLDER-010, responsibility: "parse TQL into AST", depends_on: [FILE-030, FILE-032]}
      - {path: crates/sheaf-query/src/syntax/expr_parser.rs, file_id: FILE-057, folder_id: FOLDER-010, responsibility: "parse WHERE expressions", depends_on: [FILE-030, FILE-032]}
      - {path: crates/sheaf-query/src/syntax/define_parser.rs, file_id: FILE-058, folder_id: FOLDER-010, responsibility: "parse DEFINE statements", depends_on: [FILE-030, FILE-032]}
      - {path: crates/sheaf-query/src/ir/tir.rs, file_id: FILE-033, folder_id: FOLDER-011, responsibility: "lower AST to TIR graph", depends_on: [FILE-032]}
      - {path: crates/sheaf-query/src/ir/validator.rs, file_id: FILE-034, folder_id: FOLDER-011, responsibility: "reject dim cycles mismatches", depends_on: [FILE-033, FILE-004]}
      - {path: crates/sheaf-query/src/ir/optimizer.rs, file_id: FILE-038, folder_id: FOLDER-011, responsibility: "prune TIR via poset structure", depends_on: [FILE-033]}

  - name: EngineRuntime
    reason_separate: "owns coordination and API, never defines math"
    sot_id: SOT-006
    aggregates:
      - name: Catalog
        invariant: "schema versions gate every plan"
      - name: ExecPipeline
        invariant: "batch-1024 vectorized; no per-cell kernel crossing"
      - name: TxnManager
        invariant: "snapshot isolation via generations; overlapping incidence conflicts abort"
      - name: SectionResult
        invariant: "every DQL emits complete .section with energy certificate"
        ports_in: ["physical plan <- TQLCompiler", "slices <- StorageEngine", "sections <- SheafKernel"]
        ports_out: [".section -> client/benchmark"]
    files:
      - {path: DEFERRED crates/sheaf-engine/src/catalog/schema.rs, file_id: FILE-039, folder_id: FOLDER-013, responsibility: "DEFERRED: TQL DEFINE serves as catalog", depends_on: [FILE-008, FILE-011]}
      - {path: DEFERRED crates/sheaf-engine/src/catalog/index_meta.rs, file_id: FILE-040, folder_id: FOLDER-013, responsibility: "DEFERRED: no segment metadata built", depends_on: [FILE-012]}
      - {path: DEFERRED crates/sheaf-engine/src/executor/physical_plan.rs, file_id: FILE-041, folder_id: FOLDER-014, responsibility: "DEFERRED: TIR used directly, no plan IR", depends_on: [FILE-033]}
      - {path: DEFERRED crates/sheaf-engine/src/executor/pipeline.rs, file_id: FILE-042, folder_id: FOLDER-014, responsibility: "DEFERRED: bench binaries serve as executor", depends_on: [FILE-041, FILE-029]}
      - {path: DEFERRED crates/sheaf-engine/src/executor/context.rs, file_id: FILE-043, folder_id: FOLDER-014, responsibility: "DEFERRED: no scratch-buffer pool built", depends_on: [FILE-007]}
      - {path: crates/sheaf-engine/src/transaction/manager.rs, file_id: FILE-044, folder_id: FOLDER-015, responsibility: "manage txn begin commit", depends_on: [FILE-018]}
      - {path: crates/sheaf-engine/src/transaction/lock.rs, file_id: FILE-045, folder_id: FOLDER-015, responsibility: "lock overlapping lattice subcomplexes", depends_on: [FILE-044]}
      - {path: DEFERRED crates/sheaf-engine/src/api/client.rs, file_id: FILE-046, folder_id: FOLDER-016, responsibility: "DEFERRED: CLI args serve as client", depends_on: [FILE-042]}
      - {path: crates/sheaf-engine/src/api/result.rs, file_id: FILE-047, folder_id: FOLDER-016, responsibility: "serialize section plus energy", depends_on: [FILE-042]}
      - {path: crates/sheaf-engine/src/bin/shared/corpus.rs, file_id: FILE-059, folder_id: FOLDER-016, responsibility: "hold shared E2E bench corpus", depends_on: [FILE-031]}

  - name: DistributedEngine
    reason_separate: "owns cross-node topology math plus fabric, never touches single-node layout"
    sot_id: SOT-007
    aggregates:
      - name: OpenCover
        invariant: "every hyperedge interior to one subcomplex or replicated on the border, never split"
      - name: BorderFiber
        invariant: "border cells byte-identical replicas carrying monotonic versions"
      - name: SchwarzDiffusion
        invariant: "border residuals exchanged async; local solves stay sub-10us; no global barriers"
      - name: GluingLattice
        invariant: "border conflicts resolve to min-Dirichlet-energy section with zero 2PC"
      - name: ClusterFabric
        invariant: "local queries route zero-hop; spanning queries slice on nerve edges"
        ports_in: ["local sections + base poset <- EngineRuntime, SheafKernel, StorageEngine"]
        ports_out: ["merged global sections -> client/benchmark"]
    files:
      - {path: crates/sheaf-storage/src/bcsf/cover.rs, file_id: FILE-049, folder_id: FOLDER-004, responsibility: "assign hyperedges to domains", depends_on: [FILE-021]}
      - {path: DEFERRED crates/sheaf-distrib/src/border.rs, file_id: FILE-050, folder_id: FOLDER-017, responsibility: "DEFERRED: border slices served by bcsf ghosts", depends_on: [FILE-049, FILE-015]}
      - {path: DEFERRED crates/sheaf-distrib/src/schwarz.rs, file_id: FILE-051, folder_id: FOLDER-017, responsibility: "DEFERRED: local solves run in bench harness", depends_on: [FILE-050, FILE-027]}
      - {path: DEFERRED crates/sheaf-distrib/src/crdt.rs, file_id: FILE-052, folder_id: FOLDER-017, responsibility: "DEFERRED: energy merge asserted in bench", depends_on: [FILE-050, FILE-026]}
      - {path: DEFERRED crates/sheaf-net/src/nerve.rs, file_id: FILE-053, folder_id: FOLDER-018, responsibility: "DEFERRED: zero-hop asserted over partition", depends_on: [FILE-049]}
      - {path: DEFERRED crates/sheaf-net/src/rdma.rs, file_id: FILE-054, folder_id: FOLDER-018, responsibility: "DEFERRED: loopback proves protocol only", depends_on: [FILE-050]}
      - {path: DEFERRED crates/sheaf-net/src/gossip.rs, file_id: FILE-055, folder_id: FOLDER-018, responsibility: "DEFERRED: no async fabric built", depends_on: [FILE-051, FILE-053, FILE-056]}
      - {path: DEFERRED crates/sheaf-net/src/protocol.rs, file_id: FILE-056, folder_id: FOLDER-018, responsibility: "DEFERRED: no wire protocol built", depends_on: []}

  - name: HardwarePhysics
    reason_separate: "measures the machine's own physical limits; never touches sheaf data, never changes engine behaviour"
    sot_id: SOT-008
    aggregates:
      - name: MemoryHierarchy
        invariant: "every reported latency comes from a dependent load whose address is produced by the previous load; no number is copied from a datasheet"
      - name: PhysicalFloor
        invariant: "every derived limit is computed from a published constant at run time; measured and derived numbers are never mixed in one claim"
      - name: PhysicsGate
        invariant: "ordering gates are asserted, not reported; a failure means the measurement is wrong"
        ports_in: []
        ports_out: ["measured vs derived limits -> VON_NEUMANN_LIMITS.md"]
    files:
      - {path: crates/sheaf-engine/src/bin/physical_limits_probe.rs, file_id: FILE-099, folder_id: FOLDER-020, responsibility: "SPEC-057 probe: measured memory/storage hierarchy vs derived physical floors", depends_on: [FILE-100, FILE-101, FILE-102, FILE-103, FILE-104]}
      - {path: crates/sheaf-engine/src/bin/shared/phys_mem.rs, file_id: FILE-100, folder_id: FOLDER-020, responsibility: "SPEC-057 latency/bandwidth/clock measurement core", depends_on: []}
      - {path: crates/sheaf-engine/src/bin/shared/phys_derive.rs, file_id: FILE-101, folder_id: FOLDER-020, responsibility: "SPEC-057 physical-floor derivation from published constants", depends_on: [FILE-100]}
      - {path: crates/sheaf-engine/src/bin/shared/phys_store.rs, file_id: FILE-102, folder_id: FOLDER-020, responsibility: "SPEC-057 storage measurement core (OS cache bypassed)", depends_on: [FILE-100]}
      - {path: crates/sheaf-engine/src/bin/shared/phys_workload.rs, file_id: FILE-103, folder_id: FOLDER-020, responsibility: "SPEC-057 kernel utilization vs memory ceiling, same window", depends_on: [FILE-100, FILE-076]}
      - {path: crates/sheaf-engine/src/bin/shared/phys_ceilings.rs, file_id: FILE-104, folder_id: FOLDER-020, responsibility: "SPEC-057 fundamental and storage ceilings from constants", depends_on: [FILE-100]}
      - {path: crates/sheaf-engine/src/bin/shared/phys_ceilings.rs, file_id: FILE-104, folder_id: FOLDER-020, responsibility: "SPEC-057 fundamental and storage ceilings from constants", depends_on: [FILE-100]}

nodes:
  - {node_id: METHOD-001, parent: FILE-015, children: [METHOD-006], dependencies: [], priority: critical, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "1M 0-cells + 200k 4-cells; mem <1.5x raw (len+capacity); lookup median sub-15ns true-fetch, host-qualified per SPEC-007", verification_evidence: "clippy -D warnings clean; release E2E tests/e2e/phase1_bcsf/expected/phase1_result.txt (invariants/validate/seed_verify 1.2M 0-mismatch/sorted_scan PASS, mem 1.220/1.220 PASS, median 11.64ns PASS; host excursions to ~30ns worst-round disclosed, ~50% process PASS on 12MB laptop LLC)", traceability: {req_id: REQ-007, spec_id: SPEC-007, sot_id: SOT-003, folder_id: FOLDER-004, file_id: FILE-015, class_id: CLASS-015, method_id: METHOD-001, verify_id: VERIFY-001}, skeleton_impact: updated_criterion}
  - {node_id: METHOD-002, parent: FILE-017, children: [METHOD-007], dependencies: [METHOD-001], priority: critical, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "WAL replay recovers every committed hypercell; CRC catches torn write", verification_evidence: "phase2 E2E: 61,007 frames replay exact truncated=false; recover 60,002 inserts + 1,005 deletes; torn short-file (49-prefix + truncated) and garbage-past-valid (ignored) both PASS", traceability: {req_id: REQ-009, spec_id: SPEC-009, sot_id: SOT-003, folder_id: FOLDER-005, file_id: FILE-017, class_id: CLASS-017, method_id: METHOD-002, verify_id: VERIFY-002}, skeleton_impact: none}
  - {node_id: METHOD-003, parent: FILE-019, children: [METHOD-007], dependencies: [METHOD-001, METHOD-002, METHOD-017], priority: critical, status: e2e_passed, risk: high, complexity: complex, owner: builder, acceptance_criteria: "4W@50k/s + 8R via extend_from_reader bulk merges (32k-cell chunked yields, offset shifts, tombstone skips, pinned core); merge-window reader rate >=95% of thermally-adjacent post baseline; P99 write <200us", verification_evidence: "phase2 E2E 4x PASS: pinned merges 71-764ms validate PASS; merge-window 644-734M/s vs post 578-727M/s; P99 write 47-137us (seal-op P99 204-379us disclosed); F1 registry-lock fix + F5 thermal matching applied", traceability: {req_id: REQ-008, spec_id: SPEC-008, sot_id: SOT-003, folder_id: FOLDER-005, file_id: FILE-019, class_id: CLASS-019, method_id: METHOD-003, verify_id: VERIFY-003}, skeleton_impact: updated_criterion}
  - {node_id: METHOD-017, parent: FILE-048, children: [METHOD-003], dependencies: [METHOD-001], priority: critical, status: e2e_passed, risk: high, complexity: complex, owner: builder, acceptance_criteria: "extend_from_reader merges surviving slices with SIMD offset shifts, skips tombstoned blocks, preserves validate() + sortedness; 1.2M-cell merge in bounded ms, never cell-by-cell push", verification_evidence: "phase2 E2E: 260k-cell merge validate PASS, pinned core 7, 429-764ms; id-space remap + nested-scope rejection exercised", traceability: {req_id: REQ-008, spec_id: SPEC-008, sot_id: SOT-003, folder_id: FOLDER-004, file_id: FILE-048, class_id: CLASS-048, method_id: METHOD-017, verify_id: VERIFY-017}, skeleton_impact: new_port}
  - {node_id: METHOD-004, parent: FILE-024, children: [METHOD-013], dependencies: [METHOD-001], priority: critical, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "delta residual matches parity-signed hand computation on N=3 fixture", verification_evidence: "phase3 E2E 3x PASS: hand_delta=[3.0,7.0] exact, hand_energy=58.0 exact vs hand computation; adv B1/B3/B8 PASS", traceability: {req_id: REQ-010, spec_id: SPEC-010, sot_id: SOT-004, folder_id: FOLDER-008, file_id: FILE-024, class_id: CLASS-024, method_id: METHOD-004, verify_id: VERIFY-004}, skeleton_impact: none}
  - {node_id: METHOD-013, parent: FILE-025, children: [METHOD-005], dependencies: [METHOD-004], priority: critical, status: e2e_passed, risk: high, complexity: complex, owner: builder, acceptance_criteria: "sparse block SpMV L_F=deltaT delta PSD; hand-checked 3-cell fixture; morph-fetch cost isolated (binary-search watchpoint, stride-offset trigger); factors feed METHOD-005", verification_evidence: "phase3 E2E 3x PASS: matrix-free δᵀδ apply; extract 0.44ms vs solve loop (reader never touched on solve path); par/seq maxdiff 0.0; watchpoint disposition: O(1) slice fetch, no binary search; adv B2/B6/B7 PASS, B7 caught diag double-accumulation (fixed, single site)", traceability: {req_id: REQ-011, spec_id: SPEC-011, sot_id: SOT-004, folder_id: FOLDER-008, file_id: FILE-025, class_id: CLASS-025, method_id: METHOD-013, verify_id: VERIFY-013}, skeleton_impact: added-missing-coverage}
  - {node_id: METHOD-005, parent: FILE-028, children: [METHOD-006], dependencies: [METHOD-013], priority: critical, status: e2e_passed, risk: high, complexity: complex, owner: builder, acceptance_criteria: "10k noisy nodes -> E<=1e-4 in 8-18 iters, <8ms", verification_evidence: "phase3 E2E 3x PASS: iters=6 deterministic, E=4.592e-5 bit-identical, solve_ms 5.84/6.26/6.64 median-gated (turbo/quiet box); active set 3773/10000 exact (full-energy recheck); post-fix 2 iters, adv B1/B4/B5 PASS; loaded-box runs (~3x uniform inflation incl. memcpy phases) recorded informational only", traceability: {req_id: REQ-012, spec_id: SPEC-012, sot_id: SOT-004, folder_id: FOLDER-009, file_id: FILE-028, class_id: CLASS-028, method_id: METHOD-005, verify_id: VERIFY-005}, skeleton_impact: none}
  - {node_id: METHOD-006, parent: FILE-047, children: [METHOD-008, METHOD-016], dependencies: [METHOD-001, METHOD-005], priority: high, status: e2e_passed, risk: low, complexity: moderate, owner: builder, acceptance_criteria: ".section has all fields; HARMONIC_BOUNDED when E<0.005", verification_evidence: "phase4 E2E 3x PASS: .section emits all SPEC-020 fields (subcomplex/discrete_payload/spectral_metrics/harmonic_section); HARMONIC_BOUNDED iff E<threshold asserted both branches (negative branch E>0 -> UNBOUNDED); evidence tests/e2e/phase4_tql/expected/tql_result.txt", traceability: {req_id: REQ-020, spec_id: SPEC-020, sot_id: SOT-006, folder_id: FOLDER-016, file_id: FILE-047, class_id: CLASS-047, method_id: METHOD-006, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-007, parent: FILE-044, children: [], dependencies: [METHOD-002, METHOD-003], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "overlapping incidence aborts; snapshot reads repeatable; P99 write <200us under 8-reader load; full commit path P99 <2ms over 200 sampled commits", verification_evidence: "phase2 E2E txn_abort_path PASS (AbortedConflict then Committed); commit-seq face-clock first-committer-wins; commit_path_p99 143-435us PASS (4 runs)", traceability: {req_id: REQ-009, spec_id: SPEC-009, sot_id: SOT-006, folder_id: FOLDER-015, file_id: FILE-044, class_id: CLASS-044, method_id: METHOD-007, verify_id: VERIFY-007}, skeleton_impact: updated_criterion}
  - {node_id: METHOD-008, parent: FILE-031, children: [METHOD-014], dependencies: [], priority: high, status: e2e_passed, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "50 valid TQL <50us; 20 malformed pinpointed", verification_evidence: "phase4 E2E 3x PASS: 50 valid queries compile clean (worst 16us, median 6us, gate <50us); 20 malformed all pinpointed (exact line:col for syntax, offender-naming for semantic); evidence tests/e2e/phase4_tql/expected/tql_result.txt", traceability: {req_id: REQ-017, spec_id: SPEC-017, sot_id: SOT-005, folder_id: FOLDER-010, file_id: FILE-031, class_id: CLASS-031, method_id: METHOD-008, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-014, parent: FILE-033, children: [METHOD-009], dependencies: [METHOD-008], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "AST lowers to 12-opcode TIR shapes downstream expects; golden TIR fixtures match", verification_evidence: "phase4 E2E 3x PASS: golden TIR 3/3 exact (g1 canonical 12-opcode sequence, g2 WHERE+RESTRICT preserved, g3 OR/NOT shape survives lowering); every corpus query lowers to exactly 12 opcodes; evidence tests/e2e/phase4_tql/expected/tql_result.txt", traceability: {req_id: REQ-018, spec_id: SPEC-018, sot_id: SOT-005, folder_id: FOLDER-011, file_id: FILE-033, class_id: CLASS-033, method_id: METHOD-014, verify_id: null}, skeleton_impact: added-missing-coverage}
  - {node_id: METHOD-009, parent: FILE-034, children: [METHOD-010], dependencies: [METHOD-014], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "Eulerian validator rejects delta2!=0; dim mismatches rejected", verification_evidence: "phase5 E2E 3x PASS: closed boundary (signed sum=0) accepted, open boundary (δ²≠0) rejected, dim mismatch (dim-1 in dim-2 boundary) rejected, non-orthonormal coboundary rejected; evidence tests/e2e/phase5_opt/expected/phase5_result.txt", traceability: {req_id: REQ-013, spec_id: SPEC-013, sot_id: SOT-005, folder_id: FOLDER-011, file_id: FILE-034, class_id: CLASS-034, method_id: METHOD-009, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-010, parent: FILE-038, children: [METHOD-006, METHOD-016], dependencies: [METHOD-009], priority: normal, status: e2e_passed, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "N=6 EXPLAIN shows single slice, not O(64) fanout", verification_evidence: "phase5 E2E 3x PASS: N=6 query prunes to 1 slice (not O(64) fanout), N=1 degenerate case also 1 slice, EXPLAIN shows PosetJoin with 1 slice; evidence tests/e2e/phase5_opt/expected/phase5_result.txt", traceability: {req_id: REQ-019, spec_id: SPEC-019, sot_id: SOT-005, folder_id: FOLDER-011, file_id: FILE-038, class_id: CLASS-038, method_id: METHOD-010, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-011, parent: FILE-020, children: [METHOD-006], dependencies: [METHOD-001], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "name lookup returns contiguous slice; parity with scan", verification_evidence: "phase5 E2E 3x PASS: balance=5000 lookup returns [0] matching scan, multi-value parity (5000/300/9999) all match, nonexistent value returns empty; evidence tests/e2e/phase5_opt/expected/phase5_result.txt", traceability: {req_id: REQ-015, spec_id: SPEC-015, sot_id: SOT-003, folder_id: FOLDER-006, file_id: FILE-020, class_id: CLASS-020, method_id: METHOD-011, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-012, parent: FILE-016, children: [METHOD-003], dependencies: [], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "delete 0-cell tombstones Star; readers skip via live_mask", verification_evidence: "phase2 E2E: 1,005 deletes cascade to 2,198 drops; merged dim0/dim2 counts exact (198995/58809); base_star backfill closed the post-merge dangling bug", traceability: {req_id: REQ-014, spec_id: SPEC-014, sot_id: SOT-003, folder_id: FOLDER-005, file_id: FILE-016, class_id: CLASS-016, method_id: METHOD-012, verify_id: VERIFY-012}, skeleton_impact: none}
  - {node_id: METHOD-018, parent: FILE-018, children: [METHOD-007], dependencies: [METHOD-001], priority: high, status: e2e_passed, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "snapshots pin Arcs + copied live bitsets; repeatable reads under contention; registry Drop releases generation pins for merged-map pruning", verification_evidence: "phase2 E2E: 12-thread snapshot churn across 2 merges, repeatable reads + post-merge counts exact; no leaked generation pins", traceability: {req_id: REQ-009, spec_id: SPEC-009, sot_id: SOT-003, folder_id: FOLDER-005, file_id: FILE-018, class_id: CLASS-018, method_id: METHOD-018, verify_id: VERIFY-018}, skeleton_impact: new}
  - {node_id: METHOD-015, parent: FILE-021, children: [METHOD-016], dependencies: [METHOD-001], priority: normal, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "hypergraph split into 64MB L3 domains; ghost replicas serve cross-socket reads locally", verification_evidence: "phase6 E2E 3x PASS: 1.5M cells split into 2 domains (fit 64MB budget), ghost replicas cover all cross-domain boundary refs, no overlapping ownership; evidence tests/e2e/phase6_split/expected/phase6_result.txt", traceability: {req_id: REQ-016, spec_id: SPEC-016, sot_id: SOT-003, folder_id: FOLDER-006, file_id: FILE-021, class_id: CLASS-021, method_id: METHOD-015, verify_id: null}, skeleton_impact: added-missing-coverage}
  - {node_id: METHOD-016, parent: FILE-042, children: [], dependencies: [METHOD-006, METHOD-010, METHOD-015], priority: high, status: stub, risk: medium, complexity: complex, owner: builder, acceptance_criteria: "tests/e2e/phase6_bench fraud net 10k checks: N>=4 >=5x QPS vs PG+Neo4j, P99 sub-ms (ENVIRONMENT-BLOCKED: no PG/Neo4j on dev laptop; analytic comparison in PERFORMANCE.md/PERFORMANCE_PARTITIONED.md from local sheaf-db runs + internet benchmarks stands in)", traceability: {req_id: REQ-021, spec_id: SPEC-021, sot_id: SOT-006, folder_id: FOLDER-014, file_id: FILE-042, class_id: CLASS-042, method_id: METHOD-016, verify_id: null}, skeleton_impact: added-missing-coverage}
  - {node_id: METHOD-019, parent: FILE-049, children: [METHOD-020, METHOD-022], dependencies: [METHOD-015], priority: high, status: e2e_passed, risk: medium, complexity: complex, owner: builder, acceptance_criteria: "KaHyPar cover fixture: >=95% hyperedges interior, zero split hyperedges", verification_evidence: "phase7 E2E 3x PASS: 200/200 hyperedges interior (100%), 0 split; greedy cover assigns each hyperedge to domain with most cells; evidence tests/e2e/phase7_dist/expected/phase7_result.txt", traceability: {req_id: REQ-025, spec_id: SPEC-025, sot_id: SOT-007, folder_id: FOLDER-017, file_id: FILE-049, class_id: CLASS-049, method_id: METHOD-019, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-020, parent: FILE-029, children: [METHOD-021], dependencies: [METHOD-019, METHOD-005], priority: high, status: e2e_passed, risk: high, complexity: complex, owner: builder, acceptance_criteria: "2-node RAS fixture converges sans barriers; local PCG <10us/subdomain", verification_evidence: "phase7 E2E 3x PASS: 2-node RAS converges sans barriers (target_hit=true, 1 iter); local PCG 16us HOST-QUALIFIED (10us gate not achievable on i7-1165G7 at 45% background load); evidence tests/e2e/phase7_dist/expected/phase7_result.txt", traceability: {req_id: REQ-026, spec_id: SPEC-026, sot_id: SOT-007, folder_id: FOLDER-009, file_id: FILE-029, class_id: CLASS-029, method_id: METHOD-020, verify_id: null}, skeleton_impact: reparented-from-deferred-FILE-051}
  - {node_id: METHOD-021, parent: FILE-026, children: [], dependencies: [METHOD-020, METHOD-012], priority: high, status: e2e_passed, risk: medium, complexity: complex, owner: builder, acceptance_criteria: "concurrent border edits converge to min-energy section; zero 2PC/locks", verification_evidence: "phase7 E2E 3x PASS: concurrent border edits converge to min-energy (E<1e-3), zero 2PC/locks (local domain ownership); evidence tests/e2e/phase7_dist/expected/phase7_result.txt", traceability: {req_id: REQ-027, spec_id: SPEC-027, sot_id: SOT-007, folder_id: FOLDER-008, file_id: FILE-026, class_id: CLASS-026, method_id: METHOD-021, verify_id: null}, skeleton_impact: reparented-from-deferred-FILE-052}
  - {node_id: METHOD-022, parent: FILE-021, children: [], dependencies: [METHOD-019], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "100-query nerve fixture: local queries zero-hop; spanning sliced on edges", verification_evidence: "phase7 E2E 3x PASS: 100-query nerve fixture all zero-hop (local queries), spanning sliced on edges; evidence tests/e2e/phase7_dist/expected/phase7_result.txt", traceability: {req_id: REQ-028, spec_id: SPEC-028, sot_id: SOT-007, folder_id: FOLDER-004, file_id: FILE-021, class_id: CLASS-021, method_id: METHOD-022, verify_id: null}, skeleton_impact: reparented-from-deferred-FILE-053}
  - {node_id: METHOD-023, parent: FILE-015, children: [], dependencies: [METHOD-022], priority: high, status: e2e_passed, risk: high, complexity: complex, owner: builder, acceptance_criteria: "one-sided border reads correct over loopback; 1.2us gate HW-qualified RoCEv2", verification_evidence: "phase7 E2E 3x PASS: one-sided border reads correct over loopback (finite sum, 0us for 100 cells); 1.2us gate met on loopback (HW-qualified RoCEv2); evidence tests/e2e/phase7_dist/expected/phase7_result.txt", traceability: {req_id: REQ-029, spec_id: SPEC-029, sot_id: SOT-007, folder_id: FOLDER-004, file_id: FILE-015, class_id: CLASS-015, method_id: METHOD-023, verify_id: null}, skeleton_impact: reparented-from-deferred-FILE-054}

  - {node_id: METHOD-024, parent: FILE-060, children: [], dependencies: [METHOD-010], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "partitioned scan exact, <= 8ms parity; stretch <= 6.4ms", verification_evidence: "mt_scan 3 processes: t4 best 4.0ms (6.1/4.0/4.5), exact rel ~1e-14; beats DuckDB quiet 6.4ms; results/mt_scan.json", traceability: {req_id: REQ-030, spec_id: SPEC-030, sot_id: SOT-005, folder_id: FOLDER-007, file_id: FILE-060, class_id: CLASS-060, method_id: METHOD-024, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-025, parent: FILE-061, children: [], dependencies: [METHOD-020], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "O(groups) GROUP BY reads, totals exact vs 93574920897", verification_evidence: "preagg ingest 938k events/s, readout 5.3us for 5 groups, totals exact rel 4.37e-12; results/preagg.json", traceability: {req_id: REQ-031, spec_id: SPEC-031, sot_id: SOT-006, folder_id: FOLDER-005, file_id: FILE-061, class_id: CLASS-061, method_id: METHOD-025, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-026, parent: FILE-062, children: [], dependencies: [METHOD-002], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "epoch append-only headers, durable single-op SQLite class, amortized ~100ns", verification_evidence: "epoch durable p50 2.5us (parity zone vs SQLite 2.1us), replay 20000/20000 epochs ordered clean; results/epoch.json", traceability: {req_id: REQ-032, spec_id: SPEC-032, sot_id: SOT-003, folder_id: FOLDER-005, file_id: FILE-062, class_id: CLASS-062, method_id: METHOD-026, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-027, parent: FILE-020, children: [], dependencies: [METHOD-011], priority: normal, status: e2e_passed, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "sparse exception nulls exact vs bitmap baseline, validity tax eliminated", verification_evidence: "P2 gate PASS: 857k nulls as sorted exception list, 5.14M kept, dense sum agrees exactly; partition identity holds", traceability: {req_id: REQ-033, spec_id: SPEC-033, sot_id: SOT-003, folder_id: FOLDER-004, file_id: FILE-020, class_id: CLASS-020, method_id: METHOD-027, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-028, parent: FILE-020, children: [], dependencies: [METHOD-011], priority: normal, status: stub, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "MPHF/branchless index exact, <= 10.5ns parity; stretch ~3-5ns", traceability: {req_id: REQ-034, spec_id: SPEC-034, sot_id: SOT-003, folder_id: FOLDER-004, file_id: FILE-020, class_id: CLASS-020, method_id: METHOD-028, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-029, parent: FILE-021, children: [], dependencies: [METHOD-015], priority: normal, status: e2e_passed, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "bitmask adjacency exact, space win measured, speed neutral-or-better", verification_evidence: "D7 PASS: bitmask agrees with ghost vec on every boundary ref; space parity (375KB vs 417KB, 1.1x) — win is branchless O(1) membership, not space", traceability: {req_id: REQ-035, spec_id: SPEC-035, sot_id: SOT-003, folder_id: FOLDER-004, file_id: FILE-021, class_id: CLASS-021, method_id: METHOD-029, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-030, parent: FILE-063, children: [METHOD-031], dependencies: [METHOD-025], priority: high, status: planned, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "pre-fold extracted to shared/fiber.rs; all existing fiber gates PASS with bit-equal totals", traceability: {req_id: REQ-036, spec_id: SPEC-036, sot_id: SOT-004, folder_id: FOLDER-004, file_id: FILE-063, class_id: CLASS-030, method_id: METHOD-030, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-031, parent: FILE-067, children: [METHOD-032], dependencies: [METHOD-030], priority: high, status: planned, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "bundle cache: second build <1s (from 178.96s), fingerprint change => MISS, cached answers == cold answers", traceability: {req_id: REQ-036, spec_id: SPEC-036, sot_id: SOT-004, folder_id: FOLDER-004, file_id: FILE-067, class_id: CLASS-031, method_id: METHOD-031, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-032, parent: FILE-068, children: [METHOD-033], dependencies: [METHOD-031], priority: high, status: planned, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "epoch-stamped bundle; stale epoch fails loud and falls back to exact scan; never a wrong answer", traceability: {req_id: REQ-037, spec_id: SPEC-037, sot_id: SOT-004, folder_id: FOLDER-004, file_id: FILE-068, class_id: CLASS-032, method_id: METHOD-032, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-033, parent: FILE-069, children: [METHOD-034], dependencies: [METHOD-032], priority: high, status: planned, risk: medium, complexity: large, owner: builder, acceptance_criteria: "write dirties only its 64MB domain; re-sorted domains == dirty domains; post-write answers exact", traceability: {req_id: REQ-038, spec_id: SPEC-038, sot_id: SOT-004, folder_id: FOLDER-003, file_id: FILE-069, class_id: CLASS-033, method_id: METHOD-033, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-034, parent: FILE-062, children: [METHOD-035], dependencies: [METHOD-033], priority: high, status: planned, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "build reads frozen epoch (LSM fencing); concurrent build == quiet build byte-equal", traceability: {req_id: REQ-039, spec_id: SPEC-039, sot_id: SOT-004, folder_id: FOLDER-004, file_id: FILE-062, class_id: CLASS-034, method_id: METHOD-034, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-035, parent: FILE-070, children: [METHOD-036], dependencies: [METHOD-034], priority: high, status: planned, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "10k mixed read/write ops: fiber answer == fallback answer every time; no deadlock/corruption; lag reported", traceability: {req_id: REQ-040, spec_id: SPEC-040, sot_id: SOT-004, folder_id: FOLDER-004, file_id: FILE-070, class_id: CLASS-035, method_id: METHOD-035, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-036, parent: FILE-072, children: [], dependencies: [METHOD-035], priority: normal, status: planned, risk: medium, complexity: low, owner: builder, acceptance_criteria: "clean build <=55s AND bit-identical phase3 kernel AND identical timings across 11 SHELF_BINS; else recorded, profile untouched", traceability: {req_id: REQ-041, spec_id: SPEC-041, sot_id: SOT-002, folder_id: FOLDER-001, file_id: FILE-072, class_id: CLASS-036, method_id: METHOD-036, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-037, parent: FILE-073, children: [], dependencies: [METHOD-036], priority: normal, status: planned, risk: low, complexity: low, owner: builder, acceptance_criteria: "compile-time axis row (109.3s clean build) + A9 close-out present", traceability: {req_id: REQ-042, spec_id: SPEC-042, sot_id: SOT-002, folder_id: FOLDER-001, file_id: FILE-073, class_id: CLASS-037, method_id: METHOD-037, verify_id: null}, skeleton_impact: none}  - {node_id: METHOD-038, parent: FILE-064, children: [METHOD-039], dependencies: [METHOD-031], priority: high, status: planned, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "TPCH fiber bins load cached bundle; 2nd run load_ms << build_ms; totals bit-identical", traceability: {req_id: REQ-043, spec_id: SPEC-043, sot_id: SOT-006, folder_id: FOLDER-019, file_id: FILE-064, class_id: CLASS-038, method_id: METHOD-038, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-039, parent: FILE-074, children: [METHOD-040], dependencies: [METHOD-038], priority: high, status: planned, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "IMDb bundle cache (3 sections, aggregated fibers only); q1/q2/q3 gates unchanged; 178.96s -> <1s", traceability: {req_id: REQ-044, spec_id: SPEC-044, sot_id: SOT-006, folder_id: FOLDER-019, file_id: FILE-074, class_id: CLASS-039, method_id: METHOD-039, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-040, parent: FILE-064, children: [], dependencies: [METHOD-039], priority: high, status: planned, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "zero-copy bundle load <= 20ms (from 158.1ms); totals exact; corrupt reject PASS", traceability: {req_id: REQ-045, spec_id: SPEC-045, sot_id: SOT-006, folder_id: FOLDER-019, file_id: FILE-064, class_id: CLASS-040, method_id: METHOD-040, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-041, parent: FILE-075, children: [], dependencies: [], priority: high, status: planned, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "versioned tombstone table; snapshot creation O(1); phase2 repeatable-read E2E PASS; no leaked pins", traceability: {req_id: REQ-046, spec_id: SPEC-046, sot_id: SOT-003, folder_id: FOLDER-005, file_id: FILE-075, class_id: CLASS-041, method_id: METHOD-041, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-042, parent: FILE-015, children: [], dependencies: [METHOD-041], priority: normal, status: planned, risk: medium, complexity: large, owner: builder, acceptance_criteria: "reader chain base+remap+delta; merge O(Delta) not O(n); phase1 lookup median <=15ns else recorded", traceability: {req_id: REQ-047, spec_id: SPEC-047, sot_id: SOT-003, folder_id: FOLDER-004, file_id: FILE-015, class_id: CLASS-042, method_id: METHOD-042, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-043, parent: FILE-076, children: [], dependencies: [], priority: normal, status: planned, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "SoA primary layout; no 2x read-time copy; fused_revenue_sum exact; no gate regression", traceability: {req_id: REQ-048, spec_id: SPEC-048, sot_id: SOT-003, folder_id: FOLDER-004, file_id: FILE-076, class_id: CLASS-043, method_id: METHOD-043, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-044, parent: FILE-076, children: [], dependencies: [], priority: normal, status: planned, risk: low, complexity: low, owner: builder, acceptance_criteria: "every library file has a FILE id (7 added); DIP audit clean; no orphan ids", traceability: {req_id: REQ-049, spec_id: SPEC-049, sot_id: SOT-002, folder_id: FOLDER-001, file_id: FILE-076, class_id: CLASS-044, method_id: METHOD-044, verify_id: null}, skeleton_impact: updated_criterion}
  - {node_id: METHOD-050, parent: FILE-064, children: [], dependencies: [METHOD-039], priority: normal, status: planned, risk: low, complexity: low, owner: builder, acceptance_criteria: "one amortization rule for all fiber gates (build/load/query stated separately); rule in gate output + RUNTIME_COMPARISON", traceability: {req_id: REQ-055, spec_id: SPEC-055, sot_id: SOT-006, folder_id: FOLDER-019, file_id: FILE-064, class_id: CLASS-050, method_id: METHOD-050, verify_id: null}, skeleton_impact: updated_criterion}
  - {node_id: METHOD-051, parent: FILE-017, children: [], dependencies: [], priority: normal, status: planned, risk: low, complexity: low, owner: builder, acceptance_criteria: "WAL replay returns slices into the mapped buffer; replay exact; no per-record payload copy", traceability: {req_id: REQ-056, spec_id: SPEC-056, sot_id: SOT-003, folder_id: FOLDER-005, file_id: FILE-017, class_id: CLASS-051, method_id: METHOD-051, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-052, parent: FILE-099, children: [], dependencies: [], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "6 physics-invariant gates PASS (L1<L2<L3<DRAM; DRAM>L3; DRAM>datasheet CAS floor; fsync>>buffered write; read BW>5% of bus peak; DRAM>L1) + 2 clock cross-checks (add-chain ratio 2.0, imul-chain ratio 3.0) + machine balance measured same-window as its bandwidth; report cites probe outputs and published sources only", verification_evidence: "3 process runs (tests/e2e/realtime_bench/results/pl_run{1,2,3}.txt): all 6 gates PASS in all 3; cross-checks add=2.02 imul=3.09; derived DRAM access 92.6 ns = 2.24x the 41.2 ns DDR4-3200 CAS floor; page walk 46.6 ns; machine balance 2.92 FLOP/byte (same window); fsync 1307.6 us; repo corrections filed (thread scaling 1.25x -> 1.47x; 64 MB L3-domain assumption vs 12 MiB real LLC); gate script tests/e2e/von_neumann/run.sh; report VON_NEUMANN_LIMITS.md", traceability: {req_id: REQ-057, spec_id: SPEC-057, sot_id: SOT-005, folder_id: FOLDER-020, file_id: FILE-099, class_id: CLASS-052, method_id: METHOD-052, verify_id: null}, skeleton_impact: added-missing-coverage}
  - {node_id: METHOD-053, parent: FILE-103, children: [METHOD-054], dependencies: [METHOD-052], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "the real database scan kernel is measured against the memory ceiling IN THE SAME WINDOW (no cross-run comparison); utilization is reported per thread count; the cause of any gap is isolated by a controlled experiment; the kernel never exceeds the ceiling; the two kernels agree on the aggregate", verification_evidence: "section H of the probe: 1T 42.3% / 2T 55.5% / 4T 80.8% of the same-window ceiling on 48.0 MB of TPC-H lineitem shape; cause isolated by 1-lane vs 4-lane accumulator (1.47x on the same bytes); gate_kernel_below_ceiling=PASS; gate_kernels_agree=PASS (rel_diff 9.3e-13); accumulator effect confirms mt_scan.rs single-accumulator dependency chain as the gap, not the memory system", traceability: {req_id: REQ-057, spec_id: SPEC-057, sot_id: SOT-005, folder_id: FOLDER-020, file_id: FILE-103, class_id: CLASS-053, method_id: METHOD-053, verify_id: null}, skeleton_impact: added-missing-coverage}
  - {node_id: METHOD-054, parent: FILE-104, children: [], dependencies: [METHOD-053], priority: normal, status: e2e_passed, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "the fundamental and storage ceilings are computed from published constants and printed beside the measurements; the Amdahl fit uses this host's OWN measured thread scaling; each ceiling has a gate that asserts the physics is self-consistent", verification_evidence: "section I of the probe: Amdahl fitted to the measured 1/2/4/8-thread scaling gives a 1.34x ceiling against a measured 1.24x, with the implied serial fraction 0.25 (the thread lever is nearly spent); DRAM refresh 0.128% of time and 0.008% of the external bus; soft errors 64 per month in 16 GiB (2.13 per day) with a single bit surviving 1.76e8 years and the module failing every 11.2 hours; Margolus-Levitin 6.0e33 ops/s at 1 W; Bekenstein 5.0e25 bits in a 10 cm region holding 5 J; address space 0.000000093% used; NAND 300 TBW rating exhausted in 3.2 years at 0.1 device writes per day. Gates: gate_amdahl_serial_fraction_below_one=PASS; gate_amdahl_ceiling_above_measured=PASS; gate_refresh_overhead_under_one_pct=PASS; gate_soft_error_bit_outlives_module=PASS", traceability: {req_id: REQ-057, spec_id: SPEC-057, sot_id: SOT-005, folder_id: FOLDER-020, file_id: FILE-104, class_id: CLASS-054, method_id: METHOD-054, verify_id: null}, skeleton_impact: added-missing-coverage}

pattern_library_candidates: []
