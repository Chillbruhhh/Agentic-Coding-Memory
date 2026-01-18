#!/bin/bash

# Create a simple 1024x1024 PNG using ImageMagick or convert
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/ui

# Create a simple red square PNG using convert (if available)
if command -v convert &> /dev/null; then
    convert -size 1024x1024 xc:red app-icon.png
    echo "✅ Created app-icon.png with ImageMagick"
elif command -v magick &> /dev/null; then
    magick -size 1024x1024 xc:red app-icon.png
    echo "✅ Created app-icon.png with Magick"
else
    echo "❌ ImageMagick not found. Creating minimal PNG manually..."
    # Create a minimal PNG header for a 32x32 red square
    printf '\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00 \x00\x00\x00 \x08\x02\x00\x00\x00\xfc\x18\xed\xa3\x00\x00\x00\tpHYs\x00\x00\x0b\x13\x00\x00\x0b\x13\x01\x00\x9a\x9c\x18\x00\x00\x00\x18tEXtSoftware\x00paint.net 4.0.6\x9c\x02\x00\x00\x00\x0cIDATx\x9cc\xf8\x0f\x00\x00\x01\x00\x01\x00\x18\xdd\x8d\xb4\x00\x00\x00\x00IEND\xaeB`\x82' > app-icon.png
    echo "✅ Created minimal app-icon.png"
fi

# Generate all icon formats using Tauri CLI
npx @tauri-apps/cli icon app-icon.png

echo "✅ Generated all icon formats!"
ls -la src-tauri/icons/
