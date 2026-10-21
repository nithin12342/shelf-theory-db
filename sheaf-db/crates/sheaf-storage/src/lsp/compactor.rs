// crates/sheaf-storage/src/lsp/compactor.rs — merge MemComplex into BCSF
// FILE-019 (METHOD-003). Must never accept client traffic.
// Background worker: drains sealed batches, bulk-merges them with the base
// through METHOD-017 in per-source chunks (natural cache-isolation
// granularity, yield between chunks), pinned to a compaction core so reader
// L3 lines survive the merge. Publishes via install_base (Arc swap).
use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::time::Instant;

use crate::bcsf::builder::BcsfBuilder;
use super::mem_complex::MemComplex;

pub struct MergeStats {
    pub merged_cells: usize,
    pub dropped_cells: usize,
    pub ms: u128,
    pub pinned: bool,
    pub pin_core: Option<usize>,
}

pub enum Job {
    Merge { respond: mpsc::Sender<Result<MergeStats, String>> },
}

pub struct Compactor {
    tx: mpsc::Sender<Job>,
}

impl Compactor {
    pub fn spawn(mem: Arc<MemComplex>) -> (Self, std::thread::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel::<Job>();
        let h = std::thread::spawn(move || {
            // Cache isolation: pin the merge worker off the reader cores.
            let (pinned, pin_core) = match core_affinity::get_core_ids() {
                Some(ids) if !ids.is_empty() => {
                    let id = ids[ids.len() - 1].id;
                    (core_affinity::set_for_current(ids[ids.len() - 1]), Some(id))
                }
                _ => (false, None),
            };
            while let Ok(Job::Merge { respond }) = rx.recv() {
                let _ = respond.send(merge_once(&mem, pinned, pin_core));
            }
        });
        (Self { tx }, h)
    }

    pub fn merge(&self) -> Result<MergeStats, String> {
        let (tx, rx) = mpsc::channel();
        self.tx.send(Job::Merge { respond: tx }).map_err(|e| e.to_string())?;
        rx.recv().map_err(|e| e.to_string())?
    }
}

fn merge_once(mem: &MemComplex, pinned: bool, pin_core: Option<usize>) -> Result<MergeStats, String> {
    let t0 = Instant::now();
    // Pin the read set first: everything sealed at snapshot time is merged;
    // anything sealed later waits for the next merge (LSM fencing).
    let snap = mem.snapshot();
    let ids: Vec<(usize, u64)> = snap.batches.iter().map(|b| (b.shard, b.id)).collect();
    let drained = mem.drain_batches(&ids);

    // New layout: live base cells in order, then live batch cells in order.
    // B4: liveness comes from the versioned tombstones AS OF the snapshot's
    // generation (no copied bitmap anywhere).
    let mut remap_base = vec![u32::MAX; snap.base.len()];
    let mut next: u32 = 0;
    for (i, slot) in remap_base.iter_mut().enumerate() {
        if snap.base_alive(i) {
            *slot = next;
            next += 1;
        }
    }
    let mut dropped = (0..snap.base.len()).filter(|i| !snap.base_alive(*i)).count();
    let mut builder = BcsfBuilder::new();
    // Chunked base merge: 32k-cell quanta with a yield between them so the
    // merge thread never holds a 50 MB memcpy burst against reader cores.
    // Batches stay one extend call each (already ~1k-cell quanta).
    const CHUNK: usize = 32_768;
    let mut c = 0;
    while c < snap.base.len() {
        let e = (c + CHUNK).min(snap.base.len());
        // B4: the remap table already encodes liveness (u32::MAX = dropped),
        // so the merge copies no boolean mask at all.
        builder.extend_range_remapped(&snap.base, &remap_base, c, e);
        c = e;
        std::thread::yield_now();
    }

    let mut translations: HashMap<(usize, u64), Vec<u32>> = HashMap::new();
    for (sh, b) in &drained {
        let view = snap.batches.iter().find(|v| v.shard == *sh && v.id == b.id).ok_or("drained batch missing from snapshot")?;
        // Phase-2 scope: batch cells reference base ids only (two-level merge).
        for (l, id) in b.reader.indices.indices.iter().enumerate() {
            if (*id as usize) >= snap.base.len() {
                return Err(format!("nested hypercell boundary at batch cell {} (Phase-2 scope)", l));
            }
        }
        // B4: batch liveness from the versioned tombstones as of the
        // snapshot's generation; the translation table doubles as the
        // liveness map (u32::MAX = dropped).
        let mut trans = vec![u32::MAX; b.reader.len()];
        for (l, slot) in trans.iter_mut().enumerate() {
            if snap.batch_alive(view, l) {
                *slot = next;
                next += 1;
            } else {
                dropped += 1;
            }
        }
        // The batch cell's BOUNDARY refers to base cells -> id map = remap_base.
        builder.extend_from_reader_remapped_with(&b.reader, &trans, &remap_base);
        translations.insert((*sh, b.id), trans);
        std::thread::yield_now();
    }
    let new_base = builder.finish();
    new_base.validate().map_err(|e| format!("merged base invalid: {}", e))?;
    let live_flags = vec![true; new_base.len()];
    let merged_cells = new_base.len();
    mem.install_base(new_base, live_flags, &ids, translations)?;
    Ok(MergeStats { merged_cells, dropped_cells: dropped, ms: t0.elapsed().as_millis(), pinned, pin_core })
}
