// crates/sheaf-storage/src/lsp/wal.rs — log mutations with CRC
// FILE-017 (METHOD-002). Must never execute queries.
// Framed binary log: [MAGIC u32LE][len u32LE][crc32 u32LE][payload].
// A torn tail (short read or CRC mismatch) replays as `truncated=true`,
// never as an error: crash recovery stops at the last intact record.
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::OnceLock;

const MAGIC: u32 = 0x53484657; // "SHFW"
const HEADER_LEN: usize = 12;
// File layout: [valid_len u64LE][header_crc u32LE][frames...][padding].
// header_crc covers the valid_len bytes: a torn header rewrite (F2) fails
// validation and replay falls back to scan-to-first-invalid with
// truncated=true instead of silently dropping committed frames.
// Group-commit buffering: appends hit a 64 KB userspace buffer (per-op
// syscalls showed up directly in write P99); flush per seal.

fn header_crc(valid: u64) -> u32 {
    crc32(&valid.to_le_bytes())
}

fn crc_table() -> &'static [u32; 256] {
    static T: OnceLock<[u32; 256]> = OnceLock::new();
    T.get_or_init(|| {
        let mut t = [0u32; 256];
        for (i, s) in t.iter_mut().enumerate() {
            let mut c = i as u32;
            for _ in 0..8 {
                c = if c & 1 == 1 { 0xEDB88320 ^ (c >> 1) } else { c >> 1 };
            }
            *s = c;
        }
        t
    })
}

pub fn crc32(data: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c = crc_table()[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

pub struct WalWriter {
    buf: std::io::BufWriter<File>,
    written: u64,
}

impl WalWriter {
    pub fn create(path: &Path) -> std::io::Result<Self> {
        let file = File::create(path)?;
        // Pre-allocate: NTFS zero-fills on cluster extension, which showed
        // up directly in write P99. Standard WAL preallocation practice.
        file.set_len(8 << 20)?;
        let mut buf = std::io::BufWriter::with_capacity(64 << 10, file);
        buf.write_all(&0u64.to_le_bytes())?;
        buf.write_all(&0u32.to_le_bytes())?;
        Ok(Self { buf, written: HEADER_LEN as u64 })
    }
    pub fn append(&mut self, payload: &[u8]) -> std::io::Result<()> {
        let len = u32::try_from(payload.len()).expect("WAL payload overflow u32");
        self.buf.write_all(&MAGIC.to_le_bytes())?;
        self.buf.write_all(&len.to_le_bytes())?;
        self.buf.write_all(&crc32(payload).to_le_bytes())?;
        self.buf.write_all(payload)?;
        self.written += (12 + payload.len()) as u64;
        Ok(())
    }
    /// Group-commit point: flush frames, then commit the valid length plus
    /// its CRC. Crash order is data -> valid_len+crc, so recovery can always
    /// tell a torn header from a short file.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.buf.flush()?;
        let f = self.buf.get_mut();
        f.seek(SeekFrom::Start(0))?;
        f.write_all(&self.written.to_le_bytes())?;
        f.write_all(&header_crc(self.written).to_le_bytes())?;
        f.seek(SeekFrom::Start(self.written))?;
        f.flush()?;
        Ok(())
    }
}

/// Replay frames inside the committed valid region. Returns (records,
/// truncated). A header with a bad CRC is distrusted: replay falls back to
/// scanning the whole file for the first invalid frame and reports
/// truncated=true (a torn header can no longer silently shrink recovery).
/// Bytes past a VALID valid_len are prealloc padding / uncommitted tail and
/// are ignored, never errors.
pub fn replay(path: &Path) -> std::io::Result<(Vec<Vec<u8>>, bool)> {
    let mut f = File::open(path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    if buf.len() < HEADER_LEN {
        return Ok((Vec::new(), true));
    }
    let valid = u64::from_le_bytes(buf[..8].try_into().unwrap()) as usize;
    let hcrc = u32::from_le_bytes(buf[8..12].try_into().unwrap());
    if hcrc != header_crc(valid as u64) {
        // Distrusted header: every returned frame below was still
        // individually CRC-verified by scan(); the flag carries the doubt.
        let (out, _) = scan(&buf, HEADER_LEN, buf.len());
        return Ok((out, true));
    }
    if valid > buf.len() {
        // Committed region extends past EOF: fall through and parse what is
        // present, flagging truncation.
    }
    let end = valid.min(buf.len());
    let (out, torn) = scan(&buf, HEADER_LEN, end);
    if torn {
        return Ok((out, true));
    }
    Ok((out, valid > buf.len()))
}

/// Scan frames over buf[start..end]. Returns (records, torn_inside).
/// Every returned record passed magic + length + CRC; torn_inside means the
/// region itself ends mid-frame or corrupt (not just short).
fn scan(buf: &[u8], start: usize, end: usize) -> (Vec<Vec<u8>>, bool) {
    let mut out = Vec::new();
    let mut pos = start;
    while pos < end {
        if end - pos < 12 {
            return (out, true);
        }
        let magic = u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap());
        let len = u32::from_le_bytes(buf[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let crc = u32::from_le_bytes(buf[pos + 8..pos + 12].try_into().unwrap());
        if magic != MAGIC || end - pos - 12 < len {
            return (out, true);
        }
        let payload = &buf[pos + 12..pos + 12 + len];
        if crc32(payload) != crc {
            return (out, true);
        }
        out.push(payload.to_vec());
        pos += 12 + len;
    }
    (out, false)
}
