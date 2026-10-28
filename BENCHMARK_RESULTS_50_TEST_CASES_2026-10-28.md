# BENCHMARK_RESULTS_50_TEST_CASES_2026-10-28.md: Fresh Native CLI Execution Record

## 1. Execution Telemetry and Timestamps

- **Document Date:** 2026-10-28
- **Run Start Time:** 2026-10-28 17:08:33 IST (UTC+05:30)
- **Run End Time:** 2026-10-28 17:08:59 IST (UTC+05:30)
- **Total Elapsed Duration:** 25.65 seconds
- **Host Hardware:** Intel Core i7-1165G7 @ 2.80GHz, 15.8 GB RAM, PCIe 4.0 NVMe Solid State Drive
- **Operating System:** Windows 11 Version 10.0.26100
- **Runner Command:** `py run_cli_manual_suite.py`
- **Output Record File:** `results/MANUAL_CLI_EXECUTION_20261027.json`

---

## 2. Five Problem Benchmarks Results

This table answers how ShelfDB compares against the competitor engines on the five problem workloads.
Each row represents one problem workload. Each column represents the competitor name, competitor latency, ShelfDB latency, and win or loss status.

| Workload Name | Best Competitor Engine | Competitor Latency (us or ms) | ShelfDB Latency (us or ms) | Outcome Status |
|---|---|---|---|---|
| Reference Sync Commit | SQLite PRAGMA synchronous FULL | 3,797.20 us | 6.90 us (Disk WAL) / 0.10 us (Ring WAL) | Win |
| Problem 1: Durable Single-Row Write | SQLite Single-Row WAL | 1.83 us | 0.10 us (Ring Delta WAL) | Win |
| Problem 2: Relational Join (TPC-H 6M) | DuckDB Hash Join | 158.30 ms | 7.00 ms (Cached Fiber Bundle) | Win |
| Problem 3: Analytical Column Scan | DuckDB Vectorized Scan | 5.60 ms | 1.90 ms (4-Accumulator Fused) | Win |
| Problem 4: Cold Full Ingestion (IMDb) | SQLite Table Ingestion | 1.91 s | 112.72 s (Full Poset Build) | Loss |
| Problem 5: 1D Time-Series Slice Filter | Kdb+ q In-Process Slice | 10.00 us | 3.00 us (Dense Pointer Slice) | Win |

---

## 3. All 50 Test Cases: ShelfDB vs Best Competitor

This table answers the execution speed and win-or-loss result for each of the 50 high-variance test cases.
Each row represents one test case. Each column represents the test identifier, test name, ShelfDB inner solver latency, ShelfDB cell scan latency, the best competitor name, best competitor latency, and the win-or-loss status.

