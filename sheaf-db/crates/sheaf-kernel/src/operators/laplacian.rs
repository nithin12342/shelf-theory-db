// crates/sheaf-kernel/src/operators/laplacian.rs — assemble sparse sheaf Laplacian
// FILE-025. Depends on FILE-024. Matrix-free L = δᵀδ over 0-cell stalks:
// unknowns are the dim-0 prefix (flat n·d), hypercells contribute operators
// only. Convention (enforced at build): every face id < n, i.e. fixtures
// store all 0-cells first. apply() needs zero allocation: stack buffers.
use super::coboundary::{CoboundaryOp, MAX_D};
use crate::simd::matvec::{matvec, matvec_transpose};

pub struct SheafLaplacian {
    pub n: usize,
    pub d: usize,
    pub ops: Vec<CoboundaryOp>,
    /// Per-cell Σ_τ M_τᵀM_τ blocks (flat n·d·d), unregularized.
    pub diag: Vec<f32>,
}

impl SheafLaplacian {
    pub fn build(n: usize, d: usize, ops: Vec<CoboundaryOp>) -> Self {
        assert!(matches!(d, 1..=MAX_D));
        let mut diag = vec![0.0f32; n * d * d];
        for op in &ops {
            assert_eq!(op.d, d, "mixed stalk dims in one Laplacian");
            for face in &op.faces {
                assert!((*face as usize) < n, "face id outside 0-cell prefix");
                let base = (*face as usize) * d * d;
                // D_v += MᵀM. NOTE: mul_add(a,b,s) already includes +s —
                // writing `s += a.mul_add(b,s)` double-accumulates (caught
                // by B7: diagonal came out 8,4,2,1 instead of 1,1,1,1).
                for r in 0..d {
                    for c in 0..d {
                        let mut s = 0.0f32;
                        for k in 0..d {
                            s = op.morph[k * d + r].mul_add(op.morph[k * d + c], s);
                        }
                        diag[base + r * d + c] += s;
                    }
                }
            }
        }
        Self { n, d, ops, diag }
    }

    /// out = L·x (overwrite contract: zeroes out first, so solver loops
    /// cannot accumulate stale values — a stale-accumulation bug here once
    /// drove CG residual to 1.14 instead of converging. The 160 KB memset
    /// is negligible next to the SpMV itself.)
    ///
    /// Parallel twin below shares the per-op body through accumulate_op so
    /// the two paths cannot silently diverge.
    pub fn apply_into(&self, x: &[f32], out: &mut [f32]) {
        debug_assert_eq!(x.len(), self.n * self.d);
        debug_assert_eq!(out.len(), self.n * self.d);
        for slot in out.iter_mut() {
            *slot = 0.0;
        }
        for op in &self.ops {
            Self::accumulate_op(self.d, op, x, out);
        }
    }

    /// One hypercell's δᵀδ contribution, added into out.
    fn accumulate_op(d: usize, op: &CoboundaryOp, x: &[f32], out: &mut [f32]) {
        let mut proj = [0.0f32; MAX_D];
        let mut delta = [0.0f32; MAX_D];
        let mut back = [0.0f32; MAX_D];
        for slot in delta[..d].iter_mut() {
            *slot = 0.0;
        }
        for (face, sign) in op.faces.iter().zip(op.signs.iter()) {
            let base = (*face as usize) * d;
            matvec(&op.morph, &x[base..base + d], d, &mut proj[..d]);
            for (slot, val) in delta[..d].iter_mut().zip(proj[..d].iter()) {
                *slot += sign * val;
            }
        }
        for (face, sign) in op.faces.iter().zip(op.signs.iter()) {
            let base = (*face as usize) * d;
            matvec_transpose(&op.morph, &delta[..d], d, &mut back[..d]);
            for (slot, val) in out[base..base + d].iter_mut().zip(back[..d].iter()) {
                *slot += sign * val;
            }
        }
    }

