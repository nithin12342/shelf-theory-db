# Von Neumann in silicon — the equipment, the hierarchy, and the physical limits

**Node:** REQ-057 / SPEC-057 / METHOD-052 · **Probe:** `crates/sheaf-engine/src/bin/physical_limits_probe.rs`
**Host:** i7-1165G7 (4C/8T, Tiger Lake, 12 MiB L3) · 16 GiB Samsung DDR4-3200 (1 channel) · 512 GB Kingston NVMe (PCIe 4.0 x4) · Win11
**Evidence:** `tests/e2e/von_neumann/expected/physical_limits.txt` (gate) · `tests/e2e/realtime_bench/results/pl_run{1,2,3}.txt` (3 runs)

**Rule for this document:** every number is either (a) **measured on this host** by the
probe above, or (b) **derived here from a published physical constant**, with the derivation
shown. No number is copied from a vendor datasheet and presented as ours. Where a published
spec is quoted, it is labelled `published` and sourced.

---

## 0. Answer in one table

Rows = the physical limit. Columns = the derived floor, what this host measures, and the gap.

| # | Limit | Derived floor | This host | Gap | Verdict |
|---|---|---|---|---|---|
| L1 | Landauer energy per bit erased | 2.87e-21 J (kT·ln2, 300 K) | ~1e-15 J (CMOS) | 3.5e5× | Not the binding constraint |
| L2 | Signal flight across a die | 120 ps / 10 mm one-way | — | — | Bounds die size, not clock |
| L3 | DRAM random access (DDR4-3200 CL22) | 41.2 ns (tRCD+tCL+tRP) | 92.6 ns | 2.24× | Real access ≫ bare timing |
| L4 | Page walk (4 KiB pages) | 46.6 ns measured | — | — | Dominates cold random access |
| L5 | DRAM read bandwidth (1 channel) | 25.6 GB/s | 11.5 GB/s (1T) | 45% | Bandwidth-saturated |
| L6 | Storage sequential read | 7.88 GB/s (PCIe 4.0 x4) | 3.20 GB/s | 40.6% | Bus-limited |
| L7 | Durable write (fsync) | 1.31 ms | 1.31 ms | 1.00× | **Hard floor, confirmed** |
| L8 | Machine balance (ridge point) | — | 2.92 FLOP/byte | — | Kernel is 6-9× below it |
| L9 | HDD areal density (10-yr retention) | 1.20 Tb/in² | — | — | Thermal stability, not area |
| L10 | Storage/DRAM latency ratio | — | 33.0× | — | Justifies pre-aggregation |
| L11 | **Database scan vs memory ceiling** | — | **42-81% of ceiling** | — | **Gap is the accumulator, not memory** |

### 0.1 Does the database reach the memory limit? Short answer

**No, not at one thread. The database scan reaches 42% of the memory ceiling at one
thread and 81% at four threads. The missing time is not the memory system. It is a
serial accumulator inside the kernel.**

The detail is in section 2.4a. The short version: the partition scan keeps one
floating-point accumulator per thread, so every row waits for the previous
row's multiply-add to finish. The memory system idles while the processor
waits. A four-lane version of the same arithmetic runs 1.47x faster on the
same bytes.

---

## 1. The machine parts, and where each one lives

The von Neumann model (1945) has five organs. Every modern computer still has all five;
what changed is that the *memory* organ became a hierarchy and the *processor* organ became
a choice among four different designs.

| Von Neumann organ (1945) | Modern equivalent | This host | Where it is in sheaf-db |
|---|---|---|---|
| Central arithmetic unit | ALU + FMA units | AVX-512 FMA, 2/cycle | `simd/fma_scan.rs` matvec |
| Central control unit | Front end + branch predictor | 8-wide OoO | `ir/optimizer.rs` plan |
| Memory (instructions + data) | L1/L2/L3 SRAM → DRAM | 32 KiB / 5 MiB / 12 MiB → 16 GiB | BCSF slabs |
| Outside recording medium | NVMe SSD | 512 GB, PCIe 4.0 x4 | WAL frames, fiber bundle |
| I/O mechanisms | DMA + PCIe + OS page cache | — | `lsp/wal.rs`, bundle cache |

