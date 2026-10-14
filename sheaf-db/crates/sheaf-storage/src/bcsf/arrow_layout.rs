// crates/sheaf-storage/src/bcsf/arrow_layout.rs — Arrow-compatible columnar stalk slabs
// SYNTHESIS Stage 1a (Choice 2 substrate). Depends on nothing.
// Layout mirrors Apache Arrow `FixedSizeListArray<Float32, d>`: one flat
// values buffer (64-byte aligned start, rows of width d back-to-back) plus
// a validity bitmap (1 bit per row). No arrow-crate dependency — the layout
// is the interop contract; C Data Interface export is a follow-up, not here.
// Why manual: the kernel/storage stay dependency-free per project precedent;
// alignment + validity semantics are what the FMA kernels (Stage 1b) need.
use std::alloc::{alloc_zeroed, dealloc, Layout};

const ALIGN: usize = 64;

/// A 64-byte aligned, length-tracked f32 buffer. Frees on drop.
/// `pub(crate)` for reuse by the coordinate store (M1) — same alignment contract.
pub(crate) struct AlignedBuf {
    ptr: *mut f32,
    cap: usize, // elements
}

impl AlignedBuf {
    pub(crate) fn new(cap_elems: usize) -> Self {
        let layout = Layout::from_size_align(
            cap_elems.checked_mul(4).expect("slab overflow"),
            ALIGN,
        )
        .expect("align layout");
        let ptr = unsafe { alloc_zeroed(layout) as *mut f32 };
        assert!(!ptr.is_null(), "slab alloc failed");
        assert!((ptr as usize).is_multiple_of(ALIGN), "slab misaligned");
        Self { ptr, cap: cap_elems }
    }

    pub(crate) fn as_slice(&self, len: usize) -> &[f32] {
        assert!(len <= self.cap);
        unsafe { std::slice::from_raw_parts(self.ptr, len) }
    }

    pub(crate) fn as_mut_slice(&mut self, len: usize) -> &mut [f32] {
        assert!(len <= self.cap);
        unsafe { std::slice::from_raw_parts_mut(self.ptr, len) }
    }
}

impl Drop for AlignedBuf {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.cap * 4, ALIGN).expect("align layout");
        unsafe { dealloc(self.ptr as *mut u8, layout) };
    }
}

/// Graded columnar stalk store: rows of width `d`, Arrow-compatible layout.
pub struct ArrowStalkSlab {
    pub d: usize,
    len: usize,
    values: AlignedBuf,
    validity: Vec<u64>,
}

impl ArrowStalkSlab {
    pub fn with_capacity(d: usize, rows: usize) -> Self {
        assert!(d >= 1, "width must be >= 1");
        Self {
            d,
            len: 0,
            values: AlignedBuf::new(rows * d),
            validity: vec![0u64; rows.div_ceil(64)],
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Base pointer (for the 64B alignment gate).
    pub fn base_addr(&self) -> usize {
        self.values.ptr as usize
    }

    pub fn push_row(&mut self, row: &[f32]) {
        assert_eq!(row.len(), self.d, "row width mismatch");
        let dst = self.values.as_mut_slice((self.len + 1) * self.d);
        dst[self.len * self.d..(self.len + 1) * self.d].copy_from_slice(row);
        self.validity[self.len / 64] |= 1u64 << (self.len % 64);
        self.len += 1;
    }

    pub fn set_null(&mut self, row: usize) {
        assert!(row < self.len);
        self.validity[row / 64] &= !(1u64 << (row % 64));
    }

    pub fn is_valid(&self, row: usize) -> bool {
        assert!(row < self.len);
        (self.validity[row / 64] >> (row % 64)) & 1 == 1
    }

    /// Scalar column sum over valid rows (Stage 1a baseline; FMA kernel
    /// arrives in Stage 1b and must beat this number on identical data).
    pub fn col_sum(&self, col: usize) -> f64 {
        assert!(col < self.d);
        let v = self.values.as_slice(self.len * self.d);
        let mut acc = 0f64;
        for r in 0..self.len {
            if self.is_valid(r) {
                acc += v[r * self.d + col] as f64;
            }
        }
        acc
    }

    /// Fused sum(price * (1 - discount)) over valid rows with price col `p`
    /// and discount col `q`, in row order (matches tpch_scan_bench exactly).
    pub fn fused_revenue_sum(&self, p: usize, q: usize) -> f64 {
        assert!(p < self.d && q < self.d);
        let v = self.values.as_slice(self.len * self.d);
        let mut acc = 0f64;
        for r in 0..self.len {
            if self.is_valid(r) {
                acc += v[r * self.d + p] as f64 * (1.0 - v[r * self.d + q] as f64);
            }
        }
        acc
    }

    /// Transpose two columns into separate contiguous (SoA) vectors.
    /// Unit-stride loads vectorize cleanly; the gather path cannot.
    /// Cost: one extra pass + 2× column memory (recorded, not hidden).
    pub fn transpose_soa(&self, p: usize, q: usize) -> (Vec<f32>, Vec<f32>) {
        assert!(p < self.d && q < self.d);
        let v = self.values.as_slice(self.len * self.d);
        let mut cp = Vec::with_capacity(self.len);
        let mut cq = Vec::with_capacity(self.len);
        for r in 0..self.len {
            if self.is_valid(r) {
                cp.push(v[r * self.d + p]);
                cq.push(v[r * self.d + q]);
            }
        }
        (cp, cq)
    }
}
