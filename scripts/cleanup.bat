@echo off
echo ===============================================
echo Mana Panel - Process Cleanup Utility (Windows)
echo ===============================================
echo.

echo Searching for mana-panel-back processes...
tasklist /FI "IMAGENAME eq mana-panel-back.exe" 2>NUL | find /I /N "mana-panel-back.exe">NUL

if "%ERRORLEVEL%"=="0" (
    echo Found running processes. Terminating...
    taskkill /F /IM mana-panel-back.exe
    if %ERRORLEVEL% EQU 0 (
        echo [SUCCESS] All mana-panel-back processes have been terminated.
    ) else (
        echo [ERROR] Failed to terminate some processes. Try running as Administrator.
    )
) else (
    echo [INFO] No mana-panel-back processes found.
)

echo.
echo Also checking for cargo processes...
tasklist /FI "IMAGENAME eq cargo.exe" 2>NUL | find /I /N "cargo.exe">NUL

if "%ERRORLEVEL%"=="0" (
    echo Found cargo processes. You may want to close your terminal running 'pnpm dev'.
    choice /C YN /M "Do you want to kill all cargo processes"
    if errorlevel 2 goto :skip_cargo
    if errorlevel 1 (
        taskkill /F /IM cargo.exe
        echo Cargo processes terminated.
    )
)

:skip_cargo
echo.
echo Cleanup complete!
echo You can now run: pnpm run dev
echo.
pause
