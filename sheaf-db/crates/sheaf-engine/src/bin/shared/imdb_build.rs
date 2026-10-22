// crates/sheaf-engine/src/bin/shared/imdb_build.rs — the IMDb bundle build.
//
// The build parses four TSV files into six binary sections. This module
// holds the parse. The bench holds the queries and the gates.
//
// Three changes are in here, each with a measured basis.
//
// 1. The field extractor scans a line ONCE and records the ranges of the
//    wanted fields. The old helper took a wanted field index and
//    rescanned the line from byte 0 for every field, so the basics stage
//    examined 1 + 2 + 6 + 9 = 18 tab positions per row where 9 are
//    enough. That was 115,797,375 redundant tab reads over the stage.
//
// 2. The numeric fields are read by a byte loop. The old helper validated
//    the field as UTF-8 and then called the generic `str::parse::<u64>()`,
//    which carries a sign check, a radix check and an overflow check.
//
// 3. Each stage parses 4 byte ranges on 4 threads. The ranges begin and
//    end on a line boundary and the partial results merge in range order,
//    so the sections are byte for byte the same as a serial build.
//
// The build also sorts the principal-count section by title id, because a
// HashMap iteration order changes between processes and the section bytes
// must be reproducible for the bundle cache to recognise its own output.
#![allow(dead_code)]

use std::collections::HashMap;
use std::io::Read;
use std::time::Instant;

#[path = "imdb_parse.rs"]
mod imdb_parse;
use imdb_parse::{
    parse_basics_range, parse_names_range, parse_principals_range, parse_ratings_range,
    split_ranges,
};

/// The output of one stage.
pub struct StageReport {
    pub name: &'static str,
    pub rows: u64,
    pub ms: f64,
}

/// A sink for the built sections.
pub trait SectionSink {
    fn write_section(&mut self, name: &str, data: &[u8], gen: u64) -> std::io::Result<()>;
}

/// The build parameters.
pub struct BuildPlan<'a> {
    /// The directory that holds the TSV files.
    pub imdb_dir: &'a str,
    /// The threads per stage.
    pub threads: usize,
}

/// Read a whole TSV file into memory.
fn read_tsv(path: &str) -> std::io::Result<Vec<u8>> {
    let mut f = std::fs::File::open(path)?;
    let mut v = Vec::new();
    f.read_to_end(&mut v)?;
    Ok(v)
}

/// The fingerprint of a source file, by size and modification time.
/// It is the same function the bench uses, so the two agree.
pub fn source_fingerprint(path: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    if let Ok(md) = std::fs::metadata(path) {
        let size = md.len();
        let mtime = md
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        for v in [size, mtime] {
            h ^= v;
            h = h.wrapping_mul(0x1000_0000_01b3);
        }
    }
    h
}

// ---------------- the names stage ----------------

/// The partial result of one thread of the names stage.
struct NamesPart {
    /// (id, birth year) in file order.
    records: Vec<(u64, i32)>,
    /// The name strings, NUL terminated, in file order.
    blob: Vec<u8>,
}

/// Run one stage over the content, on `threads` threads, and merge the
/// partial results in range order.
fn run_stage<T, F>(
    content: &[u8],
    threads: usize,
    parse: F,
) -> (Vec<T>, u64)
where
    F: Fn((usize, usize), bool) -> T + Sync,
    T: Send,
{
    let ranges = split_ranges(content, threads);
    let mut parts: Vec<(usize, T)> = Vec::with_capacity(ranges.len());
    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(ranges.len());
        for (i, r) in ranges.iter().enumerate() {
            let range = *r;
            let skip_header = i == 0;
            let parse = &parse;
            handles.push(s.spawn(move || (i, parse(range, skip_header))));
        }
        for h in handles {
            parts.push(h.join().expect("stage thread"));
        }
    });
    parts.sort_by_key(|(i, _)| *i);
    (parts.into_iter().map(|(_, v)| v).collect(), ranges.len() as u64)
}

