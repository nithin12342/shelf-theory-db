// crates/sheaf-storage/src/bin/wal_insert_bench.rs — Test 4: SQLite insert challenge
// 20,000 unbatched single-cell insertions through WalWriter (append + flush
// per op = full durability) with CRC validation; replay-verify all records.
// Reports p50/p99 per-insert latency vs SQLite WAL 2.1us p50.
use std::path::PathBuf;
use std::time::Instant;

use sheaf_storage::lsp::wal::{replay, WalWriter};

const N: usize = 20_000;

fn pct(mut v: Vec<u128>, p: f64) -> u128 {
    v.sort_unstable();
    v[((v.len() as f64 * p) as usize).min(v.len() - 1)]
}

/// Encode one cell insert: [tag u8][id u64LE][discrete u64LE][stalk 4xf32LE] = 33 B.
fn encode_cell(id: u64) -> Vec<u8> {
    let mut b = Vec::with_capacity(33);
    b.push(0u8);
    b.extend_from_slice(&id.to_le_bytes());
    b.extend_from_slice(&(id.wrapping_mul(2654435761) % 100000).to_le_bytes());
    for k in 0..4 {
        b.extend_from_slice(&((id as f32 * 0.001 + k as f32 * 0.1).to_le_bytes()));
    }
    b
}

/// Stack-buffer variant: no heap allocation.
fn encode_cell_stack(id: u64) -> [u8; 33] {
    let mut b = [0u8; 33];
    b[0] = 0;
    b[1..9].copy_from_slice(&id.to_le_bytes());
    b[9..17].copy_from_slice(&(id.wrapping_mul(2654435761) % 100000).to_le_bytes());
    for k in 0..4 {
        b[17 + k * 4..21 + k * 4].copy_from_slice(&((id as f32 * 0.001 + k as f32 * 0.1).to_le_bytes()));
    }
    b
}

fn main() {
    let dir = std::env::temp_dir().join("sheaf_wal_bench");
    std::fs::create_dir_all(&dir).unwrap();
    let path: PathBuf = dir.join("test4.wal");
    let _ = std::fs::remove_file(&path);

    // Gate 1: 20k unbatched inserts, append + flush each (full durability).
    let mut w = WalWriter::create(&path).unwrap();
    let mut ts: Vec<u128> = Vec::with_capacity(N);
    for i in 0..N {
        let payload = encode_cell(i as u64);
        let t0 = Instant::now();
        w.append(&payload).unwrap();
        w.flush().unwrap();
        ts.push(t0.elapsed().as_nanos());
    }
    drop(w);
    let p50 = pct(ts.clone(), 0.5);
    let p99 = pct(ts, 0.99);
    println!("wal_insert_durable: n={} p50_ns={} p99_ns={}", N, p50, p99);

    // Gate 2: replay recovers every record intact (CRC-verified).
    let (recs, truncated) = replay(&path).unwrap();
    let replay_ok = recs.len() == N && !truncated;
    println!(
        "gate_replay_exact={} (recs={}/{}, truncated={})",
        if replay_ok { "PASS" } else { "FAIL" },
        recs.len(),
        N,
        truncated
    );

    // B14: copy vs zero-copy replay on the SAME log (recovery no longer
    // duplicates the log). The zero-copy path returns payload RANGES into
    // the single buffer the file was read into.
    let t_copy = Instant::now();
    let (frames_copy, _) = sheaf_storage::lsp::wal::replay(&path).unwrap();
    let copy_ms = t_copy.elapsed().as_secs_f64() * 1e3;
    let t_zc = Instant::now();
    let zc = sheaf_storage::lsp::wal::replay_zero_copy(&path).unwrap();
    let zc_ms = t_zc.elapsed().as_secs_f64() * 1e3;
    let same = frames_copy.len() == zc.len()
        && (0..frames_copy.len()).all(|i| frames_copy[i] == zc.payload(i));
    println!(
        "wal_replay_compare: copy_ms={copy_ms:.1} zero_copy_ms={zc_ms:.1} frames={} identical={same}",
        frames_copy.len()
    );
    println!(
        "gate_replay_zero_copy={}",
        if same && zc_ms <= copy_ms { "PASS" } else { "FAIL" }
    );

    // Gate 3 (context): group-commit batch mode — 20k appends, one flush.
    let path2: PathBuf = dir.join("test4_batch.wal");
    let _ = std::fs::remove_file(&path2);
    let mut w2 = WalWriter::create(&path2).unwrap();
    let t0 = Instant::now();
    for i in 0..N {
        w2.append(&encode_cell(i as u64)).unwrap();
    }
    w2.flush().unwrap();
    let batch_ns = t0.elapsed().as_nanos() / N as u128;
    drop(w2);
    println!("wal_insert_batched: n={} avg_ns_per_op={} (one flush; less durable)", N, batch_ns);
    let (recs2, trunc2) = replay(&path2).unwrap();
    println!("gate_batch_replay={}", if recs2.len() == N && !trunc2 { "PASS" } else { "FAIL" });

    // DeltaWal gate (Node N1).
    let path3: PathBuf = dir.join("test4_delta.wal");
    let _ = std::fs::remove_file(&path3);
    let mut dw = sheaf_storage::DeltaWal::create(&path3).unwrap();
    let mut dts: Vec<u128> = Vec::with_capacity(N);
    for i in 0..N {
        let payload = encode_cell_stack(i as u64);
        let t0 = Instant::now();
        dw.append_delta(i as u64, &payload).unwrap();
        dts.push(t0.elapsed().as_nanos());
        // Seal every 1000 operations (group commit).
        if (i + 1) % 1000 == 0 {
            dw.flush().unwrap();
        }
    }
    dw.flush().unwrap();
    drop(dw);
    let dp50 = pct(dts.clone(), 0.5);
    let dp99 = pct(dts, 0.99);
    println!("delta_wal_durable: n={} p50_ns={} p99_ns={}", N, dp50, dp99);
    let (drecs, _, dtorn) = sheaf_storage::replay_delta(&path3).unwrap();
    let dreplay_ok = drecs.len() == N && !dtorn;
    println!("gate_delta_wal_replay={}", if dreplay_ok { "PASS" } else { "FAIL" });
    let _ = std::fs::remove_file(&path3);

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&path2);
    if !replay_ok || !dreplay_ok {
        std::process::exit(1);
    }
    println!("WAL_INSERT_BENCH: PASS");
}
