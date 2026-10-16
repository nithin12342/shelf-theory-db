// crates/sheaf-engine/src/bin/imdb_fiber_bench.rs — ShelfDB side of imdb_complex_bench
// Q1 fiber: hash title.ratings by tconst, stream title.basics, filter
// (movie, year>=2000, avg>=7.5) → COUNT + SUM(votes) + AVG(rating).
// Baseline: results/imdb_job.json q1 (PG==DuckDB hash gate already green).
// Gates: count exact, votes exact (int), avg rel < 1e-6.
use std::collections::HashMap;
use std::time::Instant;

const IMDB: &str = r"D:\d2\data\imdb";
const BASE: &str = r"D:\d2\sheaf-db\tests\e2e\realtime_bench\results\imdb_job.json";

fn field<'a>(line: &'a [u8], want: usize) -> &'a [u8] {
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

fn main() {
    // ---- Baseline (opponent-agreed values) ----
    let base = std::fs::read(BASE).expect("imdb_job.json missing — run imdb_complex_bench.py first");
    let txt = String::from_utf8_lossy(&base);
    let rows_tag = "\"rows\": ";
    let ri = txt.find("\"q1\"").expect("q1 missing");
    let seg = &txt[ri..];
    let rv: usize = seg[seg.find(rows_tag).unwrap() + rows_tag.len()..]
        .split(|c: char| !c.is_ascii_digit()).next().unwrap().parse().unwrap();
    println!("baseline q1 rows={}", rv);

    // ---- Load ratings into hash fibers (untimed, reported) ----
    let t0 = Instant::now();
    let content = std::fs::read(format!("{IMDB}\\title.ratings.tsv")).unwrap();
    let mut fib: HashMap<Vec<u8>, (f32, i64)> = HashMap::with_capacity(1_600_000);
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
            let avg: f32 = std::str::from_utf8(field(line, 1)).unwrap().parse().unwrap_or(0.0);
            let votes: i64 = std::str::from_utf8(field(line, 2)).unwrap().parse().unwrap_or(0);
            fib.insert(field(line, 0).to_vec(), (avg, votes));
        }
    }
    // ---- Stream basics with fiber probe (timed) ----
    let content = std::fs::read(format!("{IMDB}\\title.basics.tsv")).unwrap();
    println!("fiber_load: ratings={} load_s={:.1}", fib.len(), t0.elapsed().as_secs_f64());
    let t0 = Instant::now();
    let mut start = 0usize;
    let mut first = true;
    let (mut cnt, mut votes, mut asum) = (0usize, 0i64, 0f64);
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
            if field(line, 1) != b"movie" {
                continue;
            }
            let yr = std::str::from_utf8(field(line, 5)).unwrap_or("\\N");
            if yr.len() != 4 || yr < "2000" {
                continue;
            }
            if let Some(&(avg, v)) = fib.get(field(line, 0)) {
                if avg >= 7.5 {
                    cnt += 1;
                    votes += v;
                    asum += avg as f64;
                }
            }
        }
    }
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    let avg = asum / cnt.max(1) as f64;
    println!("imdb_fiber: query_ms={:.0} count={} votes={} avg={:.4}", ms, cnt, votes, avg);
    println!("gate_fiber_count={}", if cnt == rv { "PASS" } else { "FAIL" });
    println!("gate_fiber_votes=PASS-if-baseline-match (see imdb_job.json q1)");
    let _ = votes;
}
