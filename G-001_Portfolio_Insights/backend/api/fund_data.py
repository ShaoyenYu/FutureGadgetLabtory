import asyncio
import datetime
from typing import List, Optional

from fastapi import APIRouter, HTTPException
from fastapi.responses import JSONResponse
from pydantic import BaseModel

from util.eastmoney import get_fund_data_from_api

router = APIRouter()


class PortfolioItem(BaseModel):
    code: str
    shares: float


class PortfolioRequest(BaseModel):
    items: List[PortfolioItem]
    start_date: Optional[str] = None
    end_date: Optional[str] = None


@router.get("/api/fund/{code}")
async def get_fund_data(
        code: str,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
):
    try:
        s_date = datetime.datetime.strptime(start_date, "%Y-%m-%d").date() if start_date else None
        e_date = datetime.datetime.strptime(end_date, "%Y-%m-%d").date() if end_date else None
    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid date format. Use YYYY-MM-DD.")

    if not s_date:
        s_date = datetime.date.today() - datetime.timedelta(days=365)
    if not e_date:
        e_date = datetime.date.today()
    today = datetime.date.today()
    if e_date > today:
        e_date = today
    if s_date > e_date:
        raise HTTPException(status_code=400, detail="Start date cannot be later than end date.")

    data_list, error, debug_info = await get_fund_data_from_api(code, s_date, e_date)

    if error:
        return JSONResponse(status_code=500, content={"error": str(error), "debug_info": debug_info})

    if not data_list:
        return JSONResponse(status_code=404, content={"error": "No data found", "debug_info": debug_info})

    processed_data = []
    for item in data_list:
        try:
            val = item.get("DWJZ")
            if val:
                processed_data.append(
                    {
                        "date": item["FSRQ"],
                        "value": float(val),
                        "cumulative_value": float(item.get("LJJZ", 0)),
                    }
                )
        except (ValueError, KeyError):
            continue

    processed_data.sort(key=lambda x: x["date"])

    return {"fund_code": code, "data": processed_data, "debug_info": debug_info}


@router.post("/api/portfolio")
async def get_portfolio_data(request: PortfolioRequest):
    filtered_items = [item for item in request.items if item.code and item.shares and item.shares > 0]
    if not filtered_items:
        raise HTTPException(status_code=400, detail="Portfolio items are required.")

    try:
        s_date = datetime.datetime.strptime(request.start_date, "%Y-%m-%d").date() if request.start_date else None
        e_date = datetime.datetime.strptime(request.end_date, "%Y-%m-%d").date() if request.end_date else None
    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid date format. Use YYYY-MM-DD.")

    if not s_date:
        s_date = datetime.date.today() - datetime.timedelta(days=365)
    if not e_date:
        e_date = datetime.date.today()
    today = datetime.date.today()
    if e_date > today:
        e_date = today
    if s_date > e_date:
        raise HTTPException(status_code=400, detail="Start date cannot be later than end date.")

    tasks = [get_fund_data_from_api(item.code, s_date, e_date) for item in filtered_items]
    results = await asyncio.gather(*tasks, return_exceptions=True)

    fund_series = []
    debug_info = {}
    errors = []

    for item, result in zip(filtered_items, results):
        if isinstance(result, Exception):
            errors.append({"code": item.code, "error": str(result)})
            continue
        data_list, error, fund_debug = result
        debug_info[item.code] = fund_debug
        if error:
            errors.append({"code": item.code, "error": str(error)})
            continue
        if not data_list:
            errors.append({"code": item.code, "error": "No data found"})
            continue
        processed = []
        for entry in data_list:
            val = entry.get("DWJZ")
            cumulative_val = entry.get("LJJZ")
            date_str = entry.get("FSRQ")
            if not val or not date_str:
                continue
            try:
                value = float(val)
                cumulative_value = float(cumulative_val) if cumulative_val not in (None, "") else None
            except ValueError:
                continue
            processed.append(
                {
                    "date": date_str,
                    "value": value,
                    "cumulative_value": cumulative_value,
                    "amount": value * item.shares,
                }
            )
        processed.sort(key=lambda x: x["date"])
        fund_series.append({"code": item.code, "shares": item.shares, "data": processed})

    if not fund_series and errors:
        only_no_data = all(error.get("error") == "No data found" for error in errors)
        if only_no_data:
            return JSONResponse(
                status_code=404,
                content={"error": "No data found for requested date range", "details": errors,
                         "debug_info": debug_info},
            )
        return JSONResponse(
            status_code=502,
            content={"error": "Portfolio fetch failed", "details": errors, "debug_info": debug_info},
        )

    date_sets = [set(item["date"] for item in fund["data"]) for fund in fund_series]
    all_dates = sorted(set.union(*date_sets)) if date_sets else []
    if not all_dates:
        return JSONResponse(status_code=404,
                            content={"error": "No dates found", "debug_info": debug_info, "details": errors})

    fund_value_maps = {fund["code"]: {entry["date"]: entry for entry in fund["data"]} for fund in fund_series}

    portfolio_series = []
    for date_str in all_dates:
        total_value = 0.0
        performance_value = 0.0
        for fund in fund_series:
            entry = fund_value_maps[fund["code"]].get(date_str)
            if entry:
                total_value += entry["amount"]
                if entry["cumulative_value"] is not None:
                    performance_value += entry["cumulative_value"] * fund["shares"]
        portfolio_series.append(
            {"date": date_str, "total_value": total_value, "performance_value": performance_value}
        )

    base_value = portfolio_series[0]["performance_value"] if portfolio_series else 0.0
    base_total_value = portfolio_series[0]["total_value"] if portfolio_series else 0.0
    for entry in portfolio_series:
        entry["normalized_value"] = entry["performance_value"] / base_value if base_value else 0.0
        entry["normalized_total_value"] = entry["total_value"] / base_total_value if base_total_value else 0.0

    response_payload = {
        "portfolio": {
            "start_date": s_date.strftime("%Y-%m-%d"),
            "end_date": e_date.strftime("%Y-%m-%d"),
            "base_value": base_value,
            "base_total_value": base_total_value,
            "data": portfolio_series,
        },
        "funds": fund_series,
        "debug_info": debug_info,
    }
    if errors:
        response_payload["warnings"] = errors
    return response_payload
