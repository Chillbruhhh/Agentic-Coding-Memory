#!/bin/bash

cd /mnt/c/Users/Joshc/source/repos/ACM/amp/ui

# Create a simple 1024x1024 red square as a fallback
echo "Creating a simple 1024x1024 square icon..."

# Use Python to create a simple square PNG
python3 -c "
from PIL import Image
import os

# Create a 1024x1024 red square
img = Image.new('RGBA', (1024, 1024), (239, 68, 68, 255))  # Red color matching your theme
img.save('app-icon.png')
print('✅ Created 1024x1024 square icon')
" 2>/dev/null || {
    echo "Python/PIL not available, creating minimal square..."
    # If Python isn't available, just copy and hope for the best
    cp public/logo/amp-favicon.png app-icon.png
}

# Generate icons
npx @tauri-apps/cli icon app-icon.png

echo "✅ Generated Tauri icons!"
ls -la src-tauri/icons/
