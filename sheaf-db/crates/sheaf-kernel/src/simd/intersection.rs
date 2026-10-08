// crates/sheaf-kernel/src/simd/intersection.rs — intersect u64 slices via SIMD
// FILE-022. Pure slice arithmetic. Must never know TQL syntax or storage types.
// Scalar two-pointer baseline; AVX-512 path plugs in here without signature change.
pub fn intersect_count(a: &[u64], b: &[u64]) -> usize {
    let (mut i, mut j, mut n) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            n += 1;
            i += 1;
            j += 1;
        } else if a[i] < b[j] {
            i += 1;
        } else {
            j += 1;
        }
    }
    n
}
