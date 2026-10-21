// crates/sheaf-storage/src/bcsf/segment.rs — durable segment descriptor
// (B8 / METHOD-045, REQ-050 / SPEC-050).
//
// Until now a BCSF segment existed only as in-memory slabs: the compactor
// built a new `BcsfReader` and dropped the old one. There was no on-disk
// form, so a segment could not be opened without re-running the builder.
//
// This writes a segment as a self-describing file:
//   [header][offsets][indices][stalk_offsets][morph_cells][morph_offsets][discrete][stalks]
// The header carries magic, format version, cell count, generation, source
// fingerprint, each part's offset/length, and a CRC32 over the payload.
// A reader can open it directly (no builder run) and a corrupt file is
// rejected, never half-loaded.
use std::path::Path;

use crate::lsp::ring_wal::crc_hw;

use super::arena::StalkArena;
use super::indices::IncidenceIndices;
use super::offsets::CellOffsets;
use super::reader::BcsfReader;

pub const SEGMENT_MAGIC: u64 = 0x5347_534D_4E54_3031; // "SGSMNT01"
pub const SEGMENT_FMT_VER: u32 = 1;
/// Number of payload parts (offsets, indices, stalk_offsets, morph_cells,
/// morph_offsets, discrete, morphs, stalks, dims).
pub const PART_COUNT: usize = 9;
/// magic|ver|ncells|generation|fingerprint|9 parts (16 B each)|crc
pub const HEADER_LEN: usize = 8 + 4 + 8 + 8 + 8 + PART_COUNT * 16 + 4;

/// A part's location in the file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Part {
    pub off: u64,
    pub len: u64,
}

fn put_part(h: &mut Vec<u8>, p: Part) {
    h.extend_from_slice(&p.off.to_le_bytes());
    h.extend_from_slice(&p.len.to_le_bytes());
}

/// Write a reader to a durable segment file.
pub fn write_segment(reader: &BcsfReader, path: &Path, generation: u64, src_fingerprint: u64) -> Result<(), String> {
    // B5: a shadow-paging chain has no flat slabs of its own - flatten it
    // (the segment file is a flat format).
    if reader.chain.is_some() {
        let flat = reader.flatten();
        return write_segment(&flat, path, generation, src_fingerprint);
    }
    let mut header: Vec<u8> = Vec::with_capacity(HEADER_LEN);
    header.extend_from_slice(&SEGMENT_MAGIC.to_le_bytes());
    header.extend_from_slice(&SEGMENT_FMT_VER.to_le_bytes());
    header.extend_from_slice(&(reader.len() as u64).to_le_bytes());
    header.extend_from_slice(&generation.to_le_bytes());
    header.extend_from_slice(&src_fingerprint.to_le_bytes());
    // payload parts are written in a fixed order; their offsets are known
    // once the payload is assembled.
    let mut payload: Vec<u8> = Vec::new();
    let mut parts = [Part::default(); PART_COUNT];
    fn push(
        payload: &mut Vec<u8>,
        parts: &mut [Part; PART_COUNT],
        idx: usize,
        bytes: &[u8],
    ) {
        parts[idx] = Part { off: payload.len() as u64, len: bytes.len() as u64 };
        payload.extend_from_slice(bytes);
    }
    push(&mut payload, &mut parts, 0, &u32_bytes(&reader.offsets.offsets));
    push(&mut payload, &mut parts, 1, &u64_bytes(&reader.indices.indices));
    push(&mut payload, &mut parts, 2, &u32_bytes(&reader.arena.stalk_offsets));
    push(&mut payload, &mut parts, 3, &u32_bytes(&reader.arena.morph_cells));
    push(&mut payload, &mut parts, 4, &u32_bytes(&reader.arena.morph_offsets));
    push(&mut payload, &mut parts, 5, &reader.arena.discrete);
    push(&mut payload, &mut parts, 6, &f32_bytes(&reader.arena.morphs));
    push(&mut payload, &mut parts, 7, &f32_bytes(&reader.arena.stalks));
    push(&mut payload, &mut parts, 8, &reader.dims);
    let payload_crc = crc_hw(&payload);
    for p in parts {
        put_part(&mut header, p);
    }
    header.extend_from_slice(&payload_crc.to_le_bytes());
    debug_assert_eq!(header.len(), HEADER_LEN);
    let mut file = std::fs::File::create(path).map_err(|e| format!("{path:?}: {e}"))?;
    use std::io::Write;
    file.write_all(&header).map_err(|e| format!("{path:?}: {e}"))?;
    file.write_all(&payload).map_err(|e| format!("{path:?}: {e}"))?;
    Ok(())
}

