import asyncio
import datetime
import json
import math
import re
from typing import Dict, List, Tuple, Any, Optional

import httpx
from pydantic import BaseModel

RANK_URL = "http://fund.eastmoney.com/data/rankhandler.aspx?op=ph&dt=kf&ft={ft}&rs=&gs=0&sc=zzf&st=desc&pi=1&pn=30000&dx=1"
HEADERS = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
    "Referer": "https://fund.eastmoney.com/fund.html",
}

NAV_BASE_URL = "https://api.fund.eastmoney.com/f10/lsjz"
NAV_HEADERS = {
    "Referer": "https://fund.eastmoney.com/",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
}

class FundNavQuery(BaseModel):
    code: str
    start_date: Optional[datetime.date] = None
    end_date: Optional[datetime.date] = None
    page_size: int = 20
    max_concurrency: int = 20

class FundNavValue(BaseModel):
    net_asset_value: Optional[str] = None
    accumulated_asset_value: Optional[str] = None

class FundNavResult(BaseModel):
    data: List[FundNavValue]

def _model_validate(model_cls, data):
    if hasattr(model_cls, "model_validate"):
        return model_cls.model_validate(data)
    return model_cls.parse_obj(data)

def pick_name(fields: List[str]) -> str:
    if len(fields) < 2:
        return ""
    for i in range(1, min(len(fields), 6)):
        v = fields[i]
        if re.search(r"[\u4e00-\u9fa5]", v):
            return v
    return fields[1]

async def fetch_pairs(client: httpx.AsyncClient, ft: str) -> List[Tuple[str, str]]:
    url = RANK_URL.format(ft=ft)
    r = await client.get(url, headers=HEADERS, timeout=30)
    r.raise_for_status()
    content = r.text
    m = re.search(r"datas:\[(.*?)\]", content, re.DOTALL)
    if not m:
        return []
    ds = m.group(1)
    try:
        arr = json.loads(f"[{ds}]")
    except json.JSONDecodeError:
        arr = re.findall(r'"(.*?)"', ds)
    out: List[Tuple[str, str]] = []
    for item in arr:
        parts = item.split(",")
        if not parts:
            continue
        code = parts[0]
        name = pick_name(parts)
        if code and name:
            out.append((code, name))
    return out

async def fetch_all_pairs() -> Dict[str, str]:
    fts = ["gp", "hh", "zq", "zs", "qdii", "lof", "fof"]
    async with httpx.AsyncClient() as client:
        results = await asyncio.gather(*(fetch_pairs(client, ft) for ft in fts), return_exceptions=True)
    data: Dict[str, str] = {}
    for r in results:
        if isinstance(r, Exception):
            continue
        for code, name in r:
            if code not in data:
                data[code] = name
    if not data:
        async with httpx.AsyncClient() as client:
            fallback = await fetch_pairs(client, "all")
        for code, name in fallback:
            if code not in data:
                data[code] = name
    return data

async def _fetch_nav_page(
    client: httpx.AsyncClient,
    semaphore: asyncio.Semaphore,
    query: FundNavQuery,
    page: int,
    s_date: str,
    e_date: str,
) -> Dict[str, Any]:
    params = {
        "fundCode": query.code,
        "pageIndex": page,
        "pageSize": query.page_size,
        "startDate": s_date,
        "endDate": e_date,
    }
    async with semaphore:
        response = await client.get(NAV_BASE_URL, params=params, headers=NAV_HEADERS)
        response.raise_for_status()
        try:
            data = response.json()
        except ValueError as e:
            raise ValueError(f"响应数据不是有效的JSON: {e}")
        err_code = data.get("ErrCode")
        if err_code is not None and err_code != 0:
            err_msg = data.get("ErrMsg", "未知错误")
            raise ValueError(f"API返回错误 (第{page}页): {err_msg} (错误码: {err_code})")
        return data

async def _collect_nav_list(query: FundNavQuery) -> List[Dict[str, Any]]:
    s_date = query.start_date.strftime("%Y-%m-%d") if query.start_date else ""
    e_date = query.end_date.strftime("%Y-%m-%d") if query.end_date else ""
    async with httpx.AsyncClient(timeout=10.0, headers=NAV_HEADERS) as client:
        try:
            first_params = {
                "fundCode": query.code,
                "pageIndex": 1,
                "pageSize": query.page_size,
                "startDate": s_date,
                "endDate": e_date,
            }
            first_response = await client.get(NAV_BASE_URL, params=first_params)
            first_response.raise_for_status()
            try:
                first_data = first_response.json()
            except ValueError as e:
                raise ValueError(f"响应数据不是有效的JSON: {e}")
            err_code = first_data.get("ErrCode")
            err_msg = first_data.get("ErrMsg", "")
            if err_code is not None and err_code != 0:
                raise ValueError(f"API返回错误: {err_msg} (错误码: {err_code})")
            total_count = first_data.get("TotalCount", 0)
            if total_count == 0:
                return []
            total_pages = math.ceil(total_count / query.page_size) if total_count else 0
            all_data: List[Dict[str, Any]] = []
            all_data.extend(first_data.get("Data", {}).get("LSJZList") or [])
            if total_pages > 1:
                semaphore = asyncio.Semaphore(query.max_concurrency)
                tasks = [
                    _fetch_nav_page(client, semaphore, query, page, s_date, e_date)
                    for page in range(2, total_pages + 1)
                ]
                results = await asyncio.gather(*tasks, return_exceptions=True)
                page_errors = []
                for result in results:
                    if isinstance(result, Exception):
                        page_errors.append(str(result))
                        continue
                    all_data.extend(result.get("Data", {}).get("LSJZList") or [])
                if page_errors:
                    raise ValueError("; ".join(page_errors))
            if not all_data:
                raise ValueError("No data found.")
            return all_data
        except httpx.HTTPError as e:
            raise ValueError(f"网络请求失败: {e}")

async def get_fund_nav_data_from_api(query: FundNavQuery) -> FundNavResult:
    all_data = await _collect_nav_list(query)
    records = [
        _model_validate(
            FundNavValue,
            {
                "net_asset_value": item.get("DWJZ"),
                "accumulated_asset_value": item.get("LJJZ"),
            },
        )
        for item in all_data
    ]
    return FundNavResult(data=records)

async def get_fund_nav_raw_from_api(query: FundNavQuery) -> List[Dict[str, Any]]:
    return await _collect_nav_list(query)
