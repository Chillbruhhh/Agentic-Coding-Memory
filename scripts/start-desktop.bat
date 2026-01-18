@echo off
echo 🚀 Starting AMP Desktop Application...

REM Navigate to UI directory
cd /d "%~dp0\..\amp\ui"

REM Check if node_modules exists
if not exist "node_modules" (
    echo 📦 Installing dependencies...
    npm install
)

REM Start the desktop app
echo 🖥️  Launching desktop app...
npm run tauri:dev