### 1.1 Primary storage — the ladder, measured

**Rows = memory tier. Columns = what was measured and how.**

| Tier | Technology | Latency (ns) | Latency (cycles) | Capacity | Bandwidth | Cost/byte |
|---|---|---|---|---|---|---|
| Tier | Technology | Latency (ns) | Latency (cycles) | Capacity | Bandwidth | Cost/byte |
|---|---|---|---|---|---|---|
| Registers | flip-flop | ~0.3 (1 cycle) | 1 | 32 × 64-bit | 2×64 B/cycle | astronomic |
| L1d/L1i | 6T SRAM | **1.64** | **4.6** | 48 KiB/core | 2×64 B/cycle | high |
| L2 | 6T SRAM | **4.27** | **13.7** | 1.25 MiB/core | ~64 B/cycle | high |
| L2 (far) | 6T SRAM | 7.33 | 20.8 | 1.25 MiB/core | ~64 B/cycle | high |
| L3 | 6T SRAM, shared | **21.97** | **62.6** | 12 MiB | ~30 B/cycle | med |
| DRAM | 1T1C DRAM, DDR4-3200 | **163.7** | 478 | 16 GiB | **11.5 GB/s** (1T) | low |
| NVMe | 3D NAND + DRAM cache | **5.3 µs** (4 KiB random) | — | 512 GB | **3.20 GB/s** seq | low |
| (HDD, reference) | magnetic platter | 2.08 ms half-rev @7200 rpm | — | 20 TB | 260 MB/s | lowest |

**Method.** Latency is a *dependent pointer chase*: each load's address is produced by the
previous load, so the CPU cannot overlap them. Working sets are swept from 4 KiB to 128 MiB.
Best-of-3 per point; 3 process-level runs; medians reported.

**Why both ns and cycles.** The cache tiers are measured in *cycles* as well as ns, because
ns alone is not a stable quantity on this box: an L1 hit costs a fixed number of cycles,
so its nanosecond value tracks the core clock, and this box's clock swings with thermal
state and background load. **The cycle figure is the architecture; the ns figure is what a
caller experiences.** Both are printed by the probe. (The DRAM row's cycle figure is the
least reliable: DRAM latency is mostly not clock-bound, so dividing it by a clock that
varies 2-3× across the sweep inflates it. The DRAM *ns* figure is the solid one.)

**The ladder is not smooth — it has knees, and the knees are the architecture:**

```
latency (ns)
   |
164|                                        ...........  DRAM (164 ns)
   |                                    ...
   |                                ...
 22|            ...L3 (22.0 ns)......
   |        ...
  4|   L2 (4.3)
   |  .
1.6| L1 (1.64)
   +--------------------------------------------------> working set
     4K  32K  128K   1M    4M    12M   16M   128M
              |__L2__|  |_____L3____|  |__DRAM__|
```

### 1.2 The processor zoo — four different answers to the same question

All four are von Neumann (shared instruction/data memory). They differ in **where the
memory sits relative to the arithmetic** — and that single choice is what sets their roofline.

| Processor | Execution model | Memory | Peak compute | Mem BW | **Balance (FLOP/byte)** | Roofline shape |
|---|---|---|---|---|---|---|
| **CPU** (this host) | scalar OoO, 1 instr/cycle/core | DDR4 + SRAM | 29.7 GFLOP/s (1T, AVX-512) | 15.6 GB/s | **2.92** (measured, same window) | shallow, memory-bound |
| **GPU** (H100) | 132 SMs, 2048 threads, SIMT | HBM3 80 GB | 67 TFLOPS FP32 | 3.35 TB/s | **20** (FP32) | deep, but compute-bound in FP32 |
| **GPU** (H100, tensor) | tensor cores | HBM3 | 990 TFLOPS FP16 | 3.35 TB/s | **295** | very deep |
| **TPU** (v4) | 2D systolic array, 4096/chip | HBM2 32 GB | 275 TOPS | 1.2 TB/s | **229** | deepest per byte |
| **TPU** (v7 Ironwood) | systolic | HBM3e 192 GB | 4614 TFLOPS FP8 | 7.37 TB/s | **626** | deepest |
| **TPU** (8t) | systolic, FP4 | HBM3e 216 GB | 12.6 PFLOP FP4 | 6.53 TB/s | **1930** | extreme |

