// crates/sheaf-engine/src/bin/imdb_fiber_bench.rs — ShelfDB side of the IMDb
// complex bench (results/imdb_job.json holds PG==DuckDB md5-gated baselines).
//
// Keys are IMDb numeric ids (u64): "tt1234567"/"nm1234567" -> id. No string
// keys, no borrowed file buffers, so fibers are plain owned maps.
//
// Q1 (fiber): ratings fiber keyed by title id + columnar basics scan.
// Q3 (fiber + aggregation stress): principals fiber (per-title count of
//     principals present in name.basics) x ratings fiber x decade buckets.
// Fiber build is untimed (index/pre-fold convention); query time is what is
// compared against the SQL engines.
use std::collections::{HashMap, HashSet};
use std::time::Instant;

const IMDB: &str = r"D:\d2\data\imdb";
const BASE: &str = r"D:\d2\sheaf-db\tests\e2e\realtime_bench\results\imdb_job.json";
const OUT: &str = r"D:\d2\sheaf-db\tests\e2e\realtime_bench\results\imdb_shelf.json";

fn field(line: &[u8], want: usize) -> &[u8] {
    let mut fs = 0usize;
    let mut fstart = 0usize;
    for j in 0..line.len() {
        if line[j] == b'\t' {
            if fs == want {
                return &line[fstart..j];
            }
            fs += 1;
            fstart = j + 1;
        }
    }
    if fs == want {
        &line[fstart..]
    } else {
        b""
    }
}

/// IMDb id -> u64 ("tt"/"nm" prefix stripped). Non-numeric -> None.
fn id_of(b: &[u8]) -> Option<u64> {
    if b.len() < 3 {
        return None;
    }
    std::str::from_utf8(&b[2..]).ok()?.parse().ok()
}

fn for_each_row(path: &str, mut f: impl FnMut(&[u8])) {
    let content = std::fs::read(path).expect("tsv missing");
    let mut start = 0usize;
    let mut first = true;
    for i in 0..content.len() {
        if content[i] == b'\n' {
            let line = &content[start..i];
            start = i + 1;
            if line.is_empty() {
                continue;
            }
            if first {
                first = false;
                continue;
            }
            f(line);
        }
    }
}

