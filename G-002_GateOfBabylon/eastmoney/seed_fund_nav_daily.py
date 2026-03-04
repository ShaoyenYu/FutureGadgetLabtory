import argparse
import asyncio
import datetime
import datetime as dt
import logging
import os
import threading
from concurrent.futures import ThreadPoolExecutor, as_completed, wait
from decimal import Decimal, InvalidOperation
from typing import List, Tuple, Optional

import psycopg2
import yaml
from psycopg2.extras import execute_values
from pydantic import BaseModel

from util.eastmoney import FundNavQuery, get_fund_nav_raw_from_api

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s", datefmt="%Y-%m-%d %H:%M:%S")
logging.getLogger("httpx").setLevel(logging.WARNING)
logging.getLogger("httpcore").setLevel(logging.WARNING)

class FundNavRow(BaseModel):
    fund_id: str
    nav_date: datetime.date
    net_asset_value: Optional[Decimal] = None
    accumulated_asset_value: Optional[Decimal] = None

class FetchBatchResult(BaseModel):
    rows: List[FundNavRow]
    errors: int
    updated_funds: int

class WriteTaskMeta(BaseModel):
    batch_idx: int
    batch_size: int
    row_count: int
    updated_funds: int


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

def build_rows(fund_id: str, items: List[dict]) -> List[FundNavRow]:
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
        rows.append(
            FundNavRow(
                fund_id=fund_id,
                nav_date=nav_date,
                net_asset_value=net_value,
                accumulated_asset_value=accum_value,
            )
        )
    return rows

def to_db_values(rows: List[FundNavRow]) -> List[Tuple[str, datetime.date, Optional[Decimal], Optional[Decimal]]]:
    return [(r.fund_id, r.nav_date, r.net_asset_value, r.accumulated_asset_value) for r in rows]

def upsert_rows(conn, rows: List[FundNavRow]) -> int:
    if not rows:
        return 0
    sql = """
    INSERT INTO public.fund_nav (fund_id, date, net_asset_value, accumulated_asset_value)
    VALUES %s
    ON CONFLICT (fund_id, date) DO UPDATE SET
      net_asset_value = EXCLUDED.net_asset_value,
      accumulated_asset_value = EXCLUDED.accumulated_asset_value
    """
    values = to_db_values(rows)
    with conn.cursor() as cur:
        execute_values(cur, sql, values, page_size=1000)
    conn.commit()
    return len(rows)

def upsert_rows_with_new_conn(cfg: dict, rows: List[FundNavRow]) -> int:
    if not rows:
        return 0
    conn = get_conn(cfg)
    try:
        return upsert_rows(conn, rows)
    finally:
        conn.close()

async def upsert_rows_async(cfg: dict, rows: List[FundNavRow], semaphore: asyncio.Semaphore) -> int:
    async with semaphore:
        return await asyncio.to_thread(upsert_rows_with_new_conn, cfg, rows)

def chunk_rows(rows: List[FundNavRow], size: int) -> List[List[FundNavRow]]:
    return [rows[i:i + size] for i in range(0, len(rows), size)]

async def fetch_one(fund_id: str, start_date: datetime.date, end_date: datetime.date, semaphore: asyncio.Semaphore) -> Tuple[str, List[dict], Optional[str]]:
    async with semaphore:
        try:
            query = FundNavQuery(code=fund_id, start_date=start_date, end_date=end_date)
            data = await get_fund_nav_raw_from_api(query)
            return fund_id, data, None
        except Exception as e:
            return fund_id, [], str(e)

async def fetch_batch_async(batch: List[str], start_date: datetime.date, end_date: datetime.date, concurrency: int) -> FetchBatchResult:
    semaphore = asyncio.Semaphore(concurrency)
    tasks = [fetch_one(fid, start_date, end_date, semaphore) for fid in batch]
    results = await asyncio.gather(*tasks)
    rows: List[FundNavRow] = []
    errors = 0
    updated_funds = 0
    for fund_id, data, err in results:
        if err:
            errors += 1
            continue
        fund_rows = build_rows(fund_id, data)
        if fund_rows:
            updated_funds += 1
            rows.extend(fund_rows)
    return FetchBatchResult(rows=rows, errors=errors, updated_funds=updated_funds)

def fetch_batch_worker(batch: List[str], start_date: datetime.date, end_date: datetime.date, concurrency: int) -> FetchBatchResult:
    return asyncio.run(fetch_batch_async(batch, start_date, end_date, concurrency))

