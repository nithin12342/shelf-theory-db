// crates/sheaf-storage/src/bcsf/reader.rs — serve zero-copy snapshot views
// FILE-015 (METHOD-001). Must never write or lock.
// V3: boundary() carries a debug_assert sorted check (zero-cost release).
use super::arena::StalkArena;
use super::indices::IncidenceIndices;
use super::offsets::CellOffsets;

/// Immutable BCSF view: dims + offsets + indices + arena. Zero heap
/// indirection in the read path (slice indexing only).
#[derive(Debug)]
pub struct BcsfReader {
    pub dims: Vec<u8>,
    pub offsets: CellOffsets,
    pub indices: IncidenceIndices,
    pub arena: StalkArena,
}

impl BcsfReader {
    pub fn len(&self) -> usize {
        self.dims.len()
    }
    pub fn is_empty(&self) -> bool {
        self.dims.is_empty()
    }
    pub fn boundary(&self, cell: usize) -> &[u64] {
        let (s, e) = self.offsets.range(cell);
        let sl = self.indices.slice(s, e);
        debug_assert!(
            sl.windows(2).all(|w| w[0] <= w[1]),
            "BCSF boundary slice unsorted"
        );
        sl
    }
    pub fn stalk(&self, cell: usize) -> &[f32] {
        self.arena.stalk(cell)
    }
    pub fn morph_weights(&self, cell: usize) -> &[f32] {
        self.arena.morph(cell)
    }
    /// V1: discrete exact payload for a cell.
    pub fn discrete(&self, cell: usize) -> &[u8] {
        self.arena.discrete(cell)
    }
    /// Raw payload bytes counted for the <1.5x gate (V1: discrete included).
    pub fn raw_payload_bytes(&self) -> usize {
        self.indices.indices.len() * 8
            + self.arena.stalks.len() * 4
            + self.arena.morphs.len() * 4
            + self.arena.discrete.len()
    }
    /// Total resident bytes of the hot-path arrays.
    pub fn resident_bytes(&self) -> usize {
        self.raw_payload_bytes()
            + self.offsets.offsets.len() * 4
            + self.dims.len()
            + self.arena.stalk_offsets.len() * 4
            + self.arena.morph_cells.len() * 4
            + self.arena.morph_offsets.len() * 4
    }
    /// Capacity-based resident bytes (true allocation incl. Vec slack).
    /// Equals resident_bytes() after the builder freeze step; a gap here
    /// means growth slack is hiding real memory from the gate.
    pub fn resident_capacity_bytes(&self) -> usize {
        self.indices.indices.capacity() * 8
            + self.arena.stalks.capacity() * 4
            + self.arena.morphs.capacity() * 4
            + self.arena.discrete.capacity()
            + self.offsets.offsets.capacity() * 4
            + self.dims.capacity()
            + self.arena.stalk_offsets.capacity() * 4
            + self.arena.morph_cells.capacity() * 4
            + self.arena.morph_offsets.capacity() * 4
    }
    /// Structural validation: runs in release, called by the E2E bench.
    /// Closes dangling-boundary, misaligned-slab, and unsorted-morph classes.
    pub fn validate(&self) -> Result<(), String> {
        let n = self.dims.len();
        if self.offsets.offsets.len() != n + 1 {
            return Err("offsets length != ncells+1".to_string());
        }
        if self.arena.stalk_offsets.len() != n + 1 {
            return Err("stalk_offsets length != ncells+1".to_string());
        }
        if self.arena.discrete.len() != n * 16 {
            return Err("discrete slab != ncells*16".to_string());
        }
        if self.arena.morph_offsets.len() != self.arena.morph_cells.len() + 1 {
            return Err("morph_offsets length != morph_cells+1".to_string());
        }
        if !self.arena.morph_cells.windows(2).all(|w| w[0] < w[1]) {
            return Err("morph_cells not strictly sorted".to_string());
        }
        if self.arena.morph_cells.iter().any(|&c| (c as usize) >= n) {
            return Err("morph cell out of range".to_string());
        }
        let mut prev = 0usize;
        for (i, w) in self.offsets.offsets.iter().enumerate() {
            let v = *w as usize;
            if v < prev || v > self.indices.indices.len() {
                return Err(format!("offsets not monotonic at cell {}", i));
            }
            prev = v;
        }
        for (i, &id) in self.indices.indices.iter().enumerate() {
            if (id as usize) >= n {
                return Err(format!("dangling boundary id at index {}", i));
            }
        }
        Ok(())
    }
}
