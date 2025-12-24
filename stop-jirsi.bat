@echo off
echo Stopping Jirsi Platform...

REM Stop frontend (if running)
echo Stopping Frontend...
wsl bash -c "pkill -f 'trunk serve'" 2>nul

REM Stop backend (if running)  
echo Stopping Backend...
wsl bash -c "pkill -f 'cargo run --bin server'" 2>nul

REM Stop PostgreSQL container (optional - comment out if you want to keep it running)
REM echo Stopping PostgreSQL...
REM docker stop saas-postgres

echo.
echo Jirsi Platform stopped!
pause