**The one-line story:** a GPU/TPU does not beat a CPU at the von Neumann bottleneck — it
**moves the bottleneck**. Going from CPU (2.9) to TPU (626) FLOP/byte is a ~215× shift in
where the wall is, achieved by widening the memory interface (HBM: 1024-bit per stack vs
64-bit per DDR channel) and by dedicating the datapath to one operation shape.

**Published TPU generations** (Wikipedia, "Tensor Processing Unit"; bandwidth and TOPS as published):

| Gen | Process | Clock | On-chip mem | Memory | Mem BW | Perf | Balance |
|---|---|---|---|---|---|---|---|
| v1 | 28 nm | 700 MHz | 28 MiB | 8 GiB DDR3 | 34 GB/s | 23 TOPS (int8) | 676 |
| v2 | 16 nm | 700 MHz | 32 MiB | 16 GB HBM | 600 GB/s | 45 TFLOPS | 75 |
| v3 | 16 nm | 940 MHz | 128 MiB | 32 GiB HBM | 900 GB/s | 123 TFLOPS | 137 |
| v4 | 7 nm | 1050 MHz | 128 MiB | 32 GiB HBM | 1200 GB/s | 275 TOPS | 229 |
| v5e | — | — | — | 16 GB HBM | 819 GB/s | 197 TOPS (bf16) | 241 |
| v5p | — | — | — | 95 GB HBM | 2765 GB/s | 459 TOPS | 166 |
| v6e | — | 1750 MHz | — | 32 GB HBM | 1640 GB/s | 918 TOPS | 560 |
| v7 | — | — | — | 192 GB HBM3e | 7.37 TB/s | 4614 TFLOPS | 626 |
| 8t | — | — | — | 216 GB HBM3e | 6.53 TB/s | 12.6 PFLOP FP4 | 1930 |

### 1.3 The memory interface, generation by generation

**Rows = memory generation. Columns = interface width and rate, and the resulting bandwidth.**

| Gen | Interface | Pins/stack | Rate/pin | Bandwidth/stack | Note |
|---|---|---|---|---|---|
| HBM1 | 8×128b | 1024b | 1.0 GT/s | 128 GB/s | interposer required |
| HBM2 | 8×128b | 1024b | 2.0 GT/s | 256 GB/s | |
| HBM2E | 12×128b | 1024b | 3.6 GT/s | 461 GB/s | 24 GB/stack |
| HBM3 | 16×64b | 1024b | 6.4 GT/s | 819 GB/s | channels doubled, pins same |
| HBM3E | 16×64b | 1024b | 9.6 GT/s | 1229 GB/s | |
| HBM4 | 32×64b | **2048b** | 8.0 GT/s | 2048 GB/s | bus width doubled |
| DDR4-3200 (this host) | 1×64b | 64b | 3.2 GT/s | **25.6 GB/s** | **11× narrower than HBM3** |
| GDDR6X (GPU) | 16×32b | 512b | 24 GT/s | 1.8 TB/s | |

**The width is the whole story.** HBM3's 1024-bit interface is 16× a DDR channel's 64 bits.
That is why a GPU with 3.35 TB/s needs a 1024-bit bus and an interposer, and why the same
silicon on DDR4 would top out at 25.6 GB/s. **Bandwidth is bought with pins, and pins are
expensive** — which is the von Neumann bottleneck expressed in packaging.

---

## 2. The derived physical limits

Each is derived from a published constant, with the arithmetic shown.

### 2.1 Landauer limit — the energy floor of a bit

> Erasing one bit must dissipate at least `E ≥ k_B·T·ln2`.

```
k_B = 1.380649e-23 J/K
T   = 300 K
E   = 1.380649e-23 × 300 × 0.6931 = 2.871e-21 J per bit
```

A real CMOS switching event costs ~1e-15 J, so this hardware sits **3.48e5× above**
the thermodynamic floor. Published: "modern computers use about a billion times as much
energy per operation" (Landauer's principle, Wikipedia).

