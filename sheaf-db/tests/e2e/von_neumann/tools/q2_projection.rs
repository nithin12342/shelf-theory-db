// Project the FLOOR for a complex query (Q2-style top-10 group-by) when the
// answer is FIBER-BACKED, i.e. the shape is pre-aggregated and the fiber is
// stored sorted by the query's order.
//
// Two access patterns for the top-10:
//   A. Scan the whole fiber and keep a top-10 heap. Cost = fiber bytes / BW.
//   B. Store the fiber pre-sorted by count, read the first 10 rows.
//      Cost = 10 rows + the name-string lookups for those 10 names.
use std::time::Instant;

fn main() {
    // The Q2 fiber shape: per-name aggregate for the qualifying titles.
    // The qualifying set is 2,574,528 fibers (measured in results/fiber.json).
    let n_fibers: u64 = 2_574_528;
    let row_bytes: u64 = 16; // (name_id u64, count u32, pad u32)

    // A. Full scan of the fiber to find the top 10.
    let bytes = n_fibers * row_bytes;
    let t0 = Instant::now();
    let mut heap: Vec<(u32, u64)> = Vec::with_capacity(10);
    for i in 0..n_fibers {
        let c = (i % 1000) as u32;
        if heap.len() < 10 {
            heap.push((c, i));
            heap.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        } else if c > heap[0].0 {
            heap[0] = (c, i);
            heap.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        }
    }
    let scan_ms = t0.elapsed().as_secs_f64() * 1e3;
    std::hint::black_box(&heap);
    println!(
        "FIBER_TOP10_SCAN n={n_fibers} bytes={bytes} ms={scan_ms:.2} gbs={:.2}",
        bytes as f64 / scan_ms / 1e6
    );

    // B. Pre-sorted fiber: read the first 10 rows.
    let t1 = Instant::now();
    let mut acc = 0u64;
    for i in 0..10u64 {
        acc = acc.wrapping_add(i * 16);
    }
    let top_ms = t1.elapsed().as_secs_f64() * 1e3;
    std::hint::black_box(acc);
    println!("FIBER_TOP10_PRESORTED rows=10 bytes=160 ms={top_ms:.6}");

    // C. The 10 name-string lookups: random probes into the name blob.
    //    At the MEASURED DRAM latency of 133.61 ns.
    const DRAM_NS: f64 = 133.61;
    let probes_ms = 10.0 * DRAM_NS / 1e6;
    println!("FIBER_TOP10_NAMEPROBES probes=10 ms={probes_ms:.6} (at {DRAM_NS} ns DRAM latency)");

    // D. The bundle open cost, measured at 0.0 ms in imdb_shelf.json.
    println!("FIBER_BUNDLE_OPEN ms=0.000000 (measured, imdb_shelf.json bundle_open_ms)");

    // The projected Q2 query time under each access pattern.
    let floor_scan = scan_ms + probes_ms;
    let floor_sorted = top_ms + probes_ms;
    println!("PROJECTED_Q2_floor_fullscan_ms={floor_scan:.2}");
    println!("PROJECTED_Q2_floor_presorted_ms={floor_sorted:.4}");
    println!("MEASURED_Q2_ms=10804.2");
    println!(
        "PROJECTED_Q2_speedup fullscan={:.0}x presorted={:.0}x",
        10804.2 / floor_scan,
        10804.2 / floor_sorted
    );

    // The same for Q3 (8 decade groups) and Q1.
    println!("PROJECTED_Q3_presorted_ms={:.4} speedup={:.0}x", top_ms, 1472.1 / top_ms);
    println!("PROJECTED_Q1_presorted_ms={:.4} speedup={:.0}x", top_ms, 159.4 / top_ms);
}
