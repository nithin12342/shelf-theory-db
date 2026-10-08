# SKELETON.md — sheaf-db (Planning gate artifact)

system: sheaf-db
status: planning-corrected plus METHOD-001 e2e_passed plus Phase-2 LSP e2e_passed (METHOD-002/003/007/012/017/018; 4x PASS incl. F1-F5 fixes; adversarial A1-A8 PASS 3x; distributed REQ-025..029 planned Phase-7+, METHOD-019..023 stubs; DIP audit clean, 2 waived W-D1/W-D2; next: METHOD-004/013/005 Phase-3 kernel)

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
      - {path: crates/sheaf-storage/src/index/tifi.rs, file_id: FILE-020, folder_id: FOLDER-006, responsibility: "map attribute values to cells", depends_on: [FILE-005, FILE-013]}
      - {path: crates/sheaf-storage/src/index/partition.rs, file_id: FILE-021, folder_id: FOLDER-006, responsibility: "partition hypergraph NUMA domains", depends_on: [FILE-013]}

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
      - {path: crates/sheaf-query/src/ir/tir.rs, file_id: FILE-033, folder_id: FOLDER-011, responsibility: "lower AST to TIR graph", depends_on: [FILE-032]}
      - {path: crates/sheaf-query/src/ir/validator.rs, file_id: FILE-034, folder_id: FOLDER-011, responsibility: "reject dim cycles mismatches", depends_on: [FILE-033, FILE-004]}
      - {path: crates/sheaf-query/src/optimizer/egraph.rs, file_id: FILE-035, folder_id: FOLDER-012, responsibility: "hold equivalence plan classes", depends_on: [FILE-033]}
      - {path: crates/sheaf-query/src/optimizer/rules_boundary.rs, file_id: FILE-036, folder_id: FOLDER-012, responsibility: "prune via partial-partial zero", depends_on: [FILE-035]}
      - {path: crates/sheaf-query/src/optimizer/rules_pushdown.rs, file_id: FILE-037, folder_id: FOLDER-012, responsibility: "push discrete filters early", depends_on: [FILE-035]}
      - {path: crates/sheaf-query/src/optimizer/cost.rs, file_id: FILE-038, folder_id: FOLDER-012, responsibility: "choose scan versus solve", depends_on: [FILE-035]}

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
      - {path: crates/sheaf-engine/src/catalog/schema.rs, file_id: FILE-039, folder_id: FOLDER-013, responsibility: "register cell type schemas", depends_on: [FILE-008, FILE-011]}
      - {path: crates/sheaf-engine/src/catalog/index_meta.rs, file_id: FILE-040, folder_id: FOLDER-013, responsibility: "track segment offset metadata", depends_on: [FILE-012]}
      - {path: crates/sheaf-engine/src/executor/physical_plan.rs, file_id: FILE-041, folder_id: FOLDER-014, responsibility: "represent executable plan steps", depends_on: [FILE-033]}
      - {path: crates/sheaf-engine/src/executor/pipeline.rs, file_id: FILE-042, folder_id: FOLDER-014, responsibility: "run batched vectorized pipeline", depends_on: [FILE-041, FILE-029]}
      - {path: crates/sheaf-engine/src/executor/context.rs, file_id: FILE-043, folder_id: FOLDER-014, responsibility: "hold scratch execution buffers", depends_on: [FILE-007]}
      - {path: crates/sheaf-engine/src/transaction/manager.rs, file_id: FILE-044, folder_id: FOLDER-015, responsibility: "manage txn begin commit", depends_on: [FILE-018]}
      - {path: crates/sheaf-engine/src/transaction/lock.rs, file_id: FILE-045, folder_id: FOLDER-015, responsibility: "lock overlapping lattice subcomplexes", depends_on: [FILE-044]}
      - {path: crates/sheaf-engine/src/api/client.rs, file_id: FILE-046, folder_id: FOLDER-016, responsibility: "accept client query sessions", depends_on: [FILE-042]}
      - {path: crates/sheaf-engine/src/api/result.rs, file_id: FILE-047, folder_id: FOLDER-016, responsibility: "serialize section plus energy", depends_on: [FILE-042]}

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
      - {path: crates/sheaf-distrib/src/cover.rs, file_id: FILE-049, folder_id: FOLDER-017, responsibility: "partition poset into overlapping subcomplexes", depends_on: [FILE-003, FILE-021]}
      - {path: crates/sheaf-distrib/src/border.rs, file_id: FILE-050, folder_id: FOLDER-017, responsibility: "represent replicated border fiber slices", depends_on: [FILE-049, FILE-015]}
      - {path: crates/sheaf-distrib/src/schwarz.rs, file_id: FILE-051, folder_id: FOLDER-017, responsibility: "diffuse border residuals without barriers", depends_on: [FILE-050, FILE-027]}
      - {path: crates/sheaf-distrib/src/crdt.rs, file_id: FILE-052, folder_id: FOLDER-017, responsibility: "merge border edits by energy", depends_on: [FILE-050, FILE-026]}
      - {path: crates/sheaf-net/src/nerve.rs, file_id: FILE-053, folder_id: FOLDER-018, responsibility: "route queries along nerve graph", depends_on: [FILE-049]}
      - {path: crates/sheaf-net/src/rdma.rs, file_id: FILE-054, folder_id: FOLDER-018, responsibility: "read remote border arenas one-sided", depends_on: [FILE-050]}
      - {path: crates/sheaf-net/src/gossip.rs, file_id: FILE-055, folder_id: FOLDER-018, responsibility: "exchange border residuals asynchronously", depends_on: [FILE-051, FILE-053, FILE-056]}
      - {path: crates/sheaf-net/src/protocol.rs, file_id: FILE-056, folder_id: FOLDER-018, responsibility: "frame cross-node sync bytes", depends_on: []}

