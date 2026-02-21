import asyncio
from typing import List

from fastapi import APIRouter, HTTPException
from fastapi.responses import JSONResponse
from pydantic import BaseModel

from util.eastmoney import get_fund_name_from_api

router = APIRouter()


class FundNameBatchRequest(BaseModel):
    codes: List[str]


@router.get("/api/fund-name/{code}")
async def get_fund_name(code: str):
    cache_headers = {"Cache-Control": "public, max-age=86400"}
    fund_name, error, debug_info = await get_fund_name_from_api(code)
    if error:
        return JSONResponse(status_code=500, content={"error": str(error), "debug_info": debug_info})
    if not fund_name:
        return JSONResponse(status_code=404, content={"error": "No fund name found", "debug_info": debug_info})
    return JSONResponse(
        content={"fund_code": code, "fund_name": fund_name, "debug_info": debug_info},
        headers=cache_headers,
    )


@router.post("/api/fund-name/batch")
async def get_fund_name_batch(request: FundNameBatchRequest):
    cache_headers = {"Cache-Control": "public, max-age=86400"}
    codes = [code for code in request.codes if code]
    if not codes:
        raise HTTPException(status_code=400, detail="Codes are required.")
    tasks = [get_fund_name_from_api(code) for code in codes]
    results = await asyncio.gather(*tasks, return_exceptions=True)
    names = {}
    errors = []
    debug_info = {}
    for code, result in zip(codes, results):
        if isinstance(result, Exception):
            errors.append({"code": code, "error": str(result)})
            continue
        fund_name, error, info = result
        debug_info[code] = info
        if error:
            errors.append({"code": code, "error": str(error)})
            continue
        if fund_name:
            names[code] = fund_name
        else:
            errors.append({"code": code, "error": "No fund name found"})
    return JSONResponse(content={"names": names, "errors": errors, "debug_info": debug_info}, headers=cache_headers)
