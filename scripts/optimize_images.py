#!/usr/bin/env python3
"""
PRISM Lossless Image Optimization Tool
Losslessly compresses all PNGs and GIFs across docs/, screenshots/, and frontend/.
Applies PIL zlib compression level 9 and chunk optimization if and only if resulting size is strictly smaller.
"""

import os
import sys
from pathlib import Path
from PIL import Image, ImageSequence

REPO_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO_ROOT / "scripts"))
from verify_visuals_and_docs import inspect_image_file


def optimize_png(file_path: Path) -> int:
    orig_size = file_path.stat().st_size
    temp_path = file_path.with_suffix(".tmp.png")
    try:
        with Image.open(file_path) as im:
            im.save(temp_path, format="PNG", optimize=True, compress_level=9)
        new_size = temp_path.stat().st_size
        if new_size < orig_size:
            # Verify validity of optimized file before replacing
            ok, desc, _ = inspect_image_file(temp_path)
            if ok:
                temp_path.replace(file_path)
                saved = orig_size - new_size
                print(f"  ✓ Optimized {file_path.relative_to(REPO_ROOT)}: {orig_size:,} B -> {new_size:,} B (saved {saved:,} B)")
                return saved
            else:
                print(f"  ⚠ Validation failed for {file_path.relative_to(REPO_ROOT)}: {desc}")
        if temp_path.exists():
            temp_path.unlink()
    except Exception as e:
        print(f"  ✗ Error optimizing {file_path.relative_to(REPO_ROOT)}: {e}")
        if temp_path.exists():
            temp_path.unlink()
    return 0


def optimize_gif(file_path: Path) -> int:
    orig_size = file_path.stat().st_size
    temp_path = file_path.with_suffix(".tmp.gif")
    try:
        with Image.open(file_path) as im:
            duration = im.info.get("duration", 2500)
            loop = im.info.get("loop", 0)
            frames = [frame.copy() for frame in ImageSequence.Iterator(im)]
            if frames:
                frames[0].save(
                    temp_path,
                    save_all=True,
                    append_images=frames[1:],
                    duration=duration,
                    loop=loop,
                    optimize=True,
                )
        new_size = temp_path.stat().st_size
        if new_size < orig_size:
            ok, desc, _ = inspect_image_file(temp_path)
            if ok:
                temp_path.replace(file_path)
                saved = orig_size - new_size
                print(f"  ✓ Optimized {file_path.relative_to(REPO_ROOT)}: {orig_size:,} B -> {new_size:,} B (saved {saved:,} B)")
                return saved
        if temp_path.exists():
            temp_path.unlink()
    except Exception as e:
        print(f"  ✗ Error optimizing {file_path.relative_to(REPO_ROOT)}: {e}")
        if temp_path.exists():
            temp_path.unlink()
    return 0


def main():
    print("=== Optimizing Visual Assets (PNG & GIF) ===")
    targets = [
        REPO_ROOT / "docs",
        REPO_ROOT / "screenshots",
        REPO_ROOT / "frontend/src/assets",
    ]
    total_saved = 0
    file_count = 0
    for target in targets:
        if not target.exists():
            continue
        for p in sorted(target.rglob("*")):
            if p.is_file():
                if p.suffix.lower() == ".png":
                    saved = optimize_png(p)
                    total_saved += saved
                    file_count += 1
                elif p.suffix.lower() == ".gif":
                    saved = optimize_gif(p)
                    total_saved += saved
                    file_count += 1

    print(f"\nOptimization complete: Checked {file_count} images, saved a total of {total_saved:,} bytes.")


if __name__ == "__main__":
    main()
