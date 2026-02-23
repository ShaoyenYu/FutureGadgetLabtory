import argparse
import asyncio
import os
from typing import List, Tuple

import psycopg2
import yaml
from psycopg2.extras import execute_values

from util.eastmoney import fetch_all_pairs

DDL_STMTS = [
    """
    CREATE TABLE IF NOT EXISTS public.fund_info (
      fund_id VARCHAR(16) PRIMARY KEY,
      fund_name VARCHAR(64) NOT NULL,
      _create_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      _update_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
    """,
    """
    CREATE OR REPLACE FUNCTION public.set_fund_info_updated_timestamp()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
    BEGIN
      NEW._update_timestamp := NOW();
      RETURN NEW;
    END;
    $$;
    """,
    "DROP TRIGGER IF EXISTS fund_info_set_updated ON public.fund_info;",
    """
    CREATE TRIGGER fund_info_set_updated
    BEFORE UPDATE ON public.fund_info
    FOR EACH ROW
    EXECUTE FUNCTION public.set_fund_info_updated_timestamp();
    """,
]

def ensure_schema(conn, tz: str = "UTC") -> None:
    with conn.cursor() as cur:
        cur.execute("SET TIME ZONE %s;", (tz,))
        cur.execute("SET CLIENT_ENCODING TO 'UTF8';")
        for s in DDL_STMTS:
            cur.execute(s)
    conn.commit()

def upsert_funds(conn, pairs: List[Tuple[str, str]]) -> int:
    sql = """
    INSERT INTO public.fund_info (fund_id, fund_name)
    VALUES %s
    ON CONFLICT (fund_id) DO UPDATE SET fund_name = EXCLUDED.fund_name
    """
    with conn.cursor() as cur:
        execute_values(cur, sql, pairs, page_size=2000)
    conn.commit()
    return len(pairs)

def load_yaml_config(path: str | None) -> dict:
    cfg_path = path or os.path.join(os.path.dirname(__file__), "config.yaml")
    if os.path.exists(cfg_path):
        with open(cfg_path, "r", encoding="utf-8") as f:
            return yaml.safe_load(f) or {}
    return {}

def get_conn(args) -> psycopg2.extensions.connection:
    ycfg = load_yaml_config(args.config)
    host = ycfg.get("host") or "pgm-bp1z198qck54m3k0yo.pg.rds.aliyuncs.com"
    port = int(ycfg.get("port") or "5432")
    user = ycfg.get("user")
    password = ycfg.get("password")
    dbname = ycfg.get("database")
    if not (user and password and dbname):
        raise SystemExit("需要在配置文件中提供 host/port/user/password/database")
    return psycopg2.connect(host=host, port=port, user=user, password=password, dbname=dbname)

def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--config", default=None, help="YAML 配置文件路径，默认 eastmoney/config.yaml")
    return p.parse_args()

async def async_main():
    pairs_map = await fetch_all_pairs()
    pairs = sorted(pairs_map.items(), key=lambda x: x[0])
    args = parse_args()
    ycfg = load_yaml_config(args.config)
    tz = ycfg.get("timezone") or "UTC"
    conn = get_conn(args)
    try:
        ensure_schema(conn, tz=tz)
        n = upsert_funds(conn, pairs)
        print(f"upsert {n} rows into public.fund_info")
    finally:
        conn.close()

if __name__ == "__main__":
    asyncio.run(async_main())
