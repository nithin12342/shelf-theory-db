// crates/sheaf-engine/src/bin/phase6_adv.rs — Phase-6 adversarial suite
// D1: Empty reader → zero domains.
// D2: Single cell → one domain, no ghosts.
// D3: All cells fit in one domain → single domain.
// D4: Ghost replicas cover cross-domain boundary refs.
// D5: No overlapping domain ownership.
// D6: Domain byte budget respected.
use sheaf_storage::bcsf::builder::BcsfBuilder;
use sheaf_storage::bcsf::partition::{domains_fit_l3, ghosts_cover_boundaries, partition_reader};
use sheaf_storage::bcsf::reader::BcsfReader;

fn main() {
    let mut all_pass = true;

    // D1: Empty reader → zero domains.
    let builder = BcsfBuilder::new();
    let reader: BcsfReader = builder.finish();
    let p = partition_reader(&reader, 64 * 1024 * 1024);
    let d1 = p.domains.is_empty();
    println!("D1_empty_zero_domains={}", if d1 { "PASS" } else { "FAIL" });
    all_pass &= d1;

    // D2: Single cell → one domain, no ghosts.
    let mut b = BcsfBuilder::new();
    let db = [0u8; 16];
    b.push(0, vec![], &db, &[1.0, 2.0, 3.0, 4.0], &[]);
    let r: BcsfReader = b.finish();
    let p = partition_reader(&r, 64 * 1024 * 1024);
    let d2 = p.domains.len() == 1 && p.domains[0].ghosts.is_empty();
    println!("D2_single_cell={}", if d2 { "PASS" } else { "FAIL" });
    all_pass &= d2;

    // D3: Small reader → single domain (all fit in 64MB).
    let mut b = BcsfBuilder::new();
    for _ in 0..100 {
        let db = [0u8; 16];
        b.push(0, vec![], &db, &[1.0, 2.0, 3.0, 4.0], &[]);
    }
    let r: BcsfReader = b.finish();
    let p = partition_reader(&r, 64 * 1024 * 1024);
    let d3 = p.domains.len() == 1;
    println!("D3_small_single_domain={}", if d3 { "PASS" } else { "FAIL" });
    all_pass &= d3;

    // D4: Ghost replicas cover cross-domain refs (1.5M cells).
    let mut b = BcsfBuilder::new();
    for i in 0..1_200_000u64 {
        let mut db = [0u8; 16];
        db[..8].copy_from_slice(&101u64.to_le_bytes());
        db[8..16].copy_from_slice(&i.to_le_bytes());
        b.push(0, vec![], &db, &[0.001 * i as f32, -0.002, 0.003, 0.004], &[]);
    }
    for h in 0..300_000u64 {
        let mut db = [0u8; 16];
        db[..8].copy_from_slice(&102u64.to_le_bytes());
        db[8..16].copy_from_slice(&h.to_le_bytes());
        let b0 = h * 3;
        let b1 = h * 3 + 1;
        let b2 = h * 3 + 2;
        b.push(2, vec![b0, b1, b2], &db, &[0.001 * h as f32, -0.002, 0.003, 0.004], &[]);
    }
    let r: BcsfReader = b.finish();
    let p = partition_reader(&r, 64 * 1024 * 1024);
    let d4 = ghosts_cover_boundaries(&r, &p);
    println!("D4_ghosts_cover={}", if d4 { "PASS" } else { "FAIL" });
    all_pass &= d4;

    // D5: No overlapping ownership.
    let mut owner = vec![0usize; r.len()];
    for d in &p.domains {
        for c in d.cell_range.clone() {
            owner[c] += 1;
        }
    }
    let d5 = owner.iter().all(|&c| c == 1);
    println!("D5_no_overlap={}", if d5 { "PASS" } else { "FAIL" });
    all_pass &= d5;

    // D6: Domain byte budget respected.
    let d6 = domains_fit_l3(&p, 64 * 1024 * 1024);
    println!("D6_budget_respected={}", if d6 { "PASS" } else { "FAIL" });
    all_pass &= d6;

    if !all_pass {
        std::process::exit(1);
    }
    println!("ADVERSARIAL6: PASS");
}
