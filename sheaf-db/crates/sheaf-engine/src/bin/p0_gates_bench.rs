// crates/sheaf-engine/src/bin/p0_gates_bench.rs — P0 borrows gate
// Three review-driven additions, one binary, three gates:
// (1) Chebyshev Tikhonov filter vs BJ-PCG reference (agree < 1e-3, f32);
// (2) H¹-proxy inconsistency E/‖x‖² strictly drops after harmonic solve;
// (3) b₀ union-find == 3 on a two-component + isolated fixture.
// Fixture: path graph n=2000, d=2, identity morphs (SPD, deterministic).
use std::time::Instant;

use sheaf_kernel::components::betti_zero;
use sheaf_kernel::operators::coboundary::CoboundaryOp;
use sheaf_kernel::operators::energy::{dirichlet_energy, inconsistency};
use sheaf_kernel::operators::laplacian::SheafLaplacian;
use sheaf_kernel::solvers::cg::CgLimits;
use sheaf_kernel::solvers::chebyshev::{chebyshev_tikhonov, lambda_max_power};
use sheaf_kernel::solvers::tikhonov::regularized_solve_monitored;

const N: usize = 2000;
const D: usize = 2;
const MU: f32 = 0.1;
const DEGREE: usize = 40;

fn main() {
    // ---- Fixture: path-graph sheaf, identity restriction maps ----
    let mut ops = Vec::new();
    for i in 0..N - 1 {
        ops.push(CoboundaryOp::new(
            vec![i as u32, (i + 1) as u32],
            vec![1.0, -1.0],
            vec![1.0, 0.0, 0.0, 1.0],
            D,
        ));
    }
    let lap = SheafLaplacian::build(N, D, ops);
    // Smooth signal + deterministic noise (same seed convention as benches).
    let mut x0 = vec![0.0f32; N * D];
    for i in 0..N {
        for k in 0..D {
            let s = ((i * (k + 3) + 7) % 101) as f32 / 101.0 - 0.5;
            x0[i * D + k] = ((i as f32) / (N as f32) * std::f32::consts::TAU).sin() + 0.3 * s;
        }
    }
    let mut b = vec![0.0f32; N * D];
    for (dst, src) in b.iter_mut().zip(x0.iter()) {
        *dst = MU * src;
    }

    // ---- Gate 1: Chebyshev vs PCG reference ----
    let t0 = Instant::now();
    let mut x_pcg = vec![0.0f32; N * D];
    let res = regularized_solve_monitored(
        &lap, &x0, &mut x_pcg, MU, CgLimits { tol: 1e-6, max_iter: 500 }, None, 1,
    );
    let pcg_ms = t0.elapsed().as_secs_f64() * 1e3;
    let t0 = Instant::now();
    let lmax = lambda_max_power(&lap, 25);
    let mut x_cheb = vec![0.0f32; N * D];
    let used = chebyshev_tikhonov(&lap, &b, MU, lmax, DEGREE, &mut x_cheb);
    let cheb_ms = t0.elapsed().as_secs_f64() * 1e3;
    let mut num = 0.0f64;
    let mut den = 0.0f64;
    for (a, c) in x_pcg.iter().zip(x_cheb.iter()) {
        num += ((*a as f64) - (*c as f64)).powi(2);
        den += (*a as f64).powi(2);
    }
    let rel = (num / den).sqrt();
    println!("p0_cheb: pcg_iters={} pcg_ms={:.1} cheb_applies={} cheb_ms={:.1} lmax={:.3} rel={:.2e}",
        res.iters, pcg_ms, 25 + 1 + used, cheb_ms, lmax, rel);
    println!("gate_cheb_agree={}", if rel < 1e-3 { "PASS" } else { "FAIL" });

    // ---- Gate 2: inconsistency strictly drops ----
    let norm_sq: f64 = x0.iter().map(|v| (*v as f64).powi(2)).sum();
    let before = inconsistency(dirichlet_energy(&lap.ops, &x0), norm_sq);
    let norm_h: f64 = x_pcg.iter().map(|v| (*v as f64).powi(2)).sum();
    let after = inconsistency(dirichlet_energy(&lap.ops, &x_pcg), norm_h);
    println!("p0_incon: before={:.4e} after={:.4e}", before, after);
    println!("gate_incon_drop={}", if after < before { "PASS" } else { "FAIL" });

    // ---- Gate 3: b₀ == 3 (two components + one isolated cell) ----
    let faces = vec![vec![0u32, 1, 2], vec![3u32, 4]];
    let b0 = betti_zero(6, &faces);
    println!("p0_betti0: b0={} (expect 3)", b0);
    println!("gate_betti0={}", if b0 == 3 { "PASS" } else { "FAIL" });
    if rel >= 1e-3 || after >= before || b0 != 3 {
        std::process::exit(1);
    }
    println!("P0-GATES: ALL PASS");
}
