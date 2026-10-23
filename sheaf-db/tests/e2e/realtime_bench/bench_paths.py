"""bench_paths.py - the one place for machine-specific bench roots.

The repository holds no absolute path. The roots come from the environment,
or from ~/.sheaf_bench.json. run_all.py writes its root flags into the
environment for every child process, so one command sets all of them.

    py run_all.py --data <root> --build <root> --datasets <root> \
                  --imdb <root> --pg-app <dir> --mariadb-app <dir> \
                  --redis-server <exe> --mongod <exe> --pgbench-data <dir>
"""
import json
import os
from pathlib import Path

HERE = Path(__file__).resolve().parent
RESULTS = HERE / "results"
# HERE is sheaf-db/tests/e2e/realtime_bench.
SHEAF_DB = HERE.parents[2]
REPO = SHEAF_DB.parent


def _load_config() -> None:
    """Seed the environment from the machine config, when it exists."""
    p = Path(os.environ.get("SHEAF_BENCH_CONFIG") or Path.home() / ".sheaf_bench.json")
    if not p.exists():
        return
    try:
        cfg = json.loads(p.read_text(encoding="utf-8"))
    except Exception as e:  # a broken config must stop the run, not be ignored
        raise SystemExit(f"error: cannot read {p}: {e}")
    for key, value in cfg.items():
        os.environ.setdefault("SHEAF_" + key.upper(), str(value))


_load_config()


def _get(name: str) -> Path:
    value = os.environ.get(name)
    if not value:
        raise SystemExit(
            f"error: {name} is not set. Add it to {Path.home() / '.sheaf_bench.json'} "
            f"or pass the matching flag to run_all.py."
        )
    return Path(value)


def data(*parts: str) -> str:
    return str(_get("SHEAF_DATA").joinpath(*parts))


def build(*parts: str) -> str:
    return str(_get("SHEAF_BUILD").joinpath(*parts))


def datasets(*parts: str) -> str:
    return str(_get("SHEAF_DATASETS").joinpath(*parts))


def imdb(*parts: str) -> str:
    return str(_get("SHEAF_IMDB").joinpath(*parts))


def pg_app(*parts: str) -> str:
    return str(_get("SHEAF_PG_APP").joinpath(*parts))


def mariadb_app(*parts: str) -> str:
    return str(_get("SHEAF_MARIADB_APP").joinpath(*parts))


def redis_server() -> str:
    return str(_get("SHEAF_REDIS_SERVER"))


def mongod() -> str:
    return str(_get("SHEAF_MONGOD"))


def pgbench_data(*parts: str) -> str:
    return str(_get("SHEAF_PGBENCH_DATA").joinpath(*parts))