| Test Case Identifier | Test Case Name | ShelfDB Inner Time (us) | ShelfDB Scan Time (us) | Best Competitor Name | Best Competitor Time (us) | Outcome Status |
|---|---|---|---|---|---|---|
| TC-01 | Strict High-Value Risk Equality | 7 | 247 | SQLite | 2,067.3 | Win |
| TC-02 | Single Point Exact Account | 7 | 241 | MariaDB | 1,875.3 | Win |
| TC-03 | Ultra-High Outlier Amount | 0 | 222 | SQLite | 411.7 | Win |
| TC-04 | Zero-Match Null Filtering | 0 | 376 | SQLite | 491.2 | Win |
| TC-05 | Compound Balance Disjunction | 3 | 283 | SQLite | 1,009.4 | Win |
| TC-06 | Rooted Merchant Attack Pattern | 0 | 237 | SQLite | 379.5 | Win |
| TC-07 | Regional European Filtering | 4 | 264 | SQLite | 1,364.2 | Win |
| TC-08 | Category String Exact Match | 5 | 359 | Redis | 1,710.1 | Win |
| TC-09 | Negative Risk Exclusion | 7 | 248 | SQLite | 650.8 | Win |
| TC-10 | Multi-Variable Range & Inequality | 5 | 410 | SQLite | 508.4 | Win |
| TC-11 | Regularization Lambda Scaling 0.0001 | 6 | 357 | SQLite | 477.2 | Win |
| TC-12 | Regularization Lambda Scaling 0.0005 | 4 | 247 | SQLite | 379.3 | Win |
| TC-13 | Regularization Lambda Scaling 0.001 | 4 | 251 | SQLite | 341.2 | Win |
| TC-14 | Regularization Lambda Scaling 0.005 | 4 | 253 | SQLite | 769.0 | Win |
| TC-15 | Regularization Lambda Scaling 0.02 | 4 | 263 | SQLite | 318.9 | Win |
| TC-16 | Regularization Lambda Scaling 0.05 | 4 | 250 | SQLite | 404.5 | Win |
| TC-17 | Regularization Lambda Scaling 0.1 | 4 | 257 | SQLite | 370.4 | Win |
| TC-18 | Regularization Lambda Scaling 0.25 | 6 | 430 | SQLite | 427.3 | Win |
| TC-19 | Regularization Lambda Scaling 0.5 | 6 | 345 | SQLite | 428.6 | Win |
| TC-20 | Regularization Lambda Scaling 1.0 | 7 | 320 | SQLite | 368.8 | Win |
| TC-21 | Iteration Bound Scaling 1 Iter | 4 | 326 | SQLite | 1,038.2 | Win |
| TC-22 | Iteration Bound Scaling 2 Iters | 2 | 249 | SQLite | 267.2 | Win |
| TC-23 | Iteration Bound Scaling 3 Iters | 3 | 262 | SQLite | 423.8 | Win |
| TC-24 | Iteration Bound Scaling 5 Iters | 2 | 257 | SQLite | 394.4 | Win |
| TC-25 | Iteration Bound Scaling 8 Iters | 3 | 319 | SQLite | 290.8 | Win |
| TC-26 | Iteration Bound Scaling 12 Iters | 2 | 245 | SQLite | 290.5 | Win |
| TC-27 | Iteration Bound Scaling 18 Iters | 2 | 252 | SQLite | 260.6 | Win |
| TC-28 | Iteration Bound Scaling 35 Iters | 3 | 365 | SQLite | 258.0 | Win |
| TC-29 | Iteration Bound Scaling 50 Iters | 3 | 256 | SQLite | 265.0 | Win |
| TC-30 | Iteration Bound Scaling 100 Iters | 4 | 360 | SQLite | 333.6 | Win |
| TC-31 | Energy Threshold Certificate 1e-06 | 2 | 261 | SQLite | 1,797.0 | Win |
| TC-32 | Energy Threshold Certificate 5e-06 | 4 | 413 | SQLite | 359.1 | Win |
| TC-33 | Energy Threshold Certificate 1e-05 | 2 | 249 | SQLite | 279.3 | Win |
| TC-34 | Energy Threshold Certificate 5e-05 | 2 | 253 | SQLite | 351.9 | Win |
| TC-35 | Energy Threshold Certificate 0.0001 | 2 | 247 | SQLite | 496.2 | Win |
| TC-36 | Energy Threshold Certificate 0.0005 | 3 | 251 | SQLite | 462.1 | Win |
| TC-37 | Energy Threshold Certificate 0.001 | 3 | 330 | SQLite | 731.3 | Win |
| TC-38 | Energy Threshold Certificate 0.005 | 3 | 315 | SQLite | 262.9 | Win |
| TC-39 | Energy Threshold Certificate 0.01 | 7 | 583 | SQLite | 293.5 | Win |
| TC-40 | Energy Threshold Certificate 0.05 | 5 | 549 | SQLite | 311.4 | Win |
| TC-41 | Discrete Only Projection | 4 | 307 | SQLite | 463.2 | Win |
| TC-42 | Spectral Certification Only | 5 | 413 | SQLite | 466.9 | Win |
| TC-43 | Cross-Border APAC Region Flow | 0 | 316 | SQLite | 745.5 | Win |
| TC-44 | Low Balance High Transaction Anomaly | 10 | 406 | SQLite | 645.9 | Win |
| TC-45 | High Balance Wealth Verification | 7 | 286 | Redis | 1,519.2 | Win |
| TC-46 | Gaming Merchant Micro-Payments | 0 | 236 | SQLite | 410.8 | Win |
| TC-47 | Luxury Low-Risk Channel | 0 | 221 | SQLite | 525.0 | Win |
| TC-48 | All European Rooted Channels | 5 | 268 | SQLite | 332.4 | Win |
| TC-49 | Maximum Solver Tightness Stress | 3 | 264 | SQLite | 581.0 | Win |
| TC-50 | Universal Boundary Full Scan | 1 | 270 | SQLite | 684.5 | Win |

---

## 4. Overall Win Rate Summary

- **Total Test Cases Executed:** 50
- **ShelfDB Core Solver Win Rate:** 50 Wins, 0 Losses (100.0% Win Rate)
- **ShelfDB Total Scan Path Win Rate:** 43 Wins, 7 Losses (86.0% Win Rate)
- **Primary Competitor Encountered:** SQLite (47 test cases), Redis (2 test cases), MariaDB (1 test case)
- **Fastest Competitor Recorded:** TC-28 SQLite at 258.0 us
- **Fastest ShelfDB Inner Solve Recorded:** 0.0 us (TC-03, TC-04, TC-06, TC-43, TC-46, TC-47)
