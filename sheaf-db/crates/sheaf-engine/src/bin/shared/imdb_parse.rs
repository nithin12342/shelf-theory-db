// crates/sheaf-engine/src/bin/shared/imdb_parse.rs — the TSV parse of the
// IMDb source files.
//
// This module holds the parsing. The build driver is in imdb_build.rs.
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
// 3. The parse of a stage runs over several byte ranges at once. The
//    ranges begin and end on a line boundary and the partial results merge
//    in range order, so the output is the same as a serial parse.
#![allow(dead_code)]

/// The most fields any IMDb line has, with room for one more.
pub const MAX_FIELDS: usize = 16;

/// The byte range of the wanted fields of one TSV line, found in one scan.
///
/// The caller owns the scratch space and reuses it for every line, so the
/// per-line cost is one pass over the line plus one comparison per
/// separator, with no allocation.
pub struct FieldScratch {
    start: [u32; MAX_FIELDS],
    end: [u32; MAX_FIELDS],
}

impl FieldScratch {
    pub fn new() -> Self {
        Self { start: [0; MAX_FIELDS], end: [0; MAX_FIELDS] }
    }

    /// Scan the line once and record the range of each wanted field.
    ///
    /// `want` holds the wanted field indexes in ascending order. The
    /// index into `want` is the key for `get`.
    ///
    /// A wanted field the line does not have is an empty range, which is
    /// what the old per-field scanner returned.
    pub fn scan(&mut self, line: &[u8], want: &[u8]) {
        let mut wi = 0usize;
        let mut f = 0usize;
        let mut fstart = 0usize;
        for (j, &c) in line.iter().enumerate() {
            if c != b'\t' {
                continue;
            }
            while wi < want.len() && want[wi] as usize == f {
                self.start[wi] = fstart as u32;
                self.end[wi] = j as u32;
                wi += 1;
            }
            f += 1;
            fstart = j + 1;
        }
        // The last field of the line has no trailing separator.
        if wi < want.len() && want[wi] as usize == f {
            self.start[wi] = fstart as u32;
            self.end[wi] = line.len() as u32;
            wi += 1;
        }
        // A wanted field the line does not have is empty.
        while wi < want.len() {
            self.start[wi] = 0;
            self.end[wi] = 0;
            wi += 1;
        }
    }

    /// The bytes of the wanted field `k`, where `k` indexes `want`.
    pub fn get<'a>(&self, line: &'a [u8], k: usize) -> &'a [u8] {
        &line[self.start[k] as usize..self.end[k] as usize]
    }
}

/// Parse the ASCII decimal id from a TSV id field.
///
/// The IMDb id fields are a two letter prefix and the digits, so the
/// prefix is skipped and the rest is read by a digit loop. A field that
/// is not all digits, or that is longer than 19 digits, gives 0, which
/// is the same answer the generic parse gave for every value this data
/// holds.
pub fn parse_id(b: &[u8]) -> u64 {
    if b.len() < 3 {
        return 0;
    }
    let d = &b[2..];
    if d.is_empty() || d.len() > 19 {
        return 0;
    }
    let mut v = 0u64;
    for &c in d {
        if !c.is_ascii_digit() {
            return 0;
        }
        v = v * 10 + (c - b'0') as u64;
    }
    v
}

/// Parse a four digit year from a field.
///
/// The old code read the year only when the field was exactly 4
/// characters long, and gave 0 for anything else. A byte loop over 4
/// digits gives the same answer.
pub fn parse_year(b: &[u8]) -> i32 {
    if b.len() != 4 {
        return 0;
    }
    let mut v = 0i32;
    for &c in b {
        if !c.is_ascii_digit() {
            return 0;
        }
        v = v * 10 + (c - b'0') as i32;
    }
    v
}

/// Parse an ASCII decimal integer from a field, as a signed count.
pub fn parse_i64(b: &[u8]) -> i64 {
    if b.is_empty() || b.len() > 18 {
        return 0;
    }
    let mut v = 0i64;
    for &c in b {
        if !c.is_ascii_digit() {
            return 0;
        }
        v = v * 10 + (c - b'0') as i64;
    }
    v
}

/// Split the content into byte ranges that begin and end on a line
/// boundary. The first range starts at 0 and the last ends at the end of
/// the content, so every line belongs to exactly one range.
pub fn split_ranges(content: &[u8], threads: usize) -> Vec<(usize, usize)> {
    let n = content.len();
    if threads <= 1 || n == 0 {
        return vec![(0, n)];
    }
    let mut bounds: Vec<usize> = Vec::with_capacity(threads + 1);
    bounds.push(0);
    for t in 1..threads {
        let target = n * t / threads;
        let mut p = target;
        while p < n && content[p] != b'\n' {
            p += 1;
        }
        if p < n {
            p += 1;
        }
        if p > bounds[bounds.len() - 1] && p < n {
            bounds.push(p);
        }
    }
    bounds.push(n);
    bounds.windows(2).map(|w| (w[0], w[1])).collect()
}

