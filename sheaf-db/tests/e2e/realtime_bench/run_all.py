"""run_all.py — one-command full-matrix bench. Keeps installs + data, never deletes.

Usage: py run_all.py [--engines sqlite,duckdb,pg] [--fresh] [--keep-up]
                     [--no-shelf] [--shelf-only]
--fresh: delete sqlite/duckdb DB files first (true reload; default reuses =
  fast). --keep-up: leave servers running after. Results merge into existing
  JSONs as runN_dated entries (history preserved, never clobbered).
"""
import argparse
import datetime
import json
import os
import socket
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
RES = os.path.join(HERE, "results")
IST = datetime.timezone(datetime.timedelta(hours=5, minutes=30))

MARIADB = r"D:\bench\scoop\apps\mariadb\current\bin\mysqld.exe"
PGCTL = r"D:\bench\scoop\apps\postgresql\current\bin\pg_ctl.exe"
MONGOD = r"D:\bench\scoop\shims\mongod.exe"
REDIS_SRV = r"D:\bench\apps\redis\redis-server.exe"


def stamp():
    return datetime.datetime.now(IST).strftime("%Y-%m-%d %H:%M %Z")


def wait_port(port, timeout=120):
    t0 = time.time()
    while time.time() - t0 < timeout:
        try:
            socket.create_connection(("127.0.0.1", port), timeout=2).close()
            return True
        except OSError:
            time.sleep(2)
    return False


def sh(cmd, timeout=3600):
    return subprocess.run(cmd, shell=True, capture_output=True, text=True,
                          cwd=HERE, timeout=timeout)


ENGINES = {
    "sqlite": {"bench": "py sqlite_bench.py", "json": "sqlite.json"},
    "duckdb": {"bench": "py duckdb_bench.py", "json": "duckdb.json"},
    "redis": {"bench": "py redis_bench.py", "json": "redis.json",
              "start": f'Start-Process "{REDIS_SRV}" "--dir D:\\bench\\data\\redis --maxmemory 4gb --maxmemory-policy noeviction --port 6379" -WindowStyle Hidden',
              "port": 6379, "stop": "redis-server"},
    "mongodb": {"bench": "py mongo_bench.py", "json": "mongodb.json",
                "start": f'Start-Process "{MONGOD}" "--dbpath D:\\bench\\data\\mongo --port 27017 --bind_ip 127.0.0.1" -WindowStyle Hidden',
                "port": 27017, "stop": "mongod"},
    "mariadb": {"bench": "py sql_bench.py mariadb", "json": "mariadb.json",
                "start": f'Start-Process "{MARIADB}" "--datadir=D:\\bench\\data\\mariadb --port=3307 --innodb-buffer-pool-size=2G --console" -WindowStyle Hidden',
                "port": 3307, "stop": "mysqld"},
    "pg": {"bench": "py sql_bench.py pg", "json": "pg.json",
           "start": f'Start-Process "{PGCTL}" \'-D "D:\\bench\\data\\pgsql" -o "-p 5433 -c shared_buffers=2GB -c synchronous_commit=on" -l "D:\\bench\\data\\pgsql\\logfile" start\' -WindowStyle Hidden',
           "port": 5433, "stop_pg": True},
    "neo4j": {"bench": "py neo4j_bench.py", "json": "neo4j.json",
              "start": "docker start bench-neo4j", "port": 7687, "stop": None},
    "scylladb": {"bench": "py scylla_bench.py", "json": "scylladb.json",
                 "start": "docker start bench-scylla", "port": 9042, "stop": None},
}
ORDER = ["sqlite", "duckdb", "redis", "mongodb", "mariadb", "pg", "neo4j", "scylladb"]

SHELF_BINS = [("sheaf-engine", "tpch_scan_bench"), ("sheaf-engine", "fiber_join_bench"),
              ("sheaf-engine", "graph_bench"), ("sheaf-engine", "mt_bench"),
              ("sheaf-storage", "wal_insert_bench"), ("sheaf-engine", "epoch_wal_bench"),
              ("sheaf-engine", "real_load_bench")]

HEADLINES = {"sqlite.json": ("join_ms", "scan_agg_ms"), "duckdb.json": ("join_ms", "scan_agg_ms"),
             "redis.json": ("point_lookup_us", "lua_5key_us"), "mongodb.json": ("lookup4_indexed_ms", "mixed_32t"),
             "mariadb.json": ("join_ms", "scan_agg_ms"), "pg.json": ("join_ms", "scan_agg_ms"),
             "neo4j.json": ("full_traversal_ms", "two_hop_us"), "scylladb.json": ("point_lookup_us", "range_slice_ms")}


