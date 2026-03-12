@echo off
echo Starting WatchMark Tauri 2.0 Environment...
cd watchmark-tauri
call npm install
call npm run tauri dev
pause