**Consequence:** energy per bit is *not* today's binding constraint. If computation ran at
the Landauer limit, the same chip would be ~350 000× more efficient. The binding
constraints are **communication** (section 2.2) and **geometry** (section 2.3), not
thermodynamics. This is why this project's optimisation target is *bytes moved*
(`VON_NEUMANN_MAPPING.md`: "Efficiency = fewer bytes moved, not more FLOPs").

### 2.2 Signal flight — the speed of light in silicon

> A signal in a loaded on-die wire travels at ~c/n with n ≈ 3.6.

```
v = 2.998e8 / 3.6 = 8.33e7 m/s
```

| Die span | One-way flight | Round trip | Implied max clock (round trip = 1 cycle) |
|---|---|---|---|
| 10 mm | 120.1 ps | 240.2 ps | 4.16 GHz |
| 25 mm | 300.2 ps | 600.4 ps | 1.67 GHz |
| 50 mm | 600.4 ps | 1200.8 ps | 0.83 GHz |

**Consequence:** a die twice as wide halves the maximum clock rate, purely from propagation
delay. This is the physical reason multi-core won: instead of making one core bigger
(and slower), you make the die wider and put several small cores in it, each with a short
round trip. **It is also why the L3 is a separate die-level concern and why on-package
(HBM) beats on-board (DDR) memory** — every centimetre of trace is 120 ps of flight time.

### 2.3 DRAM random access — the timing floor, and what it really costs

> DDR4-3200: 3200 MT/s → 0.625 ns per clock. CL22 → 22 cycles for the CAS latency.
> A random access is `tRCD` (row activate) + `tCL` (column) + `tRP` (precharge) = 3×22.

```
cycle           = 2000 / 3200 = 0.625 ns
tRCD+tCL+tRP    = 3 × 22 × 0.625 = 41.2 ns   <- the floor
```

**Measured decomposition.** The probe separates the two costs that a naive pointer chase
conflates, by holding the *cache footprint* constant and varying only the *page count*:

| Quadrant | Measured (ns) | Meaning |
|---|---|---|
| L3 hit, TLB hit | 38.1 | pure L3 + no walk |
| L3 hit, TLB miss | 74.0 | L3 + one page walk |
| DRAM, TLB miss | 170.8 | DRAM + one page walk |

```
page walk      = 74.0 - 38.1 =  46.6 ns
DRAM access    = 170.8 - 74.0 = 92.6 ns   (2.24x the 41.2 ns bare timing)
```

The extra 51 ns over the bare timing is row activation, bus turnaround, refresh stalls
and controller queueing — the difference between a datasheet number and a real machine.

**A structural finding.** The fourth quadrant (DRAM, TLB hit) is **not measurable** on
this host with 4 KiB pages: the STLB reach is 1536 entries × 4 KiB = **6 MiB**, which is
*smaller* than the 12 MiB L3, so any footprint that fits the TLB also fits the L3. Large
pages would be needed; this box refuses them (ERROR_PRIVILEGE_NOT_HELD, 1314). The
impossibility is a result, not a gap.

**And the headline cold-access number is therefore ~164 ns, not 93 ns.** A dense chase over
128 MiB (which touches 32 768 pages) costs 163.7 ns — it pays DRAM *and* a page walk
*and* heavy TLB pressure. The single biggest lever on cold random access on this box is
not the memory chip; it is the **page size**.

### 2.4 The von Neumann bottleneck, measured: machine balance

> Machine balance (roofline ridge point) = peak FLOP/s ÷ peak byte/s.
> Below this intensity, a kernel is memory-bound no matter how fast the ALU is.

The critical subtlety: **compute and bandwidth must be measured in the same window.** This
box's clock swings from 1.1 to 3.5 GHz across runs (thermal + background load), so a
balance computed from numbers taken minutes apart is meaningless. The probe interleaves
them (section G):

```
same-window read      = 15.59 GB/s
same-window compute   = 43.32 GFLOP/s
machine balance       = 2.92 FLOP/byte
```

**Now the decisive comparison** — the arithmetic intensity of *this project's own kernel*,
a dense d×d matvec per face (reads d² weights + d inputs, does d² FMAs):