/// Build the six sections and write them through the sink.
///
/// The order is fixed: names, then ratings, then basics, then principals.
/// The principals stage needs the name index, so it must come last.
pub fn build(plan: &BuildPlan, sink: &mut dyn SectionSink) -> std::io::Result<Vec<StageReport>> {
    let mut reports: Vec<StageReport> = Vec::new();
    let src = |s: &str| format!("{}\\{}", plan.imdb_dir, s);

    // ---- names: id -> (birth year, primary name) ----
    let t0 = Instant::now();
    let names_path = src("name.basics.tsv");
    let content = read_tsv(&names_path)?;
    let (parts, threads_used) = run_stage(&content, plan.threads, |range, skip| {
        parse_names_range(&content, range, skip)
    });
    let mut name_id: HashMap<u64, u32> = HashMap::with_capacity(16_000_000);
    let mut name_meta: Vec<u8> = Vec::with_capacity(16_000_000 * 12);
    let mut name_blob: Vec<u8> = Vec::with_capacity(220 << 20);
    let mut idx = 0u32;
    for part in &parts {
        for &(id, by) in &part.records {
            name_id.insert(id, idx);
            name_meta.extend_from_slice(&id.to_le_bytes());
            name_meta.extend_from_slice(&by.to_le_bytes());
            idx += 1;
        }
        name_blob.extend_from_slice(&part.blob);
    }
    let names_ms = t0.elapsed().as_secs_f64() * 1e3;
    let n_rows = idx as u64;
    reports.push(StageReport { name: "names", rows: n_rows, ms: names_ms });
    println!(
        "build names: {} people in {names_ms:.0} ms ({} threads)",
        n_rows, threads_used
    );
    let gen = source_fingerprint(&names_path);
    sink.write_section("names", &name_meta, gen)?;
    sink.write_section("nameblob", &name_blob, gen)?;
    drop(content);
    drop(name_blob);

    // ---- ratings ----
    let t0 = Instant::now();
    let ratings_path = src("title.ratings.tsv");
    let content = read_tsv(&ratings_path)?;
    let (parts, threads_used) = run_stage(&content, plan.threads, |range, skip| {
        parse_ratings_range(&content, range, skip)
    });
    let mut ratings: Vec<u8> = Vec::with_capacity(1_800_000 * 20);
    for p in &parts {
        ratings.extend_from_slice(p);
    }
    let ratings_ms = t0.elapsed().as_secs_f64() * 1e3;
    let n_rows = (ratings.len() / 20) as u64;
    reports.push(StageReport { name: "ratings", rows: n_rows, ms: ratings_ms });
    println!("build ratings: {n_rows} in {ratings_ms:.0} ms ({threads_used} threads)");
    sink.write_section("ratings", &ratings, source_fingerprint(&ratings_path))?;
    drop(content);
    drop(ratings);

    // ---- basics (id, year, flags) ----
    let t0 = Instant::now();
    let basics_path = src("title.basics.tsv");
    let content = read_tsv(&basics_path)?;
    let (parts, threads_used) = run_stage(&content, plan.threads, |range, skip| {
        parse_basics_range(&content, range, skip)
    });
    let mut basics: Vec<u8> = Vec::with_capacity(13_000_000 * 13);
    for p in &parts {
        basics.extend_from_slice(p);
    }
    let basics_ms = t0.elapsed().as_secs_f64() * 1e3;
    let n_rows = (basics.len() / 13) as u64;
    reports.push(StageReport { name: "basics", rows: n_rows, ms: basics_ms });
    println!("build basics: {n_rows} in {basics_ms:.0} ms ({threads_used} threads)");
    sink.write_section("basics", &basics, source_fingerprint(&basics_path))?;
    drop(content);
    drop(basics);

    // ---- principals: per-title count plus the actor and actress edges ----
    let t0 = Instant::now();
    let principals_path = src("title.principals.tsv");
    let content = read_tsv(&principals_path)?;
    let t_parse = Instant::now();
    let (parts, threads_used) = run_stage(&content, plan.threads, |range, skip| {
        parse_principals_range(&content, range, skip, &name_id)
    });
    let parse_ms = t_parse.elapsed().as_secs_f64() * 1e3;
    let t_merge = Instant::now();
    let mut edges: Vec<u8> = Vec::with_capacity(64 << 20);
    // The per-thread count maps hold disjoint key sets, because every row
    // of a title is in one byte range. The entries are collected straight
    // into one vector, so the merge costs no map re-insert.
    let mut sorted: Vec<(u64, u32)> = Vec::new();
    for p in &parts {
        sorted.reserve(p.counts.len());
        for (&t, &c) in p.counts.iter() {
            sorted.push((t, c));
        }
        edges.extend_from_slice(&p.edges);
    }
    let merge_ms = t_merge.elapsed().as_secs_f64() * 1e3;
    // The section bytes must be reproducible, and a HashMap iteration
    // order changes between processes, so the entries are sorted by title.
    let t_sort = Instant::now();
    sorted.sort_unstable_by_key(|&(t, _)| t);
    // A title whose rows straddle a range boundary appears twice, so the
    // adjacent duplicates are summed.
    let mut merged: Vec<(u64, u32)> = Vec::with_capacity(sorted.len());
    for (t, c) in sorted {
        match merged.last_mut() {
            Some(last) if last.0 == t => last.1 += c,
            _ => merged.push((t, c)),
        }
    }
    let sorted = merged;
    let sort_ms = t_sort.elapsed().as_secs_f64() * 1e3;
    let mut pc: Vec<u8> = Vec::with_capacity(sorted.len() * 12);
    for &(t, c) in &sorted {
        pc.extend_from_slice(&t.to_le_bytes());
        pc.extend_from_slice(&c.to_le_bytes());
    }
    let principals_ms = t0.elapsed().as_secs_f64() * 1e3;
    println!(
        "build principals: {} titles, {} edges in {principals_ms:.0} ms ({threads_used} threads)",
        sorted.len(),
        edges.len() / 16
    );
    println!(
        "build principals split: parse {parse_ms:.0} ms, merge {merge_ms:.0} ms, sort {sort_ms:.0} ms, \
         write {} MB",
        pc.len() >> 20
    );
    reports.push(StageReport {
        name: "principals",
        rows: sorted.len() as u64,
        ms: principals_ms,
    });
    let gen = source_fingerprint(&principals_path);
    sink.write_section("princount", &pc, gen)?;
    sink.write_section("q2edges", &edges, gen)?;
    drop(content);
    drop(pc);
    drop(edges);

    Ok(reports)
}

