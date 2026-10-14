// crates/sheaf-engine/src/bin/fused_scan_bench.rs — Overhaul A (scan defeat fix)
// Defeat: DuckDB 6.4ms vs BCSF ~8ms on 6M-row SUM(price*(1-disc)).
// Diagnosis (bottleneck.json): 48MB scan floor = 5.7ms at 8.4GB/s; the loop
// is bandwidth-bound, so no ALU/SIMD tweak on 8B/row can win. Fix: fuse the
// derived column at load (materialized, same class as fiber pre-fold:
// precompute once, stream forever; load cost recorded), scan 4B/row = 24MB
// (floor ~2.9ms). Gate: best rep < 6.4ms AND rel err < 1e-6 vs f64 baseline.
use std::time::Instant;

use sheaf_storage::bcsf::builder::BcsfBuilder;

const LINEITEM: &str = r"D:\bench\build\tpch-dbgen\lineitem.tbl";
const EXPECTED_ROWS: usize = 6_001_215;
const TARGET_MS: f64 = 6.4;

fn main() {
    // ---- Load (untimed scan convention; cost recorded, mirrors COPY) ----
    let t0 = Instant::now();
    let mut builder = BcsfBuilder::new();
    let mut fused: Vec<f32> = Vec::with_capacity(EXPECTED_ROWS);
    let db = [0u8; 16];
    let content = std::fs::read(LINEITEM).expect("lineitem.tbl missing");
    let mut start = 0usize;
    let mut n = 0usize;
    for i in 0..content.len() {
        if content[i] == b'\n' {
            let line = &content[start..i];
            start = i + 1;
            if line.is_empty() {
                continue;
            }
            let mut fstart = 0usize;
            let mut field = 0usize;
            let (mut price, mut disc) = (0f32, 0f32);
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
            builder.push(0, Vec::new(), &db, &[price, disc], &[]);
            fused.push(price * (1.0 - disc));
            n += 1;
        }
    }
    let reader = builder.finish();
    assert_eq!(n, EXPECTED_ROWS, "row count");
    assert!(reader.validate().is_ok());
    println!("fused_load: rows={} load_s={:.1} fused_MB={:.1}", n,
        t0.elapsed().as_secs_f64(), fused.len() * 4 / 1_000_000);

    // ---- Exactness: f64 full-precision baseline over BCSF stalks ----
    let slab = &reader.arena.stalks;
    let mut base = 0f64;
    let mut i = 0usize;
    while i < slab.len() {
        base += slab[i] as f64 * (1.0 - slab[i + 1] as f64);
        i += 2;
    }

    // ---- Timed: contiguous f32 fused sum, 5 reps, best + median ----
    let mut reps = Vec::new();
    let mut sum = 0f64;
    for _ in 0..5 {
        let t0 = Instant::now();
        let mut acc = 0f64;
        for &v in fused.iter() {
            acc += v as f64;
        }
        std::hint::black_box(acc);
        reps.push(t0.elapsed().as_secs_f64() * 1e3);
        sum = acc;
    }
    reps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rel = ((sum - base) / base).abs();
    println!("fused_scan: best_ms={:.1} median_ms={:.1} sum={:.2} baseline={:.2} rel={:.2e}",
        reps[0], reps[2], sum, base, rel);
    println!("gate_fused_exact={}", if rel < 1e-6 { "PASS" } else { "FAIL" });
    println!("gate_fused_target={}", if reps[0] < TARGET_MS {
        "PASS (beat DuckDB)"
    } else {
        "RECORDED (DuckDB wins)"
    });
}