| Kernel | Flops | Bytes (f32) | Intensity | vs measured ridge (2.92) |
|---|---|---|---|---|
| d=2 matvec | 8 | 24 | 0.333 | **8.8× below → memory-bound** |
| d=4 matvec | 32 | 80 | 0.400 | **7.3× below → memory-bound** |
| d=8 matvec | 128 | 288 | 0.444 | **6.6× below → memory-bound** |
| d=16 matvec | 512 | 1088 | 0.471 | **6.2× below → memory-bound** |

**This is the quantitative justification for the project's core design decision.** The
kernel is 6-9× below the ridge point, so it is *structurally* memory-bound. The only
lever that moves it is **bytes moved**, not FLOPs. That is exactly why:
- `f32` over `f64` (halves every stream) is the right trade — not "precision is enough",
  but "the kernel lives on the bandwidth wall".
- the factored Laplacian (`L = δ*δ`, never materialised) is right — it removes an
  O((Vd)²) write, not an O((Vd)²) multiply.
- fused columns (SoA) are right — same reason.
- pre-aggregation (P3) and the fiber bundle (B1/B2) are right — they remove *whole scans*.

**And the limit on the lever itself:** the 1-thread read bandwidth measured here
(11.5 GB/s) is 45% of the 25.6 GB/s single-channel DDR4 peak. So even a perfect
implementation cannot go much past ~11 GB/s on one core; the wall is 4× below what a naive
"just make the kernel faster" plan would target.

### 2.4a Does the database reach the memory ceiling? The gap is the accumulator

The roofline says the kernel is memory-bound. That predicts it should run at the
memory bandwidth. It does not. The probe measures the real database kernel and
the memory ceiling **in the same window**, so the difference cannot be thermal
drift.

**Rows = thread count. Columns = the memory ceiling and the real kernel, both
measured on the same 48.0 MB of TPC-H lineitem data in the same window.**

| Threads | Memory ceiling (GB/s) | Partition scan (GB/s) | Utilization (%) | Fused scan (GB/s) | Utilization (%) |
|---|---|---|---|---|---|
| 1 | 13.67 | 5.78 | **42.3** | 7.71 | 56.4 |
| 2 | 22.21 | 12.34 | **55.5** | 10.27 | 46.2 |
| 4 | 22.42 | 18.12 | **80.8** | 10.14 | 45.2 |

The ceiling row is a streaming read of the same two columns with the same thread
count. The kernel row is the storage engine's own partition scan. The ceiling
reaches 22.4 GB/s at four threads, so the memory system can feed the kernel
faster than the kernel consumes.

**The cause is the accumulator, not the memory.** The partition scan keeps one
floating-point accumulator per thread:

```rust
let mut acc = 0f64;
for (a, b) in pp.iter().zip(qq.iter()) {
    acc += *a as f64 * (1.0 - *b as f64);
}
```

Every row waits for the previous row's multiply-add to retire. The measured
multiply-add latency sets the pace, and the memory system idles in between.
The probe isolates this by running the same arithmetic with one lane and with
four independent lanes, in the same window:

| Lanes | Time (ms) | Effective (GB/s) | Note |
|---|---|---|---|
| 1 | 7.20 | 6.66 | serial dependency chain |
| 4 | 4.91 | 9.78 | four independent chains |
| — | — | — | **1.47x faster on the same bytes** |

**Why one thread is the worst case.** The dependency chain is per thread. At
one thread the chain sets the pace and the memory idles 58% of the time. At
four threads the four chains interleave, so the memory system stays busy and
utilization rises to 81%. The kernel has a ceiling of its own, and it is below
the memory ceiling at every thread count.

**What this means for the design.** The memory hierarchy was never the
bottleneck for the single-thread scan. The fix is a data layout and loop
change, not a faster machine and not a wider bus:

- The fused scan already uses four lanes, and it is 1.33x faster than the
  partition scan at one thread.
- The partition scan is the one that still uses a single accumulator.
- Both kernels produce the same aggregate to within 1e-12, so the change is
  safe to make.

