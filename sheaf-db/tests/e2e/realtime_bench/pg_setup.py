"""One-shot: create bench db + password on local PG."""
import psycopg

c = psycopg.connect(host="127.0.0.1", port=5433, user="postgres", dbname="postgres", autocommit=True)
c.execute("ALTER USER postgres PASSWORD 'benchpass'")
c.execute("CREATE DATABASE bench")
print("bench db + password ready")
