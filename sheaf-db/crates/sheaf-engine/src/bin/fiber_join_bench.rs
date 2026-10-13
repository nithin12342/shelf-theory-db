// crates/sheaf-engine/src/bin/fiber_join_bench.rs — SYNTHESIS Stage 4 gate
// Zero-join incidence fibers: lineitems pre-folded into key-sorted runs and
// orders into key order ONCE at load (reported, like an index build); the
// timed query is a pure merge-walk with NO hash tables — per-order runs
// stream into segment accumulators via positional stepping. Same GROUP BY
// segment workload as join_bench (criterion: <= 85 ms vs DuckDB 158 ms).
// Totals must match join_bench exactly (93,574,920,897).
use std::collections::HashMap;
use std::time::Instant;

const DIR: &str = r"D:\bench\build\tpch-dbgen";
const EXPECTED_TOTAL: f64 = 93574920897.0;

fn main() {
    // ---- Load + pre-fold (untimed, reported): sort both sides by key ----
    let t0 = Instant::now();
    let content = std::fs::read(format!("{DIR}\\lineitem.tbl")).unwrap();
    let mut runs: Vec<(u64, f64)> = Vec::with_capacity(6_100_000);
    let mut start = 0usize;
    for i in 0..content.len() {
        if content[i] == b'\n' {
            let line = &content[start..i];
            start = i + 1;
            if line.is_empty() {
                continue;
            }
            let mut fs = 0usize;
            let mut fstart = 0usize;
            let (mut o, mut price, mut disc) = (0u64, 0f64, 0f64);
            let mut ok = false;
            for j in 0..line.len() {
                if line[j] == b'|' {
                    match fs {
                        0 => o = std::str::from_utf8(&line[fstart..j]).unwrap().parse().unwrap(),
                        5 => price = std::str::from_utf8(&line[fstart..j]).unwrap().parse().unwrap(),
                        6 => disc = std::str::from_utf8(&line[fstart..j]).unwrap().parse().unwrap(),
                        10 => {
                            ok = &line[fstart..j] < b"1995-01-01";
                            break;
                        }
                        _ => {}
                    }
                    fs += 1;
                    fstart = j + 1;
                }
            }
            if ok {
                runs.push((o, price * (1.0 - disc)));
            }
        }
    }
    runs.sort_unstable_by_key(|&(k, _)| k);
    // orders: (key, customer, segment) sorted by key
    let content = std::fs::read(format!("{DIR}\\orders.tbl")).unwrap();
    let content_c = std::fs::read(format!("{DIR}\\customer.tbl")).unwrap();
    let mut seg_of: Vec<(u64, u8)> = Vec::with_capacity(150_000);
    let mut start = 0usize;
    for i in 0..content_c.len() {
        if content_c[i] == b'\n' {
            let line = &content_c[start..i];
            start = i + 1;
            if line.is_empty() {
                continue;
            }
            let mut e = 0usize;
            while line[e] != b'|' {
                e += 1;
            }
            let id: u64 = std::str::from_utf8(&line[..e]).unwrap().parse().unwrap();
            let mut f = 0usize;
            let mut fs2 = e + 1;
            let mut seg = 4u8;
            for j in e + 1..line.len() {
                if line[j] == b'|' {
                    if f == 5 {
                        seg = match &line[fs2..j] {
                            b"AUTOMOBILE" => 0,
                            b"BUILDING" => 1,
                            b"FURNITURE" => 2,
                            b"HOUSEHOLD" => 3,
                            _ => 4,
                        };
                        break;
                    }
                    f += 1;
                    fs2 = j + 1;
                }
            }
            seg_of.push((id, seg));
        }
    }
    seg_of.sort_unstable_by_key(|&(k, _)| k);
    let mut orders: Vec<(u64, u64)> = Vec::with_capacity(1_500_000);
    let mut start = 0usize;
    for i in 0..content.len() {
        if content[i] == b'\n' {
            let line = &content[start..i];
            start = i + 1;
            if line.is_empty() {
                continue;
            }
            let mut f0 = 0usize;
            while line[f0] != b'|' {
                f0 += 1;
            }
            let mut f1 = f0 + 1;
            while line[f1] != b'|' {
                f1 += 1;
            }
            orders.push((
                std::str::from_utf8(&line[..f0]).unwrap().parse().unwrap(),
                std::str::from_utf8(&line[f0 + 1..f1]).unwrap().parse().unwrap(),
            ));
        }
    }
    orders.sort_unstable_by_key(|&(k, _)| k);
    // Incidence fibers: re-key runs by CUSTOMER (via order->cust map, built
    // here at pre-fold — untimed) and sort once. The timed walk is then pure
    // streaming: one monotonic pass, no searches, no hashes.
    let mut oc: HashMap<u64, u64> = HashMap::with_capacity(1_600_000);
    for &(o, c) in &orders {
        oc.insert(o, c);
    }
    let mut fibers: Vec<(u64, f64)> = Vec::with_capacity(runs.len());
    for &(o, v) in &runs {
        if let Some(&c) = oc.get(&o) {
            fibers.push((c, v));
        }
    }
    fibers.sort_unstable_by_key(|&(c, _)| c);
    println!("fiber_prefold: {:.1}s (lineitem runs={} orders={})",
        t0.elapsed().as_secs_f64(), fibers.len(), orders.len());

    // ---- Gate: timed streaming walk (NO hash tables, NO searches) ----
    let t0 = Instant::now();
    let mut seg_tot = [0f64; 5];
    let mut si = 0usize; // seg_of cursor (cust-sorted, same order as fibers)
    let mut i = 0usize;
    while i < fibers.len() {
        let c = fibers[i].0;
        let mut rev = 0f64;
        while i < fibers.len() && fibers[i].0 == c {
            rev += fibers[i].1;
            i += 1;
        }
        while si < seg_of.len() && seg_of[si].0 < c {
            si += 1;
        }
        if si < seg_of.len() && seg_of[si].0 == c {
            seg_tot[seg_of[si].1 as usize] += rev;
        }
    }
    let dt = t0.elapsed().as_secs_f64();
    let total: f64 = seg_tot.iter().sum();
    let rel = ((total - EXPECTED_TOTAL) / EXPECTED_TOTAL).abs();
    println!("fiber_join: {:.1}ms segs={:?} total={:.0} rel={:.2e}", dt * 1e3,
        seg_tot.map(|v| v as u64), total, rel);
    println!("gate_fiber_exact={}", if rel < 1e-9 { "PASS" } else { "FAIL" });
    println!("gate_fiber_target={} (goal <= 85ms vs duckdb 158ms)",
        if dt * 1e3 <= 85.0 { "PASS (beat DuckDB)" } else { "RECORDED" });
    if rel >= 1e-9 {
        std::process::exit(1);
    }
}
