@echo off
setlocal enabledelayedexpansion

echo ===================================================
echo       WatchMark - Production Build Script
echo ===================================================
echo.

:: Navigate to watchmark-tauri directory
cd /d "%~dp0watchmark-tauri"

:: Check for Node.js
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Node.js is not installed or not in PATH!
    echo Please install Node.js from https://nodejs.org/
    pause
    exit /b 1
)

:: Check for Rust / Cargo
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust / Cargo is not installed or not in PATH!
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

:: Install dependencies if node_modules missing
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

:: Run production build
echo.
echo [2/3] Building frontend and compiling release executable...
echo (This may take a few minutes for Rust optimization and LTO...)
call npm run tauri build

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Build failed! Check the error log above.
    pause
    exit /b %errorlevel%
)

:: Find and report output executable
echo.
echo [3/3] Build succeeded!
echo ===================================================

set "RELEASE_DIR=%~dp0watchmark-tauri\src-tauri\target\release"
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

:: Copy standalone executable to a top-level dist-release folder for convenient access
set "OUT_DIR=%~dp0dist-release"
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
