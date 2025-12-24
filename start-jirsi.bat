@echo off
echo ========================================
echo  Starting Jirsi Platform
echo ========================================
echo.

REM Start Docker Desktop if not running
echo [1/5] Checking Docker Desktop...
docker info >nul 2>&1
if errorlevel 1 (
    echo Docker Desktop not running. Starting...
    start "" "C:\Program Files\Docker\Docker\Docker Desktop.exe"
    echo Waiting for Docker to start...
    timeout /t 15 /nobreak >nul
    
    REM Wait up to 60 seconds for Docker to be ready
    set /a counter=0
    :wait_docker
    docker info >nul 2>&1
    if errorlevel 1 (
        set /a counter+=1
        if %counter% LSS 12 (
            timeout /t 5 /nobreak >nul
            goto wait_docker
        ) else (
            echo ERROR: Docker failed to start. Please start Docker Desktop manually.
            pause
            exit /b 1
        )
    )
    echo Docker Desktop is ready!
) else (
    echo Docker Desktop is already running!
)

REM Start PostgreSQL container
echo.
echo [2/5] Starting PostgreSQL container...
docker start saas-postgres
if errorlevel 1 (
    echo ERROR: Failed to start PostgreSQL container.
    pause
    exit /b 1
)
echo PostgreSQL container started!

REM Start Backend Server in WSL
echo.
echo [3/5] Starting Backend Server in WSL...
start "Jirsi Backend" wsl bash -c "cd '/mnt/e/s_programmer/Saas System' && source ~/.cargo/env && export DATABASE_URL='postgres://postgres@172.29.208.1:15432/saas' && cargo run --bin server"
timeout /t 3 /nobreak >nul

REM Start Frontend Server in WSL
echo.
echo [4/5] Starting Frontend Server in WSL...
start "Jirsi Frontend" wsl bash -c "cd '/mnt/e/s_programmer/Saas System/crates/frontend-web' && source ~/.cargo/env && trunk serve --port 8104 --address 0.0.0.0"
echo Waiting for services to initialize...
timeout /t 10 /nobreak >nul

REM Open Browser
echo.
echo [5/5] Opening Browser...
start http://localhost:8104/app/crm/entity/contact

echo.
echo ========================================
echo  Jirsi Platform Started Successfully!
echo ========================================
echo.
echo Backend:  Running in WSL (check "Jirsi Backend" window)
echo Frontend: Running in WSL (check "Jirsi Frontend" window)
echo Browser:  http://localhost:8104/app/crm/entity/contact
echo.
echo Press any key to close this window (servers will keep running)
pause >nul
