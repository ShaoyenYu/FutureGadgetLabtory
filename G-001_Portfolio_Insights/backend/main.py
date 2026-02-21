import os

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles

from api.fund_data import router as fund_router
from api.fund_info import router as fund_name_router

app = FastAPI()

# Add CORS middleware to allow requests from any origin (useful for development)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(fund_router)
app.include_router(fund_name_router)
frontend_path = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "frontend-vue", "dist")

if os.path.exists(frontend_path):
    app.mount("/", StaticFiles(directory=frontend_path, html=True), name="frontend")
else:
    print(f"Warning: Frontend directory not found at {frontend_path}")

if __name__ == "__main__":
    import uvicorn
    log_config = os.path.join(os.path.dirname(os.path.abspath(__file__)), "logging.json")
    uvicorn.run(app, host="0.0.0.0", port=8000, log_config=log_config, log_level="info")
