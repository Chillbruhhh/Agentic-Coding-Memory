@echo off
echo Creating minimal icons for Tauri desktop app...

cd /d "%~dp0\..\amp\ui\src-tauri"

REM Create icons directory
if not exist "icons" mkdir icons

REM Create a minimal 32x32 PNG as icon.png (Tauri can convert this)
echo Creating placeholder icons...

REM Use PowerShell to create a simple red square PNG
powershell -Command "Add-Type -AssemblyName System.Drawing; $bmp = New-Object System.Drawing.Bitmap(32,32); $g = [System.Drawing.Graphics]::FromImage($bmp); $g.Clear([System.Drawing.Color]::Red); $bmp.Save('icons\icon.png', [System.Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $bmp.Dispose()"

REM Generate all required icon sizes using Tauri CLI
npx @tauri-apps/cli icon icons\icon.png

echo ✅ Icons created successfully!
echo 🚀 Now run: npm run tauri:dev
