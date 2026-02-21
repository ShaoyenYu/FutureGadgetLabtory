@echo off
echo Installing Backend Dependencies...
cd backend
pip install -r requirements.txt
if %errorlevel% neq 0 (
    echo Failed to install backend dependencies.
    pause
    exit /b %errorlevel%
)
echo Starting Backend...
start "Backend" cmd /k "python -m uvicorn main:app --reload --host 0.0.0.0 --port 8000 --log-level info --log-config logging.json"
cd ..

echo Installing Frontend Dependencies...
cd frontend-vue
call npm install
if %errorlevel% neq 0 (
    echo Failed to install frontend dependencies.
    pause
    exit /b %errorlevel%
)
echo Starting Frontend...
start "Frontend" cmd /k "npm run dev"
cd ..

echo Services started. Check the new windows for logs.
pause