The repo already moved the column layout to structure of arrays (FILE-076).
The accumulator lanes are the next step, and the probe now measures the size
of that prize: **1.47x on the single-thread scan.**

### 2.5 Bandwidth-delay product — the concurrency the memory system must hide

> By Little's law, to sustain bandwidth B over a latency L you need B×L bytes in flight.

```
B = 19.2 GB/s (8 threads), L = 164 ns  =>  B×L = 3081 bytes
```

The host's outstanding-miss capacity (LFBs + fill buffers) is in this ballpark, which is
why 4 threads reaches 17.0 GB/s but 8 threads adds little (19.2 GB/s — the memory
controller is saturated). **The scaling limit is not cores; it is the memory controller.**
This corrects the earlier repo note ("thread scaling is NOT a lever past ~2 threads"):
1→4 threads gives 1.47×, and 4→8 gives 1.13×.

### 2.6 Storage: the bus ceiling and the durability floor

**PCIe 4.0 x4** (16 GT/s per lane, 128b/130b encoding):

```
16 GT/s × 4 lanes × (128/130) ÷ 8 = 7.88 GB/s
```

Measured sequential read: **3.20 GB/s = 40.6% of the bus ceiling.** The NVMe device itself
is not the limit here; the drive is a 512 GB OEM part behind a shared controller.

**fsync: the one number that matches theory almost exactly.**

| Operation | Measured | Note |
|---|---|---|
| 4 KiB buffered write (no sync) | 3.6 µs | goes to the OS page cache |
| 4 KiB write + `sync_all` | **1307.6 µs** | 363× more expensive |
| 1 MiB write (no sync) | 584.5 µs | |

The ratio (363×) is the *cost of durability*, and it is a physical property of the device,
not of the code. This matches the repo's earlier independent measurement (1.207 ms) to
within 8%, from a different binary and a different day — the residual is the drive's own
flush-path variability, not a measurement disagreement.

**The layering ratio that justifies pre-aggregation:**

```
storage random access (5.3 us) / DRAM random access (164 ns) = 33.0x
```

Every question that can be answered from RAM instead of NVMe saves a factor of ~33 in
latency. That is the whole economic argument for the fiber bundle.

### 2.7 Magnetic areal density — the thermal-stability wall

> A magnetic bit is stable for time *t* only if the anisotropy barrier `E = K·V` exceeds
> `kT·ln(t/τ0)`, with `τ0 ≈ 1 ns` (thermal attempt frequency).

```
τ0  = 1e-9 s
t   = 10 years = 3.15e8 s
E   = ln(3.15e8 / 1e-9) × k_B × T = 26.8 × 4.14e-21 = 1.11e-19 J
K   = 2.2e5 J/m^3  (CoCrPt-like)
V   = E/K = 5.05e-25 m^3
side= V^(1/3) = 9.12 nm
areal density = 1/side^2 = 1.20 Tb/in^2
```

**Consequence:** for perpendicular recording, the grain cannot be shrunk below ~9 nm
without the bit flipping spontaneously, so **areal density is capped by thermal stability,
not by lithography**. This is why disks stopped scaling on area and started scaling
vertically (HAMR) and in capacity-per-platter instead.

---

## 3. Comparing against the numbers already in this repo

**Rows = an existing repo number. Columns = what it claimed, and what this probe measured.**

| Repo number | Where | Claimed | Measured now | Agreement |
|---|---|---|---|---|
| Read bandwidth | `bottleneck.json` A | 4.2 GB/s copy (=8.4 GB/s read) | 11.5 GB/s read (1T) | consistent (read vs copy) |
| 4-thread scaling | `bottleneck.json` B | 1.25× (4t vs 1t) | **1.47×** (17.0 vs 11.5) | **corrected — see below** |
| fsync floor | `bottleneck.json` D | 1.207 ms | 1.308 ms | 8% (device flush variability) |
| CRC cost | `bottleneck.json` E | 2% of durable op | not re-measured | — |
| L3 domain budget | `SPEC-016` | 64 MB | **12 MB L3** | **see below** |
| LLC host qualification | `SPEC-007` | "12 MB laptop LLC" | **confirmed: 12 MiB** | exact |
| Sub-15 ns lookup | `SPEC-007` | 11.64 ns median | L1 1.64 / L2 4.27 ns | consistent |
| Memory wall | `MAX_PERF_PLAN` W1 | "8.4 GB/s is the floor" | 11.5 GB/s, **45% of DRAM peak** | refined |