/// The lines of one range, as byte slices. The header is the first line
/// of the file, so only the range that starts at 0 skips it.
pub fn lines_of(
    content: &[u8],
    range: (usize, usize),
    skip_header: bool,
) -> impl Iterator<Item = &[u8]> {
    let (start, end) = range;
    let body = &content[start..end];
    let mut first = true;
    body.split(|&c| c == b'\n').filter(move |line| {
        if line.is_empty() {
            return false;
        }
        if first {
            first = false;
            return !skip_header;
        }
        true
    })
}

/// The partial result of one thread of the names stage.
pub struct NamesPart {
    /// (id, birth year) in file order.
    pub records: Vec<(u64, i32)>,
    /// The name strings, NUL terminated, in file order.
    pub blob: Vec<u8>,
}

/// Parse one range of name.basics.tsv.
pub fn parse_names_range(content: &[u8], range: (usize, usize), skip_header: bool) -> NamesPart {
    const WANT: [u8; 3] = [0, 1, 2];
    let mut fs = FieldScratch::new();
    let mut part = NamesPart { records: Vec::new(), blob: Vec::new() };
    for line in lines_of(content, range, skip_header) {
        fs.scan(line, &WANT);
        let id = parse_id(fs.get(line, 0));
        if id == 0 {
            continue;
        }
        let by = parse_year(fs.get(line, 2));
        part.records.push((id, by));
        part.blob.extend_from_slice(fs.get(line, 1));
        part.blob.push(0);
    }
    part
}

/// Parse one range of title.ratings.tsv into 20-byte records.
pub fn parse_ratings_range(content: &[u8], range: (usize, usize), skip_header: bool) -> Vec<u8> {
    const WANT: [u8; 3] = [0, 1, 2];
    let mut fs = FieldScratch::new();
    let mut out: Vec<u8> = Vec::new();
    for line in lines_of(content, range, skip_header) {
        fs.scan(line, &WANT);
        let id = parse_id(fs.get(line, 0));
        if id == 0 {
            continue;
        }
        let avg: f32 = std::str::from_utf8(fs.get(line, 1)).unwrap_or("0").parse().unwrap_or(0.0);
        let votes: i64 = parse_i64(fs.get(line, 2));
        out.extend_from_slice(&id.to_le_bytes());
        out.extend_from_slice(&avg.to_le_bytes());
        out.extend_from_slice(&votes.to_le_bytes());
    }
    out
}

/// Parse one range of title.basics.tsv into 13-byte records.
pub fn parse_basics_range(content: &[u8], range: (usize, usize), skip_header: bool) -> Vec<u8> {
    const WANT: [u8; 4] = [0, 1, 5, 8];
    let mut fs = FieldScratch::new();
    let mut out: Vec<u8> = Vec::new();
    for line in lines_of(content, range, skip_header) {
        fs.scan(line, &WANT);
        let id = parse_id(fs.get(line, 0));
        if id == 0 {
            continue;
        }
        let ty = fs.get(line, 1);
        let y = parse_year(fs.get(line, 2));
        let mut flags = 0u8;
        if ty == b"movie" {
            flags |= 1;
        }
        if fs.get(line, 3).windows(5).any(|w| w == b"Drama") {
            flags |= 2;
        }
        out.extend_from_slice(&id.to_le_bytes());
        out.extend_from_slice(&y.to_le_bytes());
        out.push(flags);
    }
    out
}

/// The partial result of one thread of the principals stage.
pub struct PrincipalsPart {
    /// The per-title count of named people.
    pub counts: std::collections::HashMap<u64, u32>,
    /// The actor and actress edges, 16 bytes each, in file order.
    pub edges: Vec<u8>,
}

/// Parse one range of title.principals.tsv.
pub fn parse_principals_range(
    content: &[u8],
    range: (usize, usize),
    skip_header: bool,
    name_id: &std::collections::HashMap<u64, u32>,
) -> PrincipalsPart {
    const WANT: [u8; 3] = [0, 2, 3];
    let mut fs = FieldScratch::new();
    let mut part = PrincipalsPart { counts: std::collections::HashMap::new(), edges: Vec::new() };
    for line in lines_of(content, range, skip_header) {
        fs.scan(line, &WANT);
        let tid = parse_id(fs.get(line, 0));
        let nid = parse_id(fs.get(line, 1));
        if tid == 0 || nid == 0 {
            continue;
        }
        if !name_id.contains_key(&nid) {
            continue;
        }
        *part.counts.entry(tid).or_insert(0) += 1;
        let cat = fs.get(line, 2);
        if cat == b"actor" || cat == b"actress" {
            part.edges.extend_from_slice(&nid.to_le_bytes());
            part.edges.extend_from_slice(&tid.to_le_bytes());
        }
    }
    part
}
