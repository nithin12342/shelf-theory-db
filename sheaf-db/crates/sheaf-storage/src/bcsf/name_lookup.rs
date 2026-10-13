// crates/sheaf-storage/src/bcsf/name_lookup.rs — find cells by discrete field value
// FILE-020 (METHOD-011). Depends on FILE-015 (reader).
// Given a discrete field name and target value, return the contiguous slice
// of matching cell indices. Parity with full scan is verified by E2E.
// The discrete payload is a fixed 16-byte slab per cell; field extraction
// uses a simple columnar layout defined by the schema.
use super::reader::BcsfReader;

/// A name lookup index over the discrete payload slab.
/// Built once per snapshot; serves point queries without full scan.
pub struct NameIndex {
    /// Field name -> column offset in the 16-byte discrete slab.
    fields: Vec<(String, usize)>,
    /// For each field, a sorted map from value bytes to contiguous cell range.
    /// Value bytes are the raw little-endian encoding of the field.
    index: Vec<Vec<(Vec<u8>, std::ops::Range<usize>)>>,
    /// Sorted cell indices per field for binary search.
    sorted_cells: Vec<Vec<u32>>,
}

impl NameIndex {
    /// Build a name index from a reader + schema field definitions.
    /// `fields` is [(field_name, offset_in_discrete_slab)] in schema order.
    pub fn build(
        reader: &BcsfReader,
        fields: &[(String, usize)],
        cell_classes: &[(u64, String)], // (class_id, type_name) pairs
    ) -> Self {
        let n = reader.len();
        let mut field_names: Vec<String> = Vec::new();
        let mut offsets: Vec<usize> = Vec::new();
        for (name, off) in fields {
            field_names.push(name.clone());
            offsets.push(*off);
        }
        let mut sorted_cells: Vec<Vec<u32>> = vec![Vec::new(); fields.len()];
        let mut index: Vec<Vec<(Vec<u8>, std::ops::Range<usize>)>> =
            vec![Vec::new(); fields.len()];
        // Group cells by class, then by field value.
        for fid in 0..offsets.len() {
            let mut cells: Vec<u32> = Vec::with_capacity(n);
            for cell in 0..n {
                let class_bytes = &reader.discrete(cell)[..8];
                let class_id = u64::from_le_bytes(class_bytes.try_into().unwrap());
                // Only index cells whose class matches a known type.
                if cell_classes.iter().any(|(cid, _)| *cid == class_id) {
                    cells.push(cell as u32);
                }
            }
            // Sort by field value for binary search.
            cells.sort_by(|&a, &b| {
                let va = &reader.discrete(a as usize)[offsets[fid]..offsets[fid] + 8];
                let vb = &reader.discrete(b as usize)[offsets[fid]..offsets[fid] + 8];
                va.cmp(vb)
            });
            // Build value -> contiguous range map before moving cells.
            let mut i = 0;
            while i < cells.len() {
                let val = reader.discrete(cells[i] as usize)[offsets[fid]..offsets[fid] + 8].to_vec();
                let start = i;
                while i < cells.len()
                    && reader.discrete(cells[i] as usize)[offsets[fid]..offsets[fid] + 8] == val[..]
                {
                    i += 1;
                }
                index[fid].push((val, start..i));
            }
            sorted_cells[fid] = cells;
        }
        Self {
            fields: field_names.into_iter().zip(offsets).collect(),
            index,
            sorted_cells,
        }
    }

    /// Lookup cells by field name + value. Returns contiguous slice of cell indices.
    pub fn lookup(&self, field: &str, value: &[u8]) -> Option<&std::ops::Range<usize>> {
        let fid = self.fields.iter().position(|(n, _)| n == field)?;
        let idx = &self.index[fid];
        // Binary search on value bytes.
        let pos = idx.binary_search_by(|(v, _)| v.as_slice().cmp(value)).ok()?;
        Some(&idx[pos].1)
    }

    /// Get all cell indices for a field (sorted by value).
    pub fn field_cells(&self, field: &str) -> Option<&[u32]> {
        let fid = self.fields.iter().position(|(n, _)| n == field)?;
        Some(&self.sorted_cells[fid])
    }
}

/// Extract a u64 field value from a cell's discrete slab at the given offset.
pub fn extract_u64(reader: &BcsfReader, cell: usize, offset: usize) -> u64 {
    let bytes = &reader.discrete(cell)[offset..offset + 8];
    u64::from_le_bytes(bytes.try_into().unwrap())
}

/// Extract a bool field value from a cell's discrete slab at the given offset.
pub fn extract_bool(reader: &BcsfReader, cell: usize, offset: usize) -> bool {
    reader.discrete(cell)[offset] != 0
}