def merge_result(path, prior):
    """Wrap a fresh single-run output with prior multi-run history."""
    try:
        new = json.load(open(path))
    except Exception as e:
        return {"error": f"bench output unreadable: {e}", "prior_kept": bool(prior)}
    runs = {k: v for k, v in (prior or {}).items() if k.startswith("run")}
    tag = f"run{len(runs) + 1}_{datetime.datetime.now(IST).strftime('%Y%m%d')}"
    if runs or not any(k.startswith("run") for k in new):
        merged = dict(prior or {})
        merged[tag] = new
        merged[tag]["note"] = f"runner {stamp()}"
        merged["note"] = f"{len(runs) + 1} runs; latest {tag}."
        return merged
    return new


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--engines", default=",".join(ORDER))
    ap.add_argument("--fresh", action="store_true")
    ap.add_argument("--keep-up", action="store_true")
    ap.add_argument("--no-shelf", action="store_true")
    ap.add_argument("--shelf-only", action="store_true")
    a = ap.parse_args()
    want = [e for e in a.engines.split(",") if e in ENGINES]
    if a.fresh:
        for f in (r"D:\bench\data\sqlite\bench.db", r"D:\bench\data\duckdb\bench.duckdb"):
            if os.path.exists(f):
                os.remove(f)
                print(f"fresh: deleted {f}")
    summary = {}
    started = []
    if not a.shelf_only:
        for name in ORDER:
            if name not in want:
                continue
            e = ENGINES[name]
            print(f"\n===== {name} =====")
            if e.get("start"):
                sh(f'powershell -c "{e["start"]}"')
                if not wait_port(e["port"]):
                    print(f"{name}: server failed to start, SKIP"); summary[name] = "SERVER-FAIL"; continue
                started.append(name)
            prior = None
            jp = os.path.join(RES, e["json"])
            if os.path.exists(jp):
                try:
                    prior = json.load(open(jp))
                except Exception:
                    prior = None
            r = sh(e["bench"])
            print((r.stdout or "")[-600:])
            if r.returncode != 0:
                print(f"{name}: BENCH-FAIL"); summary[name] = "BENCH-FAIL"; continue
            try:
                merged = merge_result(jp, prior)
                json.dump(merged, open(jp, "w"), indent=2)
                summary[name] = "OK"
            except Exception as ex:
                summary[name] = f"MERGE-FAIL {ex}"
    if not a.no_shelf:
        print("\n===== shelf-db bins =====")
        sh("cargo build --manifest-path D:\\d2\\sheaf-db\\Cargo.toml --release --bins -p sheaf-engine -p sheaf-storage")
        for crate, b in SHELF_BINS:
            r = sh(f"cargo run --manifest-path D:\\d2\\sheaf-db\\Cargo.toml -p {crate} --bin {b} --release")
            out = (r.stdout or "")[-300:]
            print(f"-- {b}: {out.strip().splitlines()[-1] if out.strip() else 'FAIL'}")
            summary[f"shelf:{b}"] = "OK" if r.returncode == 0 else "FAIL"
    if not a.keep_up:
        for name in started:
            e = ENGINES[name]
            if e.get("stop"):
                sh(f'powershell -c "Get-Process {e["stop"]} -ErrorAction SilentlyContinue | Stop-Process -Force"')
            elif e.get("stop_pg"):
                sh(f'powershell -c "$env:PATH=\'D:\\bench\\scoop\\shims;\'+$env:PATH; & \'{PGCTL}\' -D \'D:\\bench\\data\\pgsql\' stop"')
            elif name in ("neo4j", "scylladb"):
                sh(f"docker stop bench-{name}")
        print("servers stopped (installs + data kept)")
    print("\n===== HEADLINES =====")
    for name in ORDER:
        if name not in want:
            continue
        try:
            d = json.load(open(os.path.join(RES, ENGINES[name]["json"])))
            rk = sorted([k for k in d if k.startswith("run")])[-1]
            cols = HEADLINES[ENGINES[name]["json"]]
            vals = {c: (d[rk].get(c, {}).get("p50", d[rk].get(c, {}).get("ms", d[rk].get(c)))) for c in cols}
            print(f"{name} [{rk}]: {vals}")
        except Exception as ex:
            print(f"{name}: unreadable ({ex})")
    sp = os.path.join(RES, f"RUN_SUMMARY_{datetime.datetime.now(IST).strftime('%Y%m%d_%H%M')}.json")
    json.dump({"stamp": stamp(), "summary": summary}, open(sp, "w"), indent=2)
    print("SAVED", sp)


if __name__ == "__main__":
    sys.exit(main())
