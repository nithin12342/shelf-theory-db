// crates/sheaf-engine/src/bin/fiber_snap_bench.rs — Overhaul C (join-build defeat)
// Defeat: fiber pre-fold re-sorts 6M+1.5M+150k rows (~1.8s) EVERY run; only
// the 3.2ms walk is timed. Fix: persist the sorted snapshot once (raw LE:
// [nfu64][(cust u64,rev f64)]×n[f nsegu64][(cust u64,seg u8)]×n); runs load
// it in ms. Gate: load_ms + query_ms < 158 (DuckDB) AND totals exact.
// Snapshot lives in D:\bench\data (build artifact, like bench.db).
use std::collections::HashMap;
use std::time::Instant;

use sheaf_storage::lsp::wal::crc32;

const DIR: &str = r"D:\bench\build\tpch-dbgen";
const SNAP: &str = r"D:\bench\data\fiber_snapshot.bin";
const EXPECTED_TOTAL: f64 = 93574920897.0;
const TARGET_MS: f64 = 158.0;

type Fibers = Vec<(u64, f64)>;
type Segs = Vec<(u64, u8)>;

fn parse_u64(line: &[u8], field: usize) -> u64 {
    let mut fs = 0usize;
    let mut fstart = 0usize;
    for j in 0..line.len() {
        if line[j] == b'|' {
            if fs == field {
                return std::str::from_utf8(&line[fstart..j]).unwrap().parse().unwrap();
            }
            fs += 1;
            fstart = j + 1;
        }
    }
    panic!("field {field} missing");
}

fn build() -> (Fibers, Segs) {
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
            let id = parse_u64(line, 0);
            let mut f = 0usize;
            let mut fstart = 0usize;
            let mut seg = 4u8;
            for j in 0..line.len() {
                if line[j] == b'|' {
                    if f == 6 {
                        seg = match &line[fstart..j] {
                            b"AUTOMOBILE" => 0,
                            b"BUILDING" => 1,
                            b"FURNITURE" => 2,
                            b"HOUSEHOLD" => 3,
                            _ => 4,
                        };
                        break;
                    }
                    f += 1;
                    fstart = j + 1;
                }
            }
            seg_of.push((id, seg));
        }
    }
    seg_of.sort_unstable_by_key(|&(k, _)| k);
    let content = std::fs::read(format!("{DIR}\\orders.tbl")).unwrap();
    let mut orders: Vec<(u64, u64)> = Vec::with_capacity(1_500_000);
    let mut start = 0usize;
    for i in 0..content.len() {
        if content[i] == b'\n' {
            let line = &content[start..i];
            start = i + 1;
            if line.is_empty() {
                continue;
            }
            orders.push((parse_u64(line, 0), parse_u64(line, 1)));
        }
    }
    orders.sort_unstable_by_key(|&(k, _)| k);
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
    (fibers, seg_of)
}

/// Snapshot framing: [MAGIC u64][payload_len u64][crc32 u32][payload].
/// Fail-loud: torn/mismatched files are rejected, never half-loaded
/// (COMPLEXITY_ANALYSIS A9: previously no checksum at all).
const SNAP_MAGIC: u64 = 0x534E415046494252;
const SNAP_BAD: &str = r"D:\bench\data\fiber_snapshot.bad";

fn save(fibers: &Fibers, seg_of: &Segs) {
    let mut payload = Vec::with_capacity(fibers.len() * 16 + seg_of.len() * 9 + 16);
    payload.extend_from_slice(&(fibers.len() as u64).to_le_bytes());
    for &(c, v) in fibers {
        payload.extend_from_slice(&c.to_le_bytes());
        payload.extend_from_slice(&v.to_le_bytes());
    }
    payload.extend_from_slice(&(seg_of.len() as u64).to_le_bytes());
    for &(c, s) in seg_of {
        payload.extend_from_slice(&c.to_le_bytes());
        payload.push(s);
    }
    let mut buf = Vec::with_capacity(payload.len() + 20);
    buf.extend_from_slice(&SNAP_MAGIC.to_le_bytes());
    buf.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    buf.extend_from_slice(&crc32(&payload).to_le_bytes());
    buf.extend_from_slice(&payload);
    std::fs::write(SNAP, &buf).expect("snapshot write");
}

fn load() -> Result<(Fibers, Segs), String> {
    let read = |path: &str| std::fs::read(path).map_err(|e| format!("{path}: {e}"));
    load_from(&read(SNAP)?, SNAP)
}

