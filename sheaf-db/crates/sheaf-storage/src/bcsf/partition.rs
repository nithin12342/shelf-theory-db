// crates/sheaf-storage/src/bcsf/partition.rs — split hypergraph into L3-sized domains
// FILE-021 (METHOD-015). Depends on FILE-015 (reader).
// Partitions cells into contiguous domains of ~64MB (L3 cache size) for
// cache-efficient local solves. Ghost replicas serve cross-socket boundary
// reads locally instead of fetching remote memory.
use super::reader::BcsfReader;

/// A partition domain: contiguous cell range + ghost replicas of boundary cells.
pub struct Domain {
    pub id: usize,
    pub cell_range: std::ops::Range<usize>,
    /// Ghost replicas: (local_cell_id, remote_cell_id) pairs.
    pub ghosts: Vec<(usize, usize)>,
}

/// A partitioned hypergraph with L3-sized domains.
pub struct Partition {
    pub domains: Vec<Domain>,
    pub domain_bytes: Vec<usize>,
}

/// Partition a BCSF reader into domains of at most `target_bytes` each.
/// Uses a greedy contiguous split: accumulate cells until the domain would
/// exceed the target, then start a new domain. Ghost replicas are cells
/// referenced by a domain's boundary but owned by another domain.
pub fn partition_reader(reader: &BcsfReader, target_bytes: usize) -> Partition {
    let n = reader.len();
    if n == 0 {
        return Partition { domains: Vec::new(), domain_bytes: Vec::new() };
    }
    // Estimate per-cell bytes from the reader.
    let total_bytes = reader.resident_bytes();
    let avg_cell_bytes = total_bytes / n;
    let cells_per_domain = (target_bytes / avg_cell_bytes.max(1)).max(1);
    let mut domains: Vec<Domain> = Vec::new();
    let mut domain_bytes: Vec<usize> = Vec::new();
    let mut start = 0usize;
    let mut domain_id = 0usize;
    while start < n {
        let end = (start + cells_per_domain).min(n);
        // Find ghost replicas: boundary cells of this domain that belong
        // to other domains.
        let mut ghosts: Vec<(usize, usize)> = Vec::new();
        for cell in start..end {
            for &b in reader.boundary(cell) {
                let b = b as usize;
                if b < start || b >= end {
                    // Ghost: local alias for remote cell.
                    let local_alias = end + ghosts.len();
                    ghosts.push((local_alias, b));
                }
            }
        }
        domains.push(Domain {
            id: domain_id,
            cell_range: start..end,
            ghosts,
        });
        domain_bytes.push((end - start) * avg_cell_bytes);
        start = end;
        domain_id += 1;
    }
    Partition { domains, domain_bytes }
}

/// Check that all domains are within the target byte budget.
pub fn domains_fit_l3(partition: &Partition, target_bytes: usize) -> bool {
    partition.domain_bytes.iter().all(|&b| b <= target_bytes)
}

/// Check that every boundary reference resolves to either a local cell or
/// a ghost replica (no dangling cross-domain references).
pub fn ghosts_cover_boundaries(reader: &BcsfReader, partition: &Partition) -> bool {
    for domain in &partition.domains {
        for cell in domain.cell_range.clone() {
            for &b in reader.boundary(cell) {
                let b = b as usize;
                let is_local = domain.cell_range.contains(&b);
                let is_ghost = domain.ghosts.iter().any(|(_, remote)| *remote == b);
                if !is_local && !is_ghost {
                    return false;
                }
            }
        }
    }
    true
}

/// MAX-PERF P6: ghost membership as a dense bitset over the global cell id
/// space. O(1) branchless membership test vs linear scan of the ghost vec.
/// Space: n_cells bits per domain (vs 16 B per ghost pair).
pub fn ghost_bitmask(domain: &Domain, n_cells: usize) -> Vec<u64> {
    let mut mask = vec![0u64; n_cells.div_ceil(64)];
    for &(_, remote) in &domain.ghosts {
        if remote < n_cells {
            mask[remote / 64] |= 1u64 << (remote % 64);
        }
    }
    mask
}

/// Branchless ghost test: 1 bit, no pointer chase.
pub fn is_ghost(mask: &[u64], cell: usize) -> bool {
    match mask.get(cell / 64) {
        Some(w) => (w >> (cell % 64)) & 1 == 1,
        None => false,
    }
}
