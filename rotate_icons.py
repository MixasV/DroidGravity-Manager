#!/usr/bin/env python3
"""Rotate all PNG images in src-tauri/icons directory 90 degrees counter-clockwise"""

from PIL import Image
import os
from pathlib import Path

icons_dir = Path(__file__).parent / "src-tauri" / "icons"

def rotate_image(image_path):
    """Rotate a single image 90 degrees counter-clockwise"""
    try:
        img = Image.open(image_path)
        rotated = img.rotate(90, expand=True)
        rotated.save(image_path)
        print(f"[OK] Rotated: {image_path.relative_to(icons_dir)}")
        return True
    except Exception as e:
        print(f"[FAIL] Failed: {image_path.relative_to(icons_dir)} - {e}")
        return False

def main():
    if not icons_dir.exists():
        print(f"Error: Icons directory not found: {icons_dir}")
        return
    
    png_files = list(icons_dir.rglob("*.png"))
    print(f"Found {len(png_files)} PNG files\n")
    
    success_count = 0
    for png_file in png_files:
        if rotate_image(png_file):
            success_count += 1
    
    print(f"\nRotated {success_count}/{len(png_files)} images successfully")

if __name__ == "__main__":
    main()