fn u32_bytes(v: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out
}

fn u64_bytes(v: &[u64]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 8);
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out
}

fn f32_bytes(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out
}

/// Open a durable segment file into a reader (no builder run).
pub fn open_segment(path: &Path) -> Result<BcsfReader, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path:?}: {e}"))?;
    if bytes.len() < HEADER_LEN {
        return Err(format!("{path:?}: truncated header"));
    }
    if u64::from_le_bytes(bytes[..8].try_into().unwrap()) != SEGMENT_MAGIC {
        return Err(format!("{path:?}: bad magic (not a segment)"));
    }
    let ver = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
    if ver != SEGMENT_FMT_VER {
        return Err(format!("{path:?}: format version {ver} != {SEGMENT_FMT_VER}"));
    }
    let ncells = u64::from_le_bytes(bytes[12..20].try_into().unwrap()) as usize;
    // layout: magic|ver|ncells|generation|fingerprint|9 parts|crc
    let mut off = 36usize;
    let mut parts = [Part::default(); PART_COUNT];
    for p in parts.iter_mut() {
        p.off = u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap());
        p.len = u64::from_le_bytes(bytes[off + 8..off + 16].try_into().unwrap());
        off += 16;
    }
    let crc = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
    let payload = &bytes[HEADER_LEN..];
    if crc_hw(payload) != crc {
        return Err(format!("{path:?}: payload crc mismatch (torn write?)"));
    }
    let slice = |p: Part| -> Result<&[u8], String> {
        let s = p.off as usize;
        let e = s.checked_add(p.len as usize).ok_or("part overflow")?;
        if e > payload.len() {
            return Err("part outside payload".to_string());
        }
        Ok(&payload[s..e])
    };
    let u32s = |b: &[u8]| -> Vec<u32> {
        b.as_chunks::<4>()
            .0
            .iter()
            .map(|c| u32::from_le_bytes(*c))
            .collect()
    };
    let u64s = |b: &[u8]| -> Vec<u64> {
        b.as_chunks::<8>()
            .0
            .iter()
            .map(|c| u64::from_le_bytes(*c))
            .collect()
    };
    let f32s = |b: &[u8]| -> Vec<f32> {
        b.as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect()
    };
    let offsets = CellOffsets { offsets: u32s(slice(parts[0])?) };
    let indices = IncidenceIndices { indices: u64s(slice(parts[1])?) };
    let arena = StalkArena {
        stalk_offsets: u32s(slice(parts[2])?),
        stalks: f32s(slice(parts[7])?),
        morph_cells: u32s(slice(parts[3])?),
        morph_offsets: u32s(slice(parts[4])?),
        morphs: f32s(slice(parts[6])?),
        discrete: slice(parts[5])?.to_vec(),
    };
    let dims = slice(parts[8])?.to_vec();
    let reader = BcsfReader { dims, offsets, indices, arena, chain: None };
    if reader.len() != ncells {
        return Err(format!("{path:?}: cell count mismatch"));
    }
    reader.validate().map_err(|e| format!("{path:?}: {e}"))?;
    Ok(reader)
}

/// Header metadata of a segment file (generation, source fingerprint).
pub fn segment_meta(path: &Path) -> Result<(u64, u64), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path:?}: {e}"))?;
    if bytes.len() < 28 {
        return Err("truncated header".to_string());
    }
    if u64::from_le_bytes(bytes[..8].try_into().unwrap()) != SEGMENT_MAGIC {
        return Err("bad magic".to_string());
    }
    let generation = u64::from_le_bytes(bytes[20..28].try_into().unwrap());
    let fingerprint = u64::from_le_bytes(bytes[28..36].try_into().unwrap());
    Ok((generation, fingerprint))
}
