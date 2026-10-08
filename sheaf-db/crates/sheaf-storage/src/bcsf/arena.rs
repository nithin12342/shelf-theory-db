// crates/sheaf-storage/src/bcsf/arena.rs — pack stalks plus morphism weights
// FILE-014. Must never parse or plan.
// Holds continuous stalk vectors (x), discrete exact payloads (V1: fixed
// 16 B per cell, no offset table needed), AND morphism weights (W) in flat
// slabs. Morphs use a sparse index (V6): only cells with non-empty morphs
// get an entry, so 1M empty morphs cost zero instead of 4.8 MB of u32s.
#[derive(Debug)]
pub struct StalkArena {
    pub stalk_offsets: Vec<u32>,
    pub stalks: Vec<f32>,
    /// Sorted cell indices that own a non-empty morph (index order by build).
    pub morph_cells: Vec<u32>,
    pub morph_offsets: Vec<u32>,
    pub morphs: Vec<f32>,
    /// Fixed 16-byte discrete payload per cell: discrete[cell*16..cell*16+16].
    pub discrete: Vec<u8>,
}

impl StalkArena {
    pub fn stalk(&self, cell: usize) -> &[f32] {
        let s = self.stalk_offsets[cell] as usize;
        let e = self.stalk_offsets[cell + 1] as usize;
        &self.stalks[s..e]
    }
    pub fn morph(&self, cell: usize) -> &[f32] {
        let key = u32::try_from(cell).expect("BCSF cell index overflow u32");
        match self.morph_cells.binary_search(&key) {
            Ok(pos) => {
                let s = self.morph_offsets[pos] as usize;
                let e = self.morph_offsets[pos + 1] as usize;
                &self.morphs[s..e]
            }
            Err(_) => &[],
        }
    }
    pub fn discrete(&self, cell: usize) -> &[u8] {
        &self.discrete[cell * 16..cell * 16 + 16]
    }
}