### 3.1 Correction: thread scaling is 1.47×, not 1.25× — and the limit is the controller

The old probe measured a **memcpy** (read+write traffic) and concluded "4-thread barely
scales (controller saturated)". This probe measures a **pure read stream**:

| Threads | Read GB/s | Scaling |
|---|---|---|
| 1 | 11.53 | 1.00× |
| 2 | 16.17 | 1.40× |
| 4 | 16.95 | **1.47×** |
| 8 | 19.18 | 1.66× |

The earlier 1.25× figure was an artefact of counting a copy's write traffic against the
read bus. The corrected picture is still "scaling stops at the memory controller", but for
a sharper reason: the 8-thread figure (19.18 GB/s) is **74.9% of the 25.6 GB/s
single-channel DDR4 peak**, and the 4→8 step adds only 13% while the DRAM bus is
75-95% saturated. **The scaling limit is not cores; it is the 64-bit memory bus.** Since
the box has only one populated DIMM, adding the second channel would roughly double the
ceiling — a hardware fact, not an optimisation.

### 3.2 Correction: the "64 MB L3 domain" assumption is 5.3× this host's L3

`SPEC-016` and `ARCHITECTURE.md` partition data into **64 MB "L3 domains"**. This host has
a **12 MiB L3** — 5.3× smaller. Consequences, all now measured:

- The partitioner's own gate passes (`domains_fit_l3`) because it checks a *budget constant*,
  not the real LLC.
- The measured L3 latency is 21.97 ns and the L2 is 4.27 ns — a **5.1× cliff** at the L2/L3
  boundary. A "64 MB domain" on this host is therefore mostly **DRAM** (163.7 ns), not L3.
  The real cliff for this host is at **12 MiB**, and a 64 MB domain would sit **5.3× past
  the knee**, i.e. entirely in DRAM.