async def write_rows_async(cfg: dict, rows: List[FundNavRow], write_concurrency: int, write_chunk_size: int) -> int:
    write_semaphore = asyncio.Semaphore(write_concurrency)
    row_chunks = chunk_rows(rows, write_chunk_size)
    inserted_parts = await asyncio.gather(*[upsert_rows_async(cfg, chunk, write_semaphore) for chunk in row_chunks])
    return sum(inserted_parts)

def write_rows_worker(cfg: dict, rows: List[FundNavRow], write_concurrency: int, write_chunk_size: int) -> int:
    return asyncio.run(write_rows_async(cfg, rows, write_concurrency, write_chunk_size))

def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--config", default=None, help="YAML 配置文件路径，默认 eastmoney/config.yaml")
    p.add_argument("--start-date", default=f"{dt.date.today() - dt.timedelta(days=7)}")
    p.add_argument("--end-date", default=f"{dt.date.today()}")
    p.add_argument("--batch-size", type=int, default=200)
    p.add_argument("--concurrency", type=int, default=20)
    p.add_argument("--write-concurrency", type=int, default=5)
    p.add_argument("--write-chunk-size", type=int, default=2000)
    p.add_argument("--workers", type=int, default=6)
    return p.parse_args()

def main():
    args = parse_args()
    cfg = load_yaml_config(args.config)
    start_date = datetime.date.fromisoformat(args.start_date)
    end_date = datetime.date.fromisoformat(args.end_date)
    total_inserted = 0
    total_updated_funds = 0
    errors = 0
    conn = get_conn(cfg)
    try:
        ensure_schema(conn)
        fund_ids = fetch_fund_ids(conn)
        fetch_batches = [
            fund_ids[i:i + args.batch_size]
            for i in range(0, len(fund_ids), args.batch_size)
        ]
        with ThreadPoolExecutor(max_workers=args.workers) as worker_pool:
            lock = threading.Lock()
            write_future_to_meta = {}

            def on_fetch_done(fetch_future, batch_idx: int, batch_size: int):
                nonlocal errors
                try:
                    fetch_result = fetch_future.result()
                except Exception as e:
                    with lock:
                        errors += batch_size
                    logging.info(f"batch {batch_idx}: fetch failed, fetched {batch_size}, rows 0, total_inserted {total_inserted}, errors {errors}, reason {e}")
                    return
                rows = fetch_result.rows
                batch_errors = fetch_result.errors
                updated_funds = fetch_result.updated_funds
                with lock:
                    errors += batch_errors
                logging.info(f"batch {batch_idx}: write preview rows={rows[:1]}")
                write_future = worker_pool.submit(write_rows_worker, cfg, rows, args.write_concurrency, args.write_chunk_size)
                with lock:
                    write_future_to_meta[write_future] = WriteTaskMeta(
                        batch_idx=batch_idx,
                        batch_size=batch_size,
                        row_count=len(rows),
                        updated_funds=updated_funds,
                    )

            fetch_futures = []
            for idx, batch in enumerate(fetch_batches, start=1):
                fetch_future = worker_pool.submit(fetch_batch_worker, batch, start_date, end_date, args.concurrency)
                fetch_future.add_done_callback(lambda f, batch_idx=idx, batch_size=len(batch): on_fetch_done(f, batch_idx, batch_size))
                fetch_futures.append(fetch_future)

            wait(fetch_futures)

            write_futures = list(write_future_to_meta.keys())
            for write_future in as_completed(write_futures):
                meta = write_future_to_meta[write_future]
                try:
                    inserted = write_future.result()
                except Exception as e:
                    logging.info(f"batch {meta.batch_idx}: write failed, fetched {meta.batch_size}, rows {meta.row_count}, total_inserted {total_inserted}, errors {errors}, reason {e}")
                    continue
                total_inserted += inserted
                total_updated_funds += meta.updated_funds
                logging.info(f"batch {meta.batch_idx}: fetched {meta.batch_size}, rows {meta.row_count}, total_inserted {total_inserted}, errors {errors}")
                logging.info(f"after_write: cumulative_updated_funds={total_updated_funds}, cumulative_written_rows={total_inserted}, errors={errors}")
    finally:
        logging.info(f"done: cumulative_updated_funds={total_updated_funds}, cumulative_written_rows={total_inserted}, errors={errors}")
        conn.close()

if __name__ == "__main__":
    main()