    /// Threaded twin of apply_into: fixed contiguous chunks, per-thread
    /// buffers, ordered reduce — deterministic for a given thread count.
    /// Used by the solver hot loop; apply_into remains the reference path.
    pub fn apply_into_par(&self, x: &[f32], out: &mut [f32], n_threads: usize) {
        debug_assert_eq!(x.len(), self.n * self.d);
        debug_assert_eq!(out.len(), self.n * self.d);
        for slot in out.iter_mut() {
            *slot = 0.0;
        }
        let d = self.d;
        let out_len = out.len();
        let chunks = self.ops.chunks((self.ops.len() + n_threads - 1) / n_threads.max(1));
        std::thread::scope(|s| {
            let mut handles = Vec::new();
            for chunk in chunks {
                handles.push(s.spawn(move || {
                    let mut local = vec![0.0f32; out_len];
                    for op in chunk {
                        Self::accumulate_op(d, op, x, &mut local);
                    }
                    local
                }));
            }
            for h in handles {
                let local = h.join().unwrap();
                for (slot, val) in out.iter_mut().zip(local.iter()) {
                    *slot += val;
                }
            }
        });
    }

    /// Restrict to the active set (cells incident to ≥1 hypercell).
    /// Isolated cells solve analytically (x = x₀ exactly: empty rows give
    /// (0+λI)x = λx₀), so excluding them is exact, not approximate. Returns
    /// the compact Laplacian plus both index maps. On sparse fixtures this
    /// removes most unknowns (measured: 10k → ~3.3k) from every O(n) pass.
    pub fn restrict_active(&self) -> (SheafLaplacian, Vec<u32>, Vec<u32>) {
        let mut active = vec![false; self.n];
        for op in &self.ops {
            for f in &op.faces {
                active[*f as usize] = true;
            }
        }
        let mut g2c = vec![u32::MAX; self.n];
        let mut c2g = Vec::new();
        for (g, on) in active.iter().enumerate() {
            if *on {
                g2c[g] = c2g.len() as u32;
                c2g.push(g as u32);
            }
        }
        let ops = self
            .ops
            .iter()
            .map(|op| {
                CoboundaryOp::new(
                    op.faces.iter().map(|f| g2c[*f as usize]).collect(),
                    op.signs.clone(),
                    op.morph.clone(),
                    op.d,
                )
            })
            .collect();
        (SheafLaplacian::build(c2g.len(), self.d, ops), g2c, c2g)
    }
    /// Block-Jacobi factors: per-cell inverse of (D_v + λI), flat n·d·d.
    /// Cholesky (D_v + λI is SPD by construction): half the flops of
    /// Gauss-Jordan with no pivot branches. Asserts definiteness.
    pub fn block_jacobi_inv(&self, lambda: f32) -> Vec<f32> {
        let (n, d) = (self.n, self.d);
        let mut inv = vec![0.0f32; n * d * d];
        // Lower-triangular factor in scratch, reused per cell (no allocs).
        let mut l = [0.0f32; MAX_D * MAX_D];
        let mut y = [0.0f32; MAX_D];
        for v in 0..n {
            let src = &self.diag[v * d * d..(v + 1) * d * d];
            // Factor A = LLᵀ.
            let mut ok = true;
            for k in 0..d {
                let mut s = src[k * d + k] + lambda;
                for j in 0..k {
                    s -= l[k * MAX_D + j] * l[k * MAX_D + j];
                }
                if s <= 1e-18 {
                    ok = false;
                    break;
                }
                l[k * MAX_D + k] = s.sqrt();
                for i in k + 1..d {
                    let mut t = src[i * d + k];
                    for j in 0..k {
                        t -= l[i * MAX_D + j] * l[k * MAX_D + j];
                    }
                    l[i * MAX_D + k] = t / l[k * MAX_D + k];
                }
            }
            assert!(ok, "non-PD diagonal block (should be impossible for D+λI)");
            // Invert column by column: solve L·y = e_j, then Lᵀ·x = y.
            for j in 0..d {
                y.iter_mut().enumerate().for_each(|(i, s)| *s = if i == j { 1.0 } else { 0.0 });
                for i in 0..d {
                    let mut s = y[i];
                    for k in 0..i {
                        s -= l[i * MAX_D + k] * y[k];
                    }
                    y[i] = s / l[i * MAX_D + i];
                }
                for i in (0..d).rev() {
                    let mut s = y[i];
                    for k in i + 1..d {
                        s -= l[k * MAX_D + i] * inv[v * d * d + k * d + j];
                    }
                    inv[v * d * d + i * d + j] = s / l[i * MAX_D + i];
                }
            }
        }
        inv
    }
}