- This is the same effect the repo already documented for `SPEC-007` ("marginal on 12 MB
  laptop LLC, ~50% process PASS, excursions to ~30 ns"). The two findings are the same
  wall seen from two directions, and this probe now measures the wall directly.

**The architecture decision this forces:** the 64 MB figure is a **server** target (a
server L3 is 32-64 MB per socket, often 256 MB+). On this dev box the honest budget is the
**12 MiB real LLC**, and a domain plan sized to it would produce very different ghost-replica
counts. The plan should size domains to the *measured* LLC of the target class, and the
probe makes that measurable rather than assumed.

### 3.3 Confirmation: the repo's "fewer bytes, not more FLOPs" thesis

`VON_NEUMANN_MAPPING.md` already concluded "Efficiency = fewer bytes moved, not more
FLOPs (memory wall binds)". This probe turns that from a qualitative claim into a
**number**: the measured ridge point is **2.92 FLOP/byte**, and the project's kernel runs at
**0.33-0.47 FLOP/byte** — 6-9× below the ridge. The kernel *is* memory-bound, and the
design (f32, factored L, SoA fused columns, pre-aggregation, fiber bundle) is the correct
response. The measurement also bounds the ambition: the 1-core read wall is 11.5 GB/s, so
the achievable gain from a perfect kernel is bounded by bytes moved, not by arithmetic.

---

## 4. What the physics says is *not* the limit

Equally important, and a place where intuition is usually wrong:

| Tempting explanation | What the measurement says |
|---|---|
| "The 2.8 ms scan is slow because of the CPU" | Ridge point 2.92 vs kernel 0.40 FLOP/byte → the CPU is idle; memory is the wall |
| "fsync is slow because of the OS" | 4 KiB buffered write = 3.6 µs, fsync = 1308 µs; the cost is the device's flush-to-media path, and it matches theory |
| "The clock is the limit" | The clock is why the *ns* figures move, but the *cycle* figures are stable (L1 = 4.6, L2 = 13.7, L3 = 62.6) — and the scan is bandwidth-bound, so the shape is the same at any clock |
| "We need more FLOPs" | The kernel is 6-9× *below* the ridge; adding FLOPs cannot help |
| "Landauer/energy is why laptops throttle" | Landauer is 3.5e5× below CMOS; energy is a *power-delivery* limit, not an information limit |
| "Large pages would fix cold random access" | Correct — and unmeasurable here: the STLB reach (6 MiB) is *below* the L3 (12 MiB), so with 4 KiB pages a TLB-resident footprint always fits in L3 |
| "More cores would fix the bandwidth" | 4→8 threads adds 13%: the 64-bit DDR bus is 75-95% saturated, and the host has only ONE populated channel |
| "GPUs would fix this" | GPUs fix it by moving the ridge (2.9 → 20..1930 FLOP/byte) via a 1024-bit memory bus, not by being faster at the same bus |

---

## 5. Reproducing this

```bash
# The probe (gate + full output)
cd sheaf-db && cargo run -p sheaf-engine --bin physical_limits_probe --release
sh tests/e2e/von_neumann/run.sh

# 3-run summary
powershell -ExecutionPolicy Bypass -File tests/e2e/realtime_bench/summarize_physical_limits.ps1
```

**The 6 physics-invariant gates** (asserted, not reported — a failure means the measurement
is wrong, not that the machine is odd):

| Gate | Assertion |
|---|---|
| `gate_hierarchy_ordered` | L1 < L2 < L3 < DRAM, strictly |
| `gate_dram_slower_than_l3` | DRAM access exceeds L3 access |
| `gate_dram_above_ddr4_cas_floor` | DRAM > 41.2 ns (else we never reached DRAM) |
| `gate_fsync_floor_exists` | fsync > 10× a buffered write of the same size |
| `gate_read_bw_significant` | read bandwidth > 5% of the DRAM bus peak |
| `gate_dram_slower_than_l1` | DRAM exceeds L1 |

Plus the clock cross-checks: the 8-add vs 4-add chain ratio must be 2.000, and the
4×imul vs 4-add ratio must be 3.000 — the second is what validates the *absolute* cycle
count, and both were what caught two wrong clock derivations during development.

---

## 6. Honest limits of this study

- **One host, one machine class.** A laptop i7-1165G7 with **one** populated DDR4 channel
  and **no** spinning disk. The HDD rows are derived from published timings, not measured
  here; the NVMe rows are measured.
- **The clock is not stable.** It swung 1.44-3.21 GHz across the session, with thermal
  state and ~25-30% background load from other processes. Absolute-latency numbers are
  therefore reported in **cycles** (stable) as well as ns (what a caller sees); the
  clock-dependent ones (FLOP/s) vary more, which is why the machine balance is computed
  same-window and why the cross-checks are asserted rather than eyeballed.
- **The clock anchor was wrong twice before it was right.** The first two versions
  hand-analysed a Rust loop's cycle count from the disassembly and got it wrong (LLVM
  strength-reduced the chain and unrolled it; then a `mov` was not eliminated). The
  current probe derives the clock from the *marginal* cost of one added dependency and
  validates it with two independent cross-checks (8-add vs 4-add = 2.02 vs 2.00;
  4×imul vs 4-add = 3.09 vs 3.00). A third bug — dividing *independently best-taken*
  chain times, which mixes thermal windows — was caught by the same cross-checks
  (1.78 instead of 2.00) and is why the probe now measures adjacent pairs and takes the
  median of per-pair ratios.
- **The DRAM-access decomposition is a 3-quadrant inference.** The 4th quadrant (DRAM with
  a TLB hit) is unreachable with 4 KiB pages on this host (§2.3), so the split between
  "DRAM access" and "page walk" rests on holding the footprint constant — a sound but
  indirect method.
- **The TPU/GPU rows are `published`, not measured.** Only the CPU row is this host's.
- **`landauer` uses T = 300 K** and a CMOS switching cost of ~1e-15 J, which is a
  representative rather than a measured figure for this specific part.
- **The `d×d` kernel-intensity figures in §2.4 are analytical**, computed from the matvec
  shape; they are not a measured profile of the shipped kernel.
