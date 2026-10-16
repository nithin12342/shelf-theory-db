# ShelfDB server/client version — detailed plan

| Item | Decision |
|---|---|
| Goal | `shelf-server` daemon + thin clients (CLI, Rust lib, Python); engine stays Rust |
| Keep | In-process embedded path (10 ns class) alongside client-server path |
| Default bind | 127.0.0.1:7432 (no clash: PG 5433, Maria 3307, Redis 6379, Mongo 27017, Neo4j 7687, Scylla 9042) |
| Auth | Token file; loopback trusts token, non-loopback refuses without it |
| Numbers policy | Report both classes (in-process vs loopback), never mix — established precedent |

## Architecture

```
client (CLI / py / rs lib)
  |  TCP loopback, length-prefixed frames [MAGIC|len|crc|msg] (reuse WAL framing)
  v
listener (1 thread, accept only)
  v
session pool (N workers) — one MemComplex shared, Star-shard locks
  v
TQL parse (existing phase4) → plan (existing optimizer/validator)
  v
kernel (Laplacian/solvers, BCSF reads) / WAL (writes, existing epoch path)
  v
response frames (rows / ERROR code — fix-loud over the wire, never silent)
```

## Wire messages

| # | Type | Payload | Reply |
|---|---|---|---|
| 1 | HELLO | client version, token | READY / AUTH-FAIL |
| 2 | QUERY | TQL text | ROWS (streamed frames) / ERROR |
| 3 | INSERT | table + fields | ACK seq / ERROR |
| 4 | PING | — | PONG (readiness probe for runner) |
| 5 | CLOSE | — | BYE |

| Limit | Value | Reason |
|---|---|---|
| Max frame | 64 MB | Matches L3 domain; kills OOM-by-client |
| Query timeout | 300 s | Longest join (SQLite 250 s) + margin |
| Max sessions | 64 | Matches 32-thread test headroom |

## New files (all inside `sheaf-db/`)

| File | Role | Phase |
|---|---|---|
| `crates/sheaf-net/src/lib.rs` + `protocol.rs` | Frame codec (MAGIC\|len\|crc), msg enum | P1 |
| `crates/sheaf-net/src/session.rs` | Session state, timeout, limits | P1 |
| `crates/sheaf-server/src/main.rs` (`--port --datadir --auth-token`) | Daemon: listener + pool | P1–P2 |
| `crates/sheaf-client/src/lib.rs` | Rust client (connect/query/fetch) | P2 |
| `crates/sheaf-client/src/bin/shelf-cli.rs` | CLI (`query "..."`, `insert ...`) | P2 |
| `tests/e2e/realtime_bench/shelf_client.py` | Python client for benches | P2 |
| `D:\bench\data\shelf\` | Server data dir (BCSF + WAL + snapshot) | P3 |

## Concurrency (from MT lessons)

| Lesson (measured) | Design answer |
|---|---|
| Single-mutex p99 spikes (1.5–6.5 ms at 32t) | Shard session state; share only Star shards + WAL file mutex |
| Readers never block each other | BCSF readers immutable + refcounted; RCU-style snapshot swap on seal |
| One slow query must not stall pool | Per-session timeout + dedicated slow-lane worker (1 slot) |

## Durability and transactions

| Now | Server v1 | Later |
|---|---|---|
| Single-op atomic (WAL frame) | Same, per INSERT | Multi-op TX (2PC-free: single-node group commit) |
| Epoch fsync policy | Same (epoch boundaries) | Client-requested SYNC flag per write |
| No auth | Token file | Users/roles (deferred, not designed here) |

## Benchmark impact (honest, upfront)

| Number class | Expected | Example |
|---|---|---|
| In-process (keep) | Unchanged | Lookup 10.5 ns, traversal 41.7 ms |
| Loopback (new) | +50–100 µs floor (measured PG/Redis class) | Lookup ≈ 60–100 µs via client |
| Rule | Both recorded side by side; matrix gains a `client_` column | No silent substitution |

## Deployment (Windows, this box)

| Item | Setting |
|---|---|
| Run mode | Scheduled task on logon (no service wrapper dependency) |
| Data | `D:\bench\data\shelf\` (kept-install convention) |
| Logs | `D:\bench\data\shelf\server.log` (rotation at 100 MB) |
| Runner integration | `run_all.py` gains `shelf-server` lifecycle (start → PING probe → benches → stop) |

## Phases and gates

| Phase | Delivers | Gate (E2E, real binary) |
|---|---|---|
| P1 | Codec + echo server + PING | Round-trip frame, torn-frame reject, timeout kill |
| P2 | Read path (QUERY→ROWS) + CLI + py client | TPC-H Q1/Q6 match in-process bytes exactly |
| P3 | INSERT→WAL + shared sessions (8 concurrent) | 20k inserts replay-exact; mixed workload no deadlock |
| P4 | Token auth + limits + slow-lane | AUTH-FAIL without token; 64 MB frame refused; timeout fires |
| P5 | Bench parity | `client_` column filled for all 7 cases; both classes published |

## Risks

| Risk | Mitigation |
|---|---|
| Loopback floor dwarfs ns-scale wins | Dual reporting (policy above); embedded path untouched |
| Windows TCP + thread pool stalls | PING-gated runner; slow-lane isolation |
| Protocol ossification | Version in HELLO; server rejects unknown major |
| Scope creep (users/roles/TLS) | Explicitly deferred; token + loopback suffice for v1 |
