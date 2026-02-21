import asyncio
import datetime
import math
from typing import Dict

import httpx


async def _fetch_page(
        client: httpx.AsyncClient,
        semaphore: asyncio.Semaphore,
        base_url: str,
        headers: Dict[str, str],
        code: str,
        page: int,
        page_size: int,
        s_date: str,
        e_date: str,
):
    params = {
        "fundCode": code,
        "pageIndex": page,
        "pageSize": page_size,
        "startDate": s_date,
        "endDate": e_date,
    }
    async with semaphore:
        response = await client.get(base_url, params=params, headers=headers)
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


async def get_fund_data_from_api(code: str, start_date: datetime.date, end_date: datetime.date):
    base_url = "https://api.fund.eastmoney.com/f10/lsjz"
    page_size = 20
    headers = {
        "Referer": "https://fund.eastmoney.com/",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    }

    s_date = start_date.strftime("%Y-%m-%d") if start_date else ""
    e_date = end_date.strftime("%Y-%m-%d") if end_date else ""

    debug_info = {}

    async with httpx.AsyncClient(timeout=10.0, headers=headers) as client:
        try:
            first_params = {
                "fundCode": code,
                "pageIndex": 1,
                "pageSize": page_size,
                "startDate": s_date,
                "endDate": e_date,
            }
            first_response = await client.get(base_url, params=first_params)
            first_response.raise_for_status()
            try:
                first_data = first_response.json()
            except ValueError as e:
                return None, f"响应数据不是有效的JSON: {e}", {"url": str(first_response.url)}

            err_code = first_data.get("ErrCode")
            err_msg = first_data.get("ErrMsg", "")
            debug_info = {
                "url": str(first_response.url),
                "params": first_params,
                "status_code": first_response.status_code,
                "err_code": err_code,
                "err_msg": err_msg,
            }

            if err_code is not None and err_code != 0:
                return None, f"API返回错误: {err_msg} (错误码: {err_code})", debug_info

            total_count = first_data.get("TotalCount", 0)
            if total_count == 0:
                return [], None, debug_info

            total_pages = math.ceil(total_count / page_size) if total_count else 0
            all_data = []
            all_data.extend(first_data.get("Data", {}).get("LSJZList") or [])

            if total_pages > 1:
                semaphore = asyncio.Semaphore(20)
                tasks = [
                    _fetch_page(client, semaphore, base_url, headers, code, page, page_size, s_date, e_date)
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
                    debug_info["page_errors"] = page_errors

            if not all_data:
                return None, "No data found.", debug_info

            return all_data, None, debug_info
        except httpx.HTTPError as e:
            return None, f"网络请求失败: {e}", debug_info


async def get_fund_name_from_api(code: str):
    url = f"https://fundsuggest.eastmoney.com/FundSearch/api/FundSearchAPI.ashx?m=1&key={code}"
    headers = {
        "Referer": "https://fund.eastmoney.com/",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    }
    async with httpx.AsyncClient(timeout=20.0, headers=headers) as client:
        try:
            response = await client.get(url)
            response.raise_for_status()
            try:
                payload = response.json()
            except ValueError as e:
                return None, f"响应数据不是有效的JSON: {e}", {"url": str(response.url)}
            err_code = payload.get("ErrCode")
            err_msg = payload.get("ErrMsg", "")
            debug_info = {
                "url": str(response.url),
                "status_code": response.status_code,
                "err_code": err_code,
                "err_msg": err_msg,
            }
            if err_code is not None and err_code != 0:
                return None, f"API返回错误: {err_msg} (错误码: {err_code})", debug_info
            records = payload.get("Datas") or []
            target = next(
                (item for item in records if str(item.get("CATEGORY")) == "700"),
                None
            )
            fund_name = target.get("NAME") if target else None
            return fund_name, None, debug_info
        except httpx.HTTPError as e:
            return None, f"网络请求失败: {e}", {"url": url}
