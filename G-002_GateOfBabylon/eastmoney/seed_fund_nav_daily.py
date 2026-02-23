import argparse
import asyncio
import datetime
import os
from decimal import Decimal, InvalidOperation
from typing import List, Tuple, Optional

import psycopg2
import yaml
from psycopg2.extras import execute_values

from util.eastmoney import FundNavQuery, get_fund_nav_raw_from_api


def load_yaml_config(path: str | None) -> dict:
    cfg_path = path or os.path.join(os.path.dirname(__file__), "config.yaml")
    if os.path.exists(cfg_path):
        with open(cfg_path, "r", encoding="utf-8") as f:
            return yaml.safe_load(f) or {}
    return {}

def get_conn(cfg: dict) -> psycopg2.extensions.connection:
    host = cfg.get("host") or "pgm-bp1z198qck54m3k0yo.pg.rds.aliyuncs.com"
    port = int(cfg.get("port") or "5432")
    user = cfg.get("user")
    password = cfg.get("password")
    dbname = cfg.get("database")
    if not (user and password and dbname):
        raise SystemExit("需要在配置文件中提供 host/port/user/password/database")
    return psycopg2.connect(host=host, port=port, user=user, password=password, dbname=dbname)

def ensure_schema(conn) -> None:
    schema_path = os.path.join(os.path.dirname(__file__), "..", "schema", "fund_nav_daily.sql")
    with open(schema_path, "r", encoding="utf-8") as f:
        ddl = f.read()
    with conn.cursor() as cur:
        cur.execute(ddl)
    conn.commit()

def fetch_fund_ids(conn) -> List[str]:
    with conn.cursor() as cur:
        cur.execute("SELECT fund_id FROM public.fund_info ORDER BY fund_id")
        rows = cur.fetchall()
    return [r[0] for r in rows]

def to_decimal(value: Optional[str]) -> Optional[Decimal]:
    if value is None:
        return None
    v = str(value).strip()
    if v == "":
        return None
    try:
        return Decimal(v)
    except InvalidOperation:
        return None

def build_rows(fund_id: str, items: List[dict]) -> List[Tuple[str, datetime.date, Optional[Decimal], Optional[Decimal]]]:
    rows = []
    for item in items:
        date_str = item.get("FSRQ")
        if not date_str:
            continue
        try:
            nav_date = datetime.date.fromisoformat(date_str)
        except ValueError:
            continue
        net_value = to_decimal(item.get("DWJZ"))
        accum_value = to_decimal(item.get("LJJZ"))
        if net_value is None and accum_value is None:
            continue
        rows.append((fund_id, nav_date, net_value, accum_value))
    return rows

def upsert_rows(conn, rows: List[Tuple[str, datetime.date, Optional[Decimal], Optional[Decimal]]]) -> int:
    if not rows:
        return 0
    sql = """
    INSERT INTO public.fund_nav_daily (fund_id, nav_date, net_asset_value, accumulated_asset_value)
    VALUES %s
    ON CONFLICT (fund_id, nav_date) DO UPDATE SET
      net_asset_value = EXCLUDED.net_asset_value,
      accumulated_asset_value = EXCLUDED.accumulated_asset_value
    """
    with conn.cursor() as cur:
        execute_values(cur, sql, rows, page_size=1000)
    conn.commit()
    return len(rows)

async def fetch_one(fund_id: str, start_date: datetime.date, end_date: datetime.date, semaphore: asyncio.Semaphore) -> Tuple[str, List[dict], Optional[str]]:
    async with semaphore:
        try:
            query = FundNavQuery(code=fund_id, start_date=start_date, end_date=end_date)
            data = await get_fund_nav_raw_from_api(query)
            return fund_id, data, None
        except Exception as e:
            return fund_id, [], str(e)

def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--config", default=None, help="YAML 配置文件路径，默认 eastmoney/config.yaml")
    p.add_argument("--start-date", default="2026-02-09")
    p.add_argument("--end-date", default="2026-02-13")
    p.add_argument("--batch-size", type=int, default=200)
    p.add_argument("--concurrency", type=int, default=20)
    return p.parse_args()

async def async_main():
    args = parse_args()
    cfg = load_yaml_config(args.config)
    start_date = datetime.date.fromisoformat(args.start_date)
    end_date = datetime.date.fromisoformat(args.end_date)
    conn = get_conn(cfg)
    try:
        ensure_schema(conn)
        fund_ids = fetch_fund_ids(conn)
        total_inserted = 0
        errors = 0
        for i in range(0, len(fund_ids), args.batch_size):
            batch = fund_ids[i:i + args.batch_size]
            semaphore = asyncio.Semaphore(args.concurrency)
            tasks = [fetch_one(fid, start_date, end_date, semaphore) for fid in batch]
            results = await asyncio.gather(*tasks)
            rows: List[Tuple[str, datetime.date, Optional[Decimal], Optional[Decimal]]] = []
            for fund_id, data, err in results:
                if err:
                    errors += 1
                    continue
                rows.extend(build_rows(fund_id, data))
            total_inserted += upsert_rows(conn, rows)
            print(f"batch {i // args.batch_size + 1}: fetched {len(batch)}, rows {len(rows)}, total_inserted {total_inserted}, errors {errors}")
        print(f"done: total_inserted {total_inserted}, errors {errors}")
    finally:
        conn.close()

if __name__ == "__main__":
    asyncio.run(async_main())