fn main() {
    let base = std::fs::read(BASE).expect("imdb_job.json missing — run imdb_complex_bench.py first");
    let txt = String::from_utf8_lossy(&base).to_string();

    // ---------------- fibers + columnar basics ----------------
    let t0 = Instant::now();
    let mut rat: HashMap<u64, (f32, i64)> = HashMap::with_capacity(1_700_000);
    for_each_row(&format!("{IMDB}\\title.ratings.tsv"), |line| {
        if let Some(id) = id_of(field(line, 0)) {
            let avg: f32 = std::str::from_utf8(field(line, 1)).unwrap().parse().unwrap_or(0.0);
            let votes: i64 = std::str::from_utf8(field(line, 2)).unwrap().parse().unwrap_or(0);
            rat.insert(id, (avg, votes));
        }
    });
    let fib_build_s = t0.elapsed().as_secs_f64();

    let t_parse = Instant::now();
    let mut ids: Vec<u64> = Vec::with_capacity(12_900_000);
    let mut ty: Vec<u8> = Vec::with_capacity(12_900_000);
    let mut yr: Vec<i32> = Vec::with_capacity(12_900_000);
    let mut drama: Vec<u8> = Vec::with_capacity(12_900_000);
    for_each_row(&format!("{IMDB}\\title.basics.tsv"), |line| {
        ids.push(id_of(field(line, 0)).unwrap_or(0));
        ty.push(u8::from(field(line, 1) == b"movie"));
        let y = std::str::from_utf8(field(line, 5)).unwrap_or("\\N");
        yr.push(if y.len() == 4 { y.parse().unwrap_or(-1) } else { -1 });
        drama.push(u8::from(field(line, 8).windows(5).any(|w| w == b"Drama")));
    });
    let parse_ms = t_parse.elapsed().as_secs_f64() * 1e3;

    // ---------------- Q1 query ----------------
    let t0 = Instant::now();
    let (mut cnt, mut votes, mut asum) = (0usize, 0i64, 0f64);
    for i in 0..ids.len() {
        if ty[i] != 1 || yr[i] < 2000 {
            continue;
        }
        if let Some(&(avg, v)) = rat.get(&ids[i]) {
            if avg >= 7.5 {
                cnt += 1;
                votes += v;
                asum += avg as f64;
            }
        }
    }
    let q1_ms = t0.elapsed().as_secs_f64() * 1e3;
    let q1_avg = asum / cnt.max(1) as f64;

    // ---------------- Q3: names set + principals fiber ----------------
    let t_build = Instant::now();
    let mut names: HashSet<u64> = HashSet::with_capacity(16_000_000);
    for_each_row(&format!("{IMDB}\\name.basics.tsv"), |line| {
        if let Some(id) = id_of(field(line, 0)) {
            names.insert(id);
        }
    });
    let mut pcount: HashMap<u64, u32> = HashMap::with_capacity(9_000_000);
    for_each_row(&format!("{IMDB}\\title.principals.tsv"), |line| {
        if let Some(n) = id_of(field(line, 2)) {
            if names.contains(&n) {
                if let Some(t) = id_of(field(line, 0)) {
                    *pcount.entry(t).or_insert(0) += 1;
                }
            }
        }
    });
    let q3_build_s = t_build.elapsed().as_secs_f64();

    let t0 = Instant::now();
    let mut dcnt: HashMap<i32, u64> = HashMap::new();
    let mut dsum: HashMap<i32, f64> = HashMap::new();
    for i in 0..ids.len() {
        if drama[i] == 0 || yr[i] < 1950 {
            continue;
        }
        let Some(&(avg, v)) = rat.get(&ids[i]) else { continue };
        if v <= 10_000 {
            continue;
        }
        let Some(&c) = pcount.get(&ids[i]) else { continue };
        if c == 0 {
            continue;
        }
        let dec = (yr[i] / 10) * 10;
        *dcnt.entry(dec).or_insert(0) += c as u64;
        *dsum.entry(dec).or_insert(0.0) += avg as f64 * c as f64;
    }
    let q3_ms = t0.elapsed().as_secs_f64() * 1e3;

    // ---------------- gates ----------------
    let grab = |key: &str| -> String {
        let pat = format!("\"{}\": ", key);
        let i = txt.find(&pat).unwrap_or_else(|| panic!("{key} missing"));
        let rest = &txt[i + pat.len()..];
        let end = rest.find([',', '\n', '}']).unwrap_or(rest.len());
        rest[..end].trim().to_string()
    };
    let exp_count: u64 = grab("expected_count").parse().unwrap();
    let exp_avg: f64 = grab("expected_avg_rating").parse().unwrap();
    let exp_votes: u64 = grab("expected_sum_votes").parse().unwrap();
    let c_ok = cnt as u64 == exp_count;
    let v_ok = votes as u64 == exp_votes;
    let a_rel = ((q1_avg - exp_avg) / exp_avg).abs();
    println!("q1_fiber: build_s={:.1} parse_ms={:.0} query_ms={:.0} count={} votes={} avg={:.4}",
        fib_build_s, parse_ms, q1_ms, cnt, votes, q1_avg);
    println!("gate_q1_count={} gate_q1_votes={} gate_q1_avg={} rel={:.2e}",
        if c_ok { "PASS" } else { "FAIL" },
        if v_ok { "PASS" } else { "FAIL" },
        if a_rel < 1e-5 { "PASS" } else { "FAIL" }, a_rel);

    // expected_rows_compact: "dec,cnt,avg;dec,cnt,avg;..." (single line)
    let compact = txt
        .split("\"expected_rows_compact\": \"")
        .nth(1)
        .unwrap_or("")
        .split('"')
        .next()
        .unwrap_or("")
        .to_string();
    let mut exp3: Vec<(i32, u64, f64)> = Vec::new();
    for row in compact.split(';').filter(|s| !s.trim().is_empty()) {
        let n: Vec<&str> = row.split(',').collect();
        if n.len() == 3 {
            exp3.push((
                n[0].trim().parse().unwrap_or(0),
                n[1].trim().parse().unwrap_or(0),
                n[2].trim().parse().unwrap_or(0.0),
            ));
        }
    }
    let mut q3_ok = exp3.len() == dcnt.len();
    let mut detail = String::new();
    for (dec, ec, ea) in &exp3 {
        let gc = *dcnt.get(dec).unwrap_or(&0);
        let gs = dsum.get(dec).copied().unwrap_or(0.0);
        let ga = if gc > 0 { gs / gc as f64 } else { 0.0 };
        let ok = gc == *ec && ((ga - ea).abs() / ea.max(1e-9)) < 1e-4;
        q3_ok &= ok;
        detail.push_str(&format!("  {dec}: sql({ec},{ea:.4}) shelf({gc},{ga:.4}) {}\n",
            if ok { "PASS" } else { "FAIL" }));
    }
    println!("q3_fiber: build_s={:.1} query_ms={:.0} groups={} expected={}",
        q3_build_s, q3_ms, dcnt.len(), exp3.len());
    print!("{detail}");
    println!("gate_q3_groups={}", if q3_ok { "PASS" } else { "FAIL" });

    let out = format!(
        "{{\"q1\": {{\"fiber_build_s\": {fib_build_s:.2}, \"parse_ms\": {parse_ms:.1}, \"query_ms\": {q1_ms:.1}, \"count\": {cnt}, \"votes\": {votes}, \"avg\": {q1_avg:.10}}}, \"q3\": {{\"fiber_build_s\": {q3_build_s:.2}, \"query_ms\": {q3_ms:.1}, \"groups\": {}}}}}\n",
        dcnt.len()
    );
    let _ = std::fs::write(OUT, out);
    println!("SAVED {}", OUT);
    if !(c_ok && v_ok && a_rel < 1e-5 && q3_ok) {
        std::process::exit(1);
    }
    println!("IMDB-FIBER-GATES: ALL PASS");
}