nodes:
  - {node_id: METHOD-001, parent: FILE-015, children: [METHOD-006], dependencies: [], priority: critical, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "1M 0-cells + 200k 4-cells; mem <1.5x raw (len+capacity); lookup median sub-15ns true-fetch, host-qualified per SPEC-007", verification_evidence: "clippy -D warnings clean; release E2E tests/e2e/phase1_bcsf/expected/phase1_result.txt (invariants/validate/seed_verify 1.2M 0-mismatch/sorted_scan PASS, mem 1.220/1.220 PASS, median 11.64ns PASS; host excursions to ~30ns worst-round disclosed, ~50% process PASS on 12MB laptop LLC)", traceability: {req_id: REQ-007, spec_id: SPEC-007, sot_id: SOT-003, folder_id: FOLDER-004, file_id: FILE-015, class_id: CLASS-015, method_id: METHOD-001, verify_id: VERIFY-001}, skeleton_impact: updated_criterion}
  - {node_id: METHOD-002, parent: FILE-017, children: [METHOD-007], dependencies: [METHOD-001], priority: critical, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "WAL replay recovers every committed hypercell; CRC catches torn write", verification_evidence: "phase2 E2E: 61,007 frames replay exact truncated=false; recover 60,002 inserts + 1,005 deletes; torn short-file (49-prefix + truncated) and garbage-past-valid (ignored) both PASS", traceability: {req_id: REQ-009, spec_id: SPEC-009, sot_id: SOT-003, folder_id: FOLDER-005, file_id: FILE-017, class_id: CLASS-017, method_id: METHOD-002, verify_id: VERIFY-002}, skeleton_impact: none}
  - {node_id: METHOD-003, parent: FILE-019, children: [METHOD-007], dependencies: [METHOD-001, METHOD-002, METHOD-017], priority: critical, status: e2e_passed, risk: high, complexity: complex, owner: builder, acceptance_criteria: "4W@50k/s + 8R via extend_from_reader bulk merges (32k-cell chunked yields, offset shifts, tombstone skips, pinned core); merge-window reader rate >=95% of thermally-adjacent post baseline; P99 write <200us", verification_evidence: "phase2 E2E 4x PASS: pinned merges 71-764ms validate PASS; merge-window 644-734M/s vs post 578-727M/s; P99 write 47-137us (seal-op P99 204-379us disclosed); F1 registry-lock fix + F5 thermal matching applied", traceability: {req_id: REQ-008, spec_id: SPEC-008, sot_id: SOT-003, folder_id: FOLDER-005, file_id: FILE-019, class_id: CLASS-019, method_id: METHOD-003, verify_id: VERIFY-003}, skeleton_impact: updated_criterion}
  - {node_id: METHOD-017, parent: FILE-048, children: [METHOD-003], dependencies: [METHOD-001], priority: critical, status: e2e_passed, risk: high, complexity: complex, owner: builder, acceptance_criteria: "extend_from_reader merges surviving slices with SIMD offset shifts, skips tombstoned blocks, preserves validate() + sortedness; 1.2M-cell merge in bounded ms, never cell-by-cell push", verification_evidence: "phase2 E2E: 260k-cell merge validate PASS, pinned core 7, 429-764ms; id-space remap + nested-scope rejection exercised", traceability: {req_id: REQ-008, spec_id: SPEC-008, sot_id: SOT-003, folder_id: FOLDER-004, file_id: FILE-048, class_id: CLASS-048, method_id: METHOD-017, verify_id: VERIFY-017}, skeleton_impact: new_port}
  - {node_id: METHOD-004, parent: FILE-024, children: [METHOD-013], dependencies: [METHOD-001], priority: critical, status: stub, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "delta residual matches parity-signed hand computation on N=3 fixture", traceability: {req_id: REQ-010, spec_id: SPEC-010, sot_id: SOT-004, folder_id: FOLDER-008, file_id: FILE-024, class_id: CLASS-024, method_id: METHOD-004, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-013, parent: FILE-025, children: [METHOD-005], dependencies: [METHOD-004], priority: critical, status: stub, risk: high, complexity: complex, owner: builder, acceptance_criteria: "sparse block SpMV L_F=deltaT delta PSD; hand-checked 3-cell fixture; morph-fetch cost isolated (binary-search watchpoint, stride-offset trigger); factors feed METHOD-005", traceability: {req_id: REQ-011, spec_id: SPEC-011, sot_id: SOT-004, folder_id: FOLDER-008, file_id: FILE-025, class_id: CLASS-025, method_id: METHOD-013, verify_id: null}, skeleton_impact: added-missing-coverage}
  - {node_id: METHOD-005, parent: FILE-028, children: [METHOD-006], dependencies: [METHOD-013], priority: critical, status: stub, risk: high, complexity: complex, owner: builder, acceptance_criteria: "10k noisy nodes -> E<=1e-4 in 8-18 iters, <8ms", traceability: {req_id: REQ-012, spec_id: SPEC-012, sot_id: SOT-004, folder_id: FOLDER-009, file_id: FILE-028, class_id: CLASS-028, method_id: METHOD-005, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-006, parent: FILE-047, children: [METHOD-008, METHOD-016], dependencies: [METHOD-001, METHOD-005], priority: high, status: stub, risk: low, complexity: moderate, owner: builder, acceptance_criteria: ".section has all fields; HARMONIC_BOUNDED when E<0.005", traceability: {req_id: REQ-020, spec_id: SPEC-020, sot_id: SOT-006, folder_id: FOLDER-016, file_id: FILE-047, class_id: CLASS-047, method_id: METHOD-006, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-007, parent: FILE-044, children: [], dependencies: [METHOD-002, METHOD-003], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "overlapping incidence aborts; snapshot reads repeatable; P99 write <200us under 8-reader load; full commit path P99 <2ms over 200 sampled commits", verification_evidence: "phase2 E2E txn_abort_path PASS (AbortedConflict then Committed); commit-seq face-clock first-committer-wins; commit_path_p99 143-435us PASS (4 runs)", traceability: {req_id: REQ-009, spec_id: SPEC-009, sot_id: SOT-006, folder_id: FOLDER-015, file_id: FILE-044, class_id: CLASS-044, method_id: METHOD-007, verify_id: VERIFY-007}, skeleton_impact: updated_criterion}
  - {node_id: METHOD-008, parent: FILE-031, children: [METHOD-014], dependencies: [], priority: high, status: stub, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "50 valid TQL <50us; 20 malformed pinpointed", traceability: {req_id: REQ-017, spec_id: SPEC-017, sot_id: SOT-005, folder_id: FOLDER-010, file_id: FILE-031, class_id: CLASS-031, method_id: METHOD-008, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-014, parent: FILE-033, children: [METHOD-009], dependencies: [METHOD-008], priority: high, status: stub, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "AST lowers to 12-opcode TIR shapes downstream expects; golden TIR fixtures match", traceability: {req_id: REQ-018, spec_id: SPEC-018, sot_id: SOT-005, folder_id: FOLDER-011, file_id: FILE-033, class_id: CLASS-033, method_id: METHOD-014, verify_id: null}, skeleton_impact: added-missing-coverage}
  - {node_id: METHOD-009, parent: FILE-034, children: [METHOD-010], dependencies: [METHOD-014], priority: high, status: stub, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "Eulerian validator rejects delta2!=0; dim mismatches rejected", traceability: {req_id: REQ-013, spec_id: SPEC-013, sot_id: SOT-005, folder_id: FOLDER-011, file_id: FILE-034, class_id: CLASS-034, method_id: METHOD-009, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-010, parent: FILE-038, children: [METHOD-006, METHOD-016], dependencies: [METHOD-009], priority: normal, status: stub, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "N=6 EXPLAIN shows single slice, not O(64) fanout", traceability: {req_id: REQ-019, spec_id: SPEC-019, sot_id: SOT-005, folder_id: FOLDER-012, file_id: FILE-038, class_id: CLASS-038, method_id: METHOD-010, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-011, parent: FILE-020, children: [METHOD-006], dependencies: [METHOD-001], priority: high, status: stub, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "name lookup returns contiguous slice; parity with scan", traceability: {req_id: REQ-015, spec_id: SPEC-015, sot_id: SOT-003, folder_id: FOLDER-006, file_id: FILE-020, class_id: CLASS-020, method_id: METHOD-011, verify_id: null}, skeleton_impact: none}
  - {node_id: METHOD-012, parent: FILE-016, children: [METHOD-003], dependencies: [], priority: high, status: e2e_passed, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "delete 0-cell tombstones Star; readers skip via live_mask", verification_evidence: "phase2 E2E: 1,005 deletes cascade to 2,198 drops; merged dim0/dim2 counts exact (198995/58809); base_star backfill closed the post-merge dangling bug", traceability: {req_id: REQ-014, spec_id: SPEC-014, sot_id: SOT-003, folder_id: FOLDER-005, file_id: FILE-016, class_id: CLASS-016, method_id: METHOD-012, verify_id: VERIFY-012}, skeleton_impact: none}
  - {node_id: METHOD-018, parent: FILE-018, children: [METHOD-007], dependencies: [METHOD-001], priority: high, status: e2e_passed, risk: low, complexity: moderate, owner: builder, acceptance_criteria: "snapshots pin Arcs + copied live bitsets; repeatable reads under contention; registry Drop releases generation pins for merged-map pruning", verification_evidence: "phase2 E2E: 12-thread snapshot churn across 2 merges, repeatable reads + post-merge counts exact; no leaked generation pins", traceability: {req_id: REQ-009, spec_id: SPEC-009, sot_id: SOT-003, folder_id: FOLDER-005, file_id: FILE-018, class_id: CLASS-018, method_id: METHOD-018, verify_id: VERIFY-018}, skeleton_impact: new}
  - {node_id: METHOD-015, parent: FILE-021, children: [METHOD-016], dependencies: [METHOD-001], priority: normal, status: stub, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "hypergraph split into 64MB L3 domains; ghost replicas serve cross-socket reads locally", traceability: {req_id: REQ-016, spec_id: SPEC-016, sot_id: SOT-003, folder_id: FOLDER-006, file_id: FILE-021, class_id: CLASS-021, method_id: METHOD-015, verify_id: null}, skeleton_impact: added-missing-coverage}
  - {node_id: METHOD-016, parent: FILE-042, children: [], dependencies: [METHOD-006, METHOD-010, METHOD-015], priority: high, status: stub, risk: medium, complexity: complex, owner: builder, acceptance_criteria: "tests/e2e/phase6_bench fraud net 10k checks: N>=4 >=5x QPS vs PG+Neo4j, P99 sub-ms", traceability: {req_id: REQ-021, spec_id: SPEC-021, sot_id: SOT-006, folder_id: FOLDER-014, file_id: FILE-042, class_id: CLASS-042, method_id: METHOD-016, verify_id: null}, skeleton_impact: added-missing-coverage}
  - {node_id: METHOD-019, parent: FILE-049, children: [METHOD-020, METHOD-022], dependencies: [METHOD-015], priority: high, status: stub, risk: medium, complexity: complex, owner: builder, acceptance_criteria: "KaHyPar cover fixture: >=95% hyperedges interior, zero split hyperedges", traceability: {req_id: REQ-025, spec_id: SPEC-025, sot_id: SOT-007, folder_id: FOLDER-017, file_id: FILE-049, class_id: CLASS-049, method_id: METHOD-019, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-020, parent: FILE-051, children: [METHOD-021], dependencies: [METHOD-019, METHOD-005], priority: high, status: stub, risk: high, complexity: complex, owner: builder, acceptance_criteria: "2-node RAS fixture converges sans barriers; local PCG <10us/subdomain", traceability: {req_id: REQ-026, spec_id: SPEC-026, sot_id: SOT-007, folder_id: FOLDER-017, file_id: FILE-051, class_id: CLASS-051, method_id: METHOD-020, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-021, parent: FILE-052, children: [], dependencies: [METHOD-020, METHOD-012], priority: high, status: stub, risk: medium, complexity: complex, owner: builder, acceptance_criteria: "concurrent border edits converge to min-energy section; zero 2PC/locks", traceability: {req_id: REQ-027, spec_id: SPEC-027, sot_id: SOT-007, folder_id: FOLDER-017, file_id: FILE-052, class_id: CLASS-052, method_id: METHOD-021, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-022, parent: FILE-053, children: [], dependencies: [METHOD-019], priority: high, status: stub, risk: medium, complexity: moderate, owner: builder, acceptance_criteria: "100-query nerve fixture: local queries zero-hop; spanning sliced on edges", traceability: {req_id: REQ-028, spec_id: SPEC-028, sot_id: SOT-007, folder_id: FOLDER-018, file_id: FILE-053, class_id: CLASS-053, method_id: METHOD-022, verify_id: null}, skeleton_impact: new}
  - {node_id: METHOD-023, parent: FILE-054, children: [], dependencies: [METHOD-022], priority: high, status: stub, risk: high, complexity: complex, owner: builder, acceptance_criteria: "one-sided border reads correct over loopback; 1.2us gate HW-qualified RoCEv2", traceability: {req_id: REQ-029, spec_id: SPEC-029, sot_id: SOT-007, folder_id: FOLDER-018, file_id: FILE-054, class_id: CLASS-054, method_id: METHOD-023, verify_id: null}, skeleton_impact: new}

pattern_library_candidates: []
