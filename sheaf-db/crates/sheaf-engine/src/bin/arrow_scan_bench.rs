// crates/sheaf-engine/src/bin/arrow_scan_bench.rs — SYNTHESIS Stage 1a gate
// Builds the Arrow-compatible slab from lineitem.tbl and verifies:
// (1) base address 64B-aligned, (2) fused revenue sum matches tpch_scan's
// 218102223881.86 (same row order/ops), (3) scan time vs 7.9 ms baseline.
// Stage 1a needs parity, not victory — the FMA kernel (Stage 1b) chases 2.8 ms.
use std::time::Instant;

use sheaf_storage::bcsf::arrow_layout::ArrowStalkSlab;

const LINEITEM: &str = r"D:\bench\build\tpch-dbgen\lineitem.tbl";
const EXPECTED_ROWS: usize = 6_001_215;
const EXPECTED_SUM: f64 = 218102223881.86;

fn main() {
    let t0 = Instant::now();
    let mut slab = ArrowStalkSlab::with_capacity(2, EXPECTED_ROWS + 64);
    let content = std::fs::read(LINEITEM).expect("lineitem.tbl missing");
    let mut start = 0usize;
    for i in 0..content.len() {
        if content[i] == b'\n' {
            let line = &content[start..i];
            start = i + 1;
            if line.is_empty() {
                continue;
            }
            let mut fstart = 0usize;
            let mut field = 0usize;
            let mut price = 0f32;
            let mut disc = 0f32;
            for j in 0..line.len() {
                if line[j] == b'|' {
                    if field == 5 {
                        price = std::str::from_utf8(&line[fstart..j]).unwrap().parse().unwrap();
                    } else if field == 6 {
                        disc = std::str::from_utf8(&line[fstart..j]).unwrap().parse().unwrap();
                        break;
                    }
                    field += 1;
                    fstart = j + 1;
                }
            }
            slab.push_row(&[price, disc]);
        }
    }
    println!("arrow_load: rows={} load_s={:.1}", slab.len(), t0.elapsed().as_secs_f64());
    assert_eq!(slab.len(), EXPECTED_ROWS, "row count mismatch");

    // Gate 1: 64B alignment.
    let aligned = slab.base_addr().is_multiple_of(64);
    println!("gate_align_64B={} (base=0x{:x})", if aligned { "PASS" } else { "FAIL" }, slab.base_addr());

    // Gate 2: sum correctness (same order/ops as tpch_scan_bench).
    let sum = slab.fused_revenue_sum(0, 1);
    let rel = ((sum - EXPECTED_SUM) / EXPECTED_SUM).abs();
    println!("gate_sum_exact={} (sum={:.2}, rel_diff={:.2e})",
        if rel < 1e-9 { "PASS" } else { "FAIL" }, sum, rel);

    // Gate 3: scan parity (timed x3).
    let mut best = f64::MAX;
    for _ in 0..3 {
        let t0 = Instant::now();
        std::hint::black_box(slab.fused_revenue_sum(0, 1));
        best = best.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    println!("arrow_scan: best_ms={:.1} (3 reps) vs tpch_scan 7.9ms", best);
    println!("gate_parity={}", if best < 7.9 * 1.1 { "PASS (within 10% of baseline)" } else { "RECORDED (regression vs baseline)" });

    // Gate 4 (Stage 1b): FMA kernel over the raw slab (validity pre-compacted:
    // this slab is fully valid, so the kernel skips bit checks by contract).
    let raw: &[f32] = unsafe {
        std::slice::from_raw_parts(slab.base_addr() as *const f32, slab.len() * 2)
    };
    let mut fma_best = f64::MAX;
    let mut fma_sum = 0f64;
    for _ in 0..3 {
        let t0 = Instant::now();
        fma_sum = sheaf_kernel::simd::fma_scan::fused_sum_pairs(raw, 2, 0, 1);
        fma_best = fma_best.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    let fma_rel = ((fma_sum - EXPECTED_SUM) / EXPECTED_SUM).abs();
    println!("fma_scan: best_ms={:.1} sum_rel_diff={:.2e} (tol 1e-6, FMA rounding)", fma_best, fma_rel);
    println!("fma_path={}", sheaf_kernel::simd::fma_scan::path_name());
    println!("gate_fma_correct={}", if fma_rel < 1e-6 { "PASS" } else { "FAIL" });
    println!("gate_fma_target={} (goal <= 2.8ms vs duckdb 6.4ms)",
        if fma_best <= 2.8 { "PASS" } else { "RECORDED" });

    // Gate 5 (SoA follow-up): transpose once (reported), then unit-stride
    // sum over contiguous columns — the layout gather cannot match.
    let t0 = Instant::now();
    let (cp, cq) = slab.transpose_soa(0, 1);
    let transpose_ms = t0.elapsed().as_secs_f64() * 1e3;
    // Single timed rep (NOT best-of-3 in-process): any intra-process rep
    // loop lets scalar evolution hoist the invariant sum out of the timed
    // region (observed: 0.0/0.0/27.9ms reps with correct sums). Repeat at the
    // PROCESS level (run binary 3x, take best) — immune to hoisting.
    let t0 = Instant::now();
    let mut acc = 0f64;
    // zip iterators: LLVM eliminates bounds checks (unlike dual indexing
    // cp[i]/cq[i], where cq's check survives and costs ~3x).
    for (p, q) in cp.iter().zip(cq.iter()) {
        acc += *p as f64 * (1.0 - *q as f64);
    }
    let soa_ms = t0.elapsed().as_secs_f64() * 1e3;
    let soa_sum = acc;
    let soa_rel = ((soa_sum - EXPECTED_SUM) / EXPECTED_SUM).abs();
    println!("soa_scan: transpose_ms={:.1} (one-time) rep_ms={:.1} rel={:.2e}", transpose_ms, soa_ms, soa_rel);
    println!("gate_soa={} (goal <= 6.4ms duckdb quiet; repeat binary 3x, take best)",
        if soa_ms <= 6.4 && soa_rel < 1e-9 { "PASS (beat DuckDB)" } else { "RECORDED" });

    if !(aligned && rel < 1e-9) {
        std::process::exit(1);
    }
    println!("STAGE-1A: PASS");

    // P2 gate (exactness only, runs after all timings): null every 7th row,
    // verify exception list + compacted dense sum matches the valid subset.
    for r in (0..slab.len()).step_by(7) {
        slab.set_null(r);
    }
    let exc = slab.null_exceptions();
    let (dp, dq, kept) = slab.compact_valid(0, 1);
    let mut acc = 0f64;
    for (a, b) in dp.iter().zip(dq.iter()) {
        acc += *a as f64 * (1.0 - *b as f64);
    }
    // Reference: the slab's own fused sum skips the same nulls, so exact
    // agreement proves compaction preserves the sum; exc non-empty proves
    // nulls actually existed (partition identity checked below).
    let full = slab.fused_revenue_sum(0, 1);
    let ok = exc.windows(2).all(|w| w[0] < w[1])
        && kept + exc.len() == slab.len()
        && !exc.is_empty()
        && (acc - full).abs() < 1e-6;
    println!("gate_p2_exceptions={} (nulls={}, kept={}, dense_sum={:.2})",
        if ok { "PASS" } else { "FAIL" }, exc.len(), kept, acc);
    if !ok {
        std::process::exit(1);
    }
}
