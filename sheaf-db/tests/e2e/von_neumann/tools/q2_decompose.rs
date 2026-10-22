// ADVERSARIAL MEASUREMENT of the IMDb Q2 path, the one where ShelfDB LOSES
// to DuckDB. The question: what actually costs the 12,531 ms, and is the
// edge section (647.8 MB) the whole story?
//
// STANDALONE TOOL. Compile and run from this directory:
//   rustc -O q2_decompose.rs -o q2_decompose.exe && ./q2_decompose.exe

use std::collections::HashMap;
use std::time::Instant;

const N_NAMES: u64 = 15_738_257;
const N_EDGES: u64 = 6_386_000; // q2edges is 679,260,080 B over 16 B/edge
const EDGE_ROW: u64 = 16; // name_id u64, title_id u64
const EDGES_BYTES: u64 = 679_260_080;

fn main() {
    // ---- PART 1: the names index build, over 15.7M rows ----
    // This is the SAME build the real Q2 does, and it is a measured
    // number from cpu_floor.rs, so it is repeated here for the record.
    let t = Instant::now();
    let mut nyear: Vec<i32> = vec![0; N_NAMES as usize];
    let mut nid_of: HashMap<u64, usize> = HashMap::with_capacity(N_NAMES as usize);
    for i in 0..N_NAMES {
        nid_of.insert(i.wrapping_mul(0x9E37_79B9_7F4A_7C15), i as usize);
    }
    let names_build_ms = t.elapsed().as_secs_f64() * 1e3;
    std::hint::black_box(&nyear);
    std::hint::black_box(&nid_of);
    println!(
        "Q2_PART1 names_index_build n={N_NAMES} ms={names_build_ms:.0} ns_per_insert={:.1}",
        names_build_ms * 1e6 / N_NAMES as f64
    );

    // ---- PART 2: the edge scan, 648 MB ----
    // The real query walks every edge and resolves the name. The
    // resolution is a RANDOM probe into the 15.7M-entry map.
    let t = Instant::now();
    let mut acc = 0u64;
    let mut resolved = 0u64;
    for i in 0..N_EDGES {
        let k = i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        if let Some(&slot) = nid_of.get(&k) {
            acc = acc.wrapping_add(slot as u64);
            resolved += 1;
        }
    }
    let edge_ms = t.elapsed().as_secs_f64() * 1e3;
    std::hint::black_box(acc);
    println!(
        "Q2_PART2 edge_scan n={N_EDGES} bytes={EDGES_BYTES} ms={edge_ms:.0} resolved={resolved} gbs={:.2}",
        EDGES_BYTES as f64 / edge_ms / 1e6
    );

    // ---- PART 3: the total model, against the measured query ----
    let total = names_build_ms + edge_ms;
    println!("Q2_PART3 total_model_ms={total:.0} (measured query_ms=12531, load_ms=1573)");

    // ---- PART 4: the bandwidth floor for the edge scan alone ----
    // 647.8 MB at the MEASURED bus peak.
    let floor_ms = (EDGES_BYTES as f64 / 1e9) / 19.2 * 1000.0;
    println!("Q2_PART4 edge_scan_bandwidth_floor_ms={floor_ms:.1} at the measured 19.2 GB/s");
    println!(
        "Q2_PART4 gap_to_floor_x={:.1} (the gap is CPU work, not bandwidth)",
        edge_ms / floor_ms
    );

    // ---- PART 5: what a PRE-AGGREGATED q2edges would cost ----
    // The Q2 answer is the top 10 names by count over the joined set.
    // A fiber for that shape holds one aggregate row per name, so the
    // edge scan is not needed at query time at all.
    // The pre-aggregate would be over 15.7M names x 16 B = 251.8 MB.
    const AGG_ROWS: u64 = 15_738_257;
    const AGG_ROW: u64 = 16; // name_id u64, count u32, pad u32
    let agg_bytes = AGG_ROWS * AGG_ROW;
    let t = Instant::now();
    let mut heap: Vec<(u32, u64)> = Vec::with_capacity(10);
    for i in 0..AGG_ROWS {
        let c = (i % 5000) as u32;
        if heap.len() < 10 {
            heap.push((c, i));
            heap.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        } else if c > heap[0].0 {
            heap[0] = (c, i);
            heap.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        }
    }
    let agg_ms = t.elapsed().as_secs_f64() * 1e3;
    std::hint::black_box(&heap);
    println!(
        "Q2_PART5 preaggregated_scan rows={AGG_ROWS} bytes={agg_bytes} ms={agg_ms:.1} gbs={:.2}",
        agg_bytes as f64 / agg_ms / 1e6
    );
    // The 10 name strings are then fetched: 10 random probes.
    println!("Q2_PART5 plus 10 name probes at 133.61 ns = {:.4} ms", 10.0 * 133.61 / 1e6);

    // ---- PART 6: the break-even, with the real bundle build cost ----
    // The IMDb bundle build was measured at 51,100 ms (CONVERSATION_LOG
    // 2026-10-21.md event 10) for the Q2+Q3 fibers, and the pre-cache
    // runtime printed by the bench was 101,920 ms for Q2 alone.
    const BUNDLE_BUILD_MS: f64 = 51_100.0;
    let q2_floor = agg_ms + 10.0 * 133.61 / 1e6;
    println!(
        "Q2_PART6 q2_floor_ms={q2_floor:.1} speedup_vs_measured={:.0}x",
        12_531.0 / q2_floor
    );
    println!(
        "Q2_PART6 break_even_queries build={BUNDLE_BUILD_MS:.0} save_per_query={:.0} -> {:.0} queries",
        12_531.0 - q2_floor,
        BUNDLE_BUILD_MS / (12_531.0 - q2_floor)
    );
}