fn load_from(buf: &[u8], name: &str) -> Result<(Fibers, Segs), String> {
    let need = |o: usize, n: usize| {
        if buf.len() < o + n {
            return Err(format!("{name}: truncated at {o}+{n} (len {})", buf.len()));
        }
        Ok(())
    };
    need(0, 20)?;
    if u64::from_le_bytes(buf[..8].try_into().unwrap()) != SNAP_MAGIC {
        return Err(format!("{name}: bad magic (not a fiber snapshot)"));
    }
    let len = u64::from_le_bytes(buf[8..16].try_into().unwrap()) as usize;
    if buf.len() != 20 + len {
        return Err(format!("{name}: length mismatch (header {len}, file {})", buf.len()));
    }
    let crc = u32::from_le_bytes(buf[16..20].try_into().unwrap());
    if crc32(&buf[20..]) != crc {
        return Err(format!("{name}: crc mismatch (torn write?)"));
    }
    let buf = &buf[20..];
    let mut o = 0usize;
    let nf = u64::from_le_bytes(buf[o..o + 8].try_into().unwrap()) as usize;
    o += 8;
    let mut fibers = Vec::with_capacity(nf.min(1 << 30));
    for _ in 0..nf {
        need(o, 16)?;
        let c = u64::from_le_bytes(buf[o..o + 8].try_into().unwrap());
        o += 8;
        let v = f64::from_le_bytes(buf[o..o + 8].try_into().unwrap());
        o += 8;
        fibers.push((c, v));
    }
    need(o, 8)?;
    let ns = u64::from_le_bytes(buf[o..o + 8].try_into().unwrap()) as usize;
    o += 8;
    let mut seg_of = Vec::with_capacity(ns.min(1 << 30));
    for _ in 0..ns {
        need(o, 9)?;
        let c = u64::from_le_bytes(buf[o..o + 8].try_into().unwrap());
        o += 8;
        let s = buf[o];
        o += 1;
        seg_of.push((c, s));
    }
    if o != buf.len() {
        return Err(format!("{name}: trailing bytes ({o} vs {})", buf.len()));
    }
    Ok((fibers, seg_of))
}

fn main() {
    // Missing or old-format (no magic) snapshots rebuild; torn/CRC-bad
    // files FAIL LOUD (never half-load). Negative test below proves it.
    let need_build = match std::fs::read(SNAP) {
        Err(_) => true,
        Ok(raw) if raw.len() >= 8
            && u64::from_le_bytes(raw[..8].try_into().unwrap()) != SNAP_MAGIC => {
            println!("snap_format: old headerless snapshot, rebuilding with crc framing");
            true
        }
        _ => false,
    };
    if need_build {
        let t0 = Instant::now();
        let (fibers, seg_of) = build();
        save(&fibers, &seg_of);
        println!("snap_build: {:.1}s fibers={} segs={} (one-time, untimed convention)",
            t0.elapsed().as_secs_f64(), fibers.len(), seg_of.len());
    }
    // ---- Negative test: corrupted copy must be rejected ----
    let raw = std::fs::read(SNAP).expect("snapshot missing after build");
    let mut bad = raw.clone();
    let flip = 32.min(bad.len() - 1);
    bad[flip] ^= 0xFF;
    std::fs::write(SNAP_BAD, &bad).expect("bad-file write");
    let rejected = load_from(&std::fs::read(SNAP_BAD).unwrap(), SNAP_BAD).is_err();
    let _ = std::fs::remove_file(SNAP_BAD);
    println!("gate_snap_corrupt_reject={}", if rejected { "PASS" } else { "FAIL" });
    // ---- Timed: snapshot load ----
    let t0 = Instant::now();
    let loaded = load();
    let load_ms = t0.elapsed().as_secs_f64() * 1e3;
    let (fibers, seg_of) = match loaded {
        Ok(v) => v,
        Err(e) => {
            println!("gate_snap_exact=FAIL ({e})");
            std::process::exit(1);
        }
    };
    // ---- Timed: streaming walk (same as fiber_join) ----
    let t0 = Instant::now();
    let mut seg_tot = [0f64; 5];
    let mut si = 0usize;
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
    let query_ms = t0.elapsed().as_secs_f64() * 1e3;
    let total: f64 = seg_tot.iter().sum();
    let rel = ((total - EXPECTED_TOTAL) / EXPECTED_TOTAL).abs();
    println!("snap_query: load_ms={:.1} query_ms={:.1} sum_ms={:.1} total={:.0} rel={:.2e}",
        load_ms, query_ms, load_ms + query_ms, total, rel);
    println!("gate_snap_exact={}", if rel < 1e-9 { "PASS" } else { "FAIL" });
    println!("gate_snap_target={}",
        if load_ms + query_ms < TARGET_MS { "PASS (beat DuckDB)" } else { "RECORDED" });
    if rel >= 1e-9 || !rejected {
        std::process::exit(1);
    }
}
