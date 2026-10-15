# Code change map — every change → every file (committed, pending, planned)

| # | Commit | Files | Lines | Purpose | State |
|---|---|---|---|---|---|
| 1 | `b79d89a` | `tests/e2e/realtime_bench/run_all.py` (NEW) | ~200 | One-command matrix: servers, sequential benches, runN merge, summary | Local only |
| 2 | `82bc053` | `crates/sheaf-engine/src/bin/fused_scan_bench.rs` (NEW) | ~90 | Overhaul A: fused revenue column scan | Local only |
| 3 | `fe53e6f` | `run_all.py` (edit) | +45/−24 | SHELF_BINS+=fused, docker ready_wait, fail output | Local only |
| 4 | `8a56243` | `crates/sheaf-engine/src/bin/fiber_snap_bench.rs` (NEW) | ~230 | Overhaul C: fiber snapshot persist + load | Local only |
| 5 | `6f4a466` | `crates/sheaf-storage/src/bcsf/name_lookup.rs` (edit) | +21 | Overhaul D: `NameIndex::range` interval API (BE bounds) | Local only |
| 5 | `6f4a466` | `crates/sheaf-engine/src/bin/range_slice_bench.rs` (NEW) | 81 | Overhaul D: 10k-slice bench vs Scylla | Local only |
| 5 | `6f4a466` | `run_all.py` (edit) | +3/−1 | SHELF_BINS+=snapshot+range bins | Local only |
| 6 | `298edd8` | `tests/e2e/realtime_bench/mongo_bench.py` (edit) | +20/−5 | Truthful unindexed/indexed pair (index-drop fix) | Local only |
| 6 | `298edd8` | `results/{8 engines}.json` + `RUN_SUMMARY_*.json` (edits) | +642/−12 | run4/run5 histories, repairs, retraction notes | Local only |
| 7 | (this session) | `sheaf-kernel/.../operators/laplacian.rs` (edit) | hoist + scale + diag | Mᵀδ hoist, per-op MᵀM, diag fast path, M⁻¹ cache | Uncommitted |
| 7 | (this session) | `sheaf-kernel/.../solvers/tikhonov.rs` (edit) | 1 line | Hot loop → cached M⁻¹ | Uncommitted |
| 8 | (this session) | `sheaf-kernel/.../solvers/chebyshev.rs` (NEW) | ~110 | P0 Chebyshev Tikhonov filter | Uncommitted |
| 8 | (this session) | `sheaf-kernel/src/components.rs` (NEW) | ~40 | b₀ union-find | Uncommitted |
| 8 | (this session) | `sheaf-kernel/.../operators/energy.rs` (edit) | +9 | H¹-proxy inconsistency() | Uncommitted |
| 8 | (this session) | `crates/sheaf-engine/src/bin/p0_gates_bench.rs` (NEW) | ~110 | P0 triple gate bin | Uncommitted |

## Uncommitted (working tree)

| File | Type | Action needed |
|---|---|---|
| `AGENTS.md` (NEW) | Response-format rule | Commit if wanted |
| `SHEAF_THEORY_REVIEW.md` (NEW) | Theory efficiency review + related-theory appendix | Commit if wanted |
| `VON_NEUMANN_MAPPING.md` (NEW) | Math→machine translation | Commit if wanted |
| `CODE_CHANGE_MAP.md` (NEW, this file) | Change inventory | Commit if wanted |
| `facebook_combined.txt.gz`, `ieee-fraud-detection.zip` | Data archives | Leave untracked (data, not code) |
| `D:\bench\data\fiber_snapshot.bin` | Build artifact (outside repo) | Keep on disk (speeds reruns) |

## Planned (not started)

| Item | Target file(s) | Source |
|---|---|---|
| Hoist Mᵀδ in apply loop | `sheaf-kernel/.../operators/laplacian.rs` | Review §constants |
| Scale-not-recompute MᵀM in build | same | Review §constants |
| Cache M⁻¹ across same-λ solves | same + `solvers/tikhonov.rs` | Review §constants |
| P0 Chebyshev Tikhonov filter | new bin + `solvers/` | Review part 2 |
| P0 H¹ curl residual metric | `solvers/hodge.rs` or `operators/energy.rs` | Review part 2 |
| P0 lumped diagonal Hodge-⋆ | `operators/` | Review part 2 |
| P1 Morse shrink, per-stalk QR, projection sparsifier | new modules | Review part 2 |
| Runner shakedown (docker E2E, --fresh/--engines/--keep-up) | `run_all.py` | Session notes |
| Push 6 local commits | remote `personal/main` | Blocked: window + no auth |

## Divergence

```
local main    : 6 commits ahead (b79d89a..298edd8)
remote        : untouched since revert 03aaec0 (0 pushed this session section)
reverted pair : 8684aa2 (bad rematch) / 03aaec0 (revert) — history only
```
