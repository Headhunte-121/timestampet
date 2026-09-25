@echo off
setlocal enabledelayedexpansion

echo ===================================================
echo       WatchMark - Production Build Script
echo ===================================================
echo.

cd /d "%~dp0"

where node >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Node.js is not installed or not in PATH!
    pause
    exit /b 1
)

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust / Cargo is not installed or not in PATH!
    pause
    exit /b 1
)

if not exist "node_modules\" (
    echo [1/3] Installing NPM dependencies...
    call npm install
    if %errorlevel% neq 0 (
        echo [ERROR] npm install failed!
        pause
        exit /b %errorlevel%
    )
) else (
    echo [1/3] NPM dependencies already installed.
)

echo.
echo [2/3] Building frontend and compiling release executable...
call npm run tauri build

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Build failed! Check the error log above.
    pause
    exit /b %errorlevel%
)

echo.
echo [3/3] Build succeeded!
echo ===================================================

set "RELEASE_DIR=%~dp0src-tauri\target\release"
set "BUNDLE_DIR=%RELEASE_DIR%\bundle"

echo.
echo Executables created:
if exist "%RELEASE_DIR%\watchmark-tauri.exe" (
    echo  - Standalone EXE: %RELEASE_DIR%\watchmark-tauri.exe
)
if exist "%BUNDLE_DIR%\nsis\" (
    for %%F in ("%BUNDLE_DIR%\nsis\*.exe") do (
        echo  - NSIS Installer: %%F
    )
)
if exist "%BUNDLE_DIR%\msi\" (
    for %%F in ("%BUNDLE_DIR%\msi\*.msi") do (
        echo  - MSI Installer: %%F
    )
)

set "OUT_DIR=%~dp0..\dist-release"
if not exist "%OUT_DIR%" mkdir "%OUT_DIR%"
if exist "%RELEASE_DIR%\watchmark-tauri.exe" (
    copy /y "%RELEASE_DIR%\watchmark-tauri.exe" "%OUT_DIR%\WatchMark.exe" >nul
    echo.
    echo [READY] Standalone executable copied to:
    echo   %OUT_DIR%\WatchMark.exe
)

echo.
echo ===================================================
echo Build completed successfully!
pause
