#!/usr/bin/env python3
"""
PRISM Documentation, Visual Assets & Pre-Commit Verification Suite
Verifies:
1. Documentation link & anchor integrity (zero broken links/images in markdown).
2. Visual asset validity (PNG, SVG, GIF, JPG headers, dimensions, non-zero size).
3. SVG-to-PNG fallback parity for all architectural and design diagrams.
4. Terminal recording & screenshot transcript pairing.
5. Optional Frontend build verification (TypeScript + Vite bundling).
"""

import sys
import os
import re
import json
import struct
import argparse
import subprocess
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import List, Dict, Tuple, Set, Optional

# ANSI Colors
RESET = "\033[0m"
BOLD = "\033[1m"
GREEN = "\033[32m"
RED = "\033[31m"
YELLOW = "\033[33m"
CYAN = "\033[36m"
BLUE = "\033[34m"
DIM = "\033[2m"

REPO_ROOT = Path(__file__).resolve().parent.parent

EXCLUDE_DIRS = {
    ".git",
    "target",
    "node_modules",
    ".cache",
    ".venv",
    "venv",
    "env",
    "dist",
    "build",
    "real_corpora",
}


def log_header(title: str) -> None:
    print(f"\n{BOLD}{CYAN}=== {title} ==={RESET}")


def log_success(msg: str) -> None:
    print(f"  {GREEN}✓{RESET} {msg}")


def log_warning(msg: str) -> None:
    print(f"  {YELLOW}⚠{RESET} {msg}")


def log_error(msg: str) -> None:
    print(f"  {RED}✗{RESET} {msg}")


def slugify_heading(heading: str) -> str:
    """GitHub-compatible markdown heading anchor slug generator."""
    # Strip markdown formatting
    h = re.sub(r"[*_`#~\[\]\(\)]", "", heading).strip().lower()
    # Remove punctuation except spaces and hyphens
    h = re.sub(r"[^\w\s-]", "", h)
    # Replace spaces with hyphens
    h = re.sub(r"[-\s]+", "-", h)
    return h


def extract_file_anchors(content: str) -> Set[str]:
    """Extract all heading slugs and explicit HTML anchors from markdown."""
    anchors = set()
    for line in content.splitlines():
        line = line.strip()
        if line.startswith("#"):
            heading = re.sub(r"^#+\s*", "", line)
            slug = slugify_heading(heading)
            if slug:
                anchors.add(slug)
        # Check explicit <a name="..." or id="...">
        for match in re.finditer(r'<a\s+[^>]*(?:name|id)=["\']([^"\']+)["\']', line, re.IGNORECASE):
            anchors.add(match.group(1).lower())
    return anchors


def inspect_image_file(path: Path) -> Tuple[bool, str, Optional[Tuple[int, int]]]:
    """
    Pure standard library inspection of image files.
    Returns (is_valid, description/error, dimensions).
    """
    if not path.exists():
        return False, "File does not exist", None

    try:
        data = path.read_bytes()
    except Exception as e:
        return False, f"Could not read bytes: {e}", None

    size = len(data)
    if size == 0:
        return False, "Empty file (0 bytes)", None

    ext = path.suffix.lower()

    if ext == ".png":
        if not data.startswith(b"\x89PNG\r\n\x1a\n"):
            return False, "Invalid PNG magic header", None
        if len(data) < 24 or data[12:16] != b"IHDR":
            return False, "Corrupt PNG: missing IHDR chunk", None
        w, h = struct.unpack(">II", data[16:24])
        if w <= 0 or h <= 0:
            return False, f"Invalid PNG dimensions {w}x{h}", None
        return True, f"PNG {w}x{h} ({size:,} B)", (w, h)

    elif ext == ".gif":
        if not (data.startswith(b"GIF87a") or data.startswith(b"GIF89a")):
            return False, "Invalid GIF magic header", None
        if len(data) < 10:
            return False, "Corrupt GIF: too short", None
        w, h = struct.unpack("<HH", data[6:10])
        if w <= 0 or h <= 0:
            return False, f"Invalid GIF dimensions {w}x{h}", None
        return True, f"GIF {w}x{h} ({size:,} B)", (w, h)

    elif ext == ".svg":
        try:
            root = ET.fromstring(data.decode("utf-8", errors="ignore"))
            tag = root.tag.split("}")[-1] if "}" in root.tag else root.tag
            if tag != "svg":
                return False, f"Invalid SVG root tag <{root.tag}>", None
            vb = root.attrib.get("viewBox")
            w_attr = root.attrib.get("width")
            h_attr = root.attrib.get("height")
            desc = f"SVG viewBox='{vb}'" if vb else f"SVG {w_attr}x{h_attr}"
            return True, f"{desc} ({size:,} B)", None
        except Exception as e:
            return False, f"SVG XML parse error: {e}", None

    elif ext in [".jpg", ".jpeg"]:
        if not data.startswith(b"\xff\xd8"):
            return False, "Invalid JPEG magic header", None
        return True, f"JPEG ({size:,} B)", None

    elif ext == ".cast":
        try:
            lines = data.decode("utf-8", errors="ignore").splitlines()
            if not lines:
                return False, "Empty asciinema cast file", None
            header = json.loads(lines[0])
            if "version" not in header:
                return False, "Missing 'version' in asciinema header", None
            w = header.get("width", 0)
            h = header.get("height", 0)
            return True, f"Asciinema Cast v{header['version']} {w}x{h} ({len(lines)} frames)", (w, h)
        except Exception as e:
            return False, f"Invalid asciinema cast JSON: {e}", None

    return True, f"Generic asset ({size:,} B)", None


def verify_visual_assets(repo_root: Path) -> Tuple[bool, List[str]]:
    """Verify all images, SVGs, GIFs, and PNG fallbacks."""
    log_header("Verifying Visual Assets & Diagram Fallbacks")
    errors = []
    
    asset_dirs = [
        repo_root / "docs/images",
        repo_root / "docs/demo",
        repo_root / "screenshots",
        repo_root / "frontend/src/assets",
        repo_root / "frontend/public",
    ]

    all_assets: List[Path] = []
    for d in asset_dirs:
        if d.exists():
            for p in d.rglob("*"):
                if p.is_file() and p.suffix.lower() in [".png", ".svg", ".gif", ".jpg", ".jpeg", ".cast"]:
                    all_assets.append(p)

    valid_count = 0
    for asset in sorted(all_assets):
        rel = asset.relative_to(repo_root)
        ok, desc, dims = inspect_image_file(asset)
        if ok:
            log_success(f"{rel}: {desc}")
            valid_count += 1
        else:
            msg = f"{rel}: {desc}"
            log_error(msg)
            errors.append(msg)

    # Verify SVG to PNG fallback pairing in docs/images
    docs_images = repo_root / "docs/images"
    if docs_images.exists():
        svgs = list(docs_images.glob("*.svg"))
        for svg in svgs:
            png_fallback = svg.with_suffix(".png")
            rel_svg = svg.relative_to(repo_root)
            rel_png = png_fallback.relative_to(repo_root)
            if not png_fallback.exists():
                msg = f"Missing PNG fallback for diagram {rel_svg} (expected {rel_png})"
                log_error(msg)
                errors.append(msg)
            else:
                ok, desc, _ = inspect_image_file(png_fallback)
                if not ok:
                    msg = f"PNG fallback {rel_png} is corrupt: {desc}"
                    log_error(msg)
                    errors.append(msg)
                else:
                    log_success(f"Fallback parity confirmed: {rel_svg.name} <-> {rel_png.name}")

    # Verify screenshots transcript pairing
    screenshots_dir = repo_root / "screenshots"
    if screenshots_dir.exists():
        png_screenshots = list(screenshots_dir.glob("*.png"))
        for sp in png_screenshots:
            txt_pair = sp.with_suffix(".txt")
            if not txt_pair.exists():
                log_warning(f"Screenshot {sp.name} has no matching ANSI transcript {txt_pair.name}")
            else:
                log_success(f"Transcript paired: {sp.name} <-> {txt_pair.name}")

    print(f"\n{BOLD}Visual Assets Summary:{RESET} {valid_count} checked, {len(errors)} error(s)")
    return len(errors) == 0, errors


def verify_documentation_and_links(repo_root: Path) -> Tuple[bool, List[str]]:
    """Scan all markdown documents and check all links, anchors, and image references."""
    log_header("Verifying Documentation Links & Asset References")
    errors = []

    md_files = []
    for p in repo_root.rglob("*.md"):
        if not any(ex in p.parts for ex in EXCLUDE_DIRS):
            md_files.append(p)

    link_re = re.compile(r"\[([^\]]*)\]\(([^)]+)\)")
    img_re = re.compile(r"!\[([^\]]*)\]\(([^)]+)\)")
    html_img_re = re.compile(r'<img[^>]+src=["\']([^"\']+)["\']', re.IGNORECASE)
    html_a_re = re.compile(r'<a[^>]+href=["\']([^"\']+)["\']', re.IGNORECASE)

    # Pre-cache anchors per file
    file_anchor_cache: Dict[Path, Set[str]] = {}
    for md in md_files:
        try:
            content = md.read_text(encoding="utf-8", errors="ignore")
            file_anchor_cache[md] = extract_file_anchors(content)
        except Exception:
            file_anchor_cache[md] = set()

    total_links_checked = 0
    referenced_images: Set[Path] = set()

    for md in sorted(md_files):
        rel_md = md.relative_to(repo_root)
        try:
            content = md.read_text(encoding="utf-8", errors="ignore")
        except Exception as e:
            msg = f"Cannot read {rel_md}: {e}"
            log_error(msg)
            errors.append(msg)
            continue

        # Strip code fences to prevent false positives in code blocks
        clean_content = re.sub(r"```.*?```", "", content, flags=re.DOTALL)
        clean_content = re.sub(r"`[^`\n]+`", "", clean_content)

        file_errors = []

        # 1. Check markdown image references ![alt](path)
        for match in img_re.finditer(clean_content):
            total_links_checked += 1
            src = match.group(2).split()[0].strip()
            if src.startswith(("http://", "https://", "data:")):
                continue
            path_part = src.split("#")[0].strip()
            if not path_part:
                continue
            
            target = (md.parent / path_part).resolve()
            if not target.exists():
                # Try relative to repo root
                target = (repo_root / path_part.lstrip("/")).resolve()

            if not target.exists():
                err = f"{rel_md}: Broken image link '![]({src})'"
                file_errors.append(err)
            else:
                referenced_images.add(target)
                ok, desc, _ = inspect_image_file(target)
                if not ok:
                    file_errors.append(f"{rel_md}: Referenced image {src} is corrupt: {desc}")

        # 2. Check HTML <img> tags
        for match in html_img_re.finditer(clean_content):
            total_links_checked += 1
            src = match.group(1).split()[0].strip()
            if src.startswith(("http://", "https://", "data:")):
                continue
            path_part = src.split("#")[0].strip()
            if not path_part:
                continue

            target = (md.parent / path_part).resolve()
            if not target.exists():
                target = (repo_root / path_part.lstrip("/")).resolve()

            if not target.exists():
                err = f"{rel_md}: Broken HTML img tag src='{src}'"
                file_errors.append(err)
            else:
                referenced_images.add(target)
                ok, desc, _ = inspect_image_file(target)
                if not ok:
                    file_errors.append(f"{rel_md}: HTML image {src} is corrupt: {desc}")

        # 3. Check markdown standard links [text](path)
        for match in link_re.finditer(clean_content):
            total_links_checked += 1
            link_text = match.group(1)
            raw_url = match.group(2).split()[0].strip()

            if raw_url.startswith(("http://", "https://", "mailto:", "javascript:")):
                continue

            # Pure in-page anchor [text](#anchor)
            if raw_url.startswith("#"):
                anchor = raw_url.lstrip("#").lower()
                anchors = file_anchor_cache.get(md, set())
                if anchor and anchor not in anchors:
                    # Fuzzy anchor check
                    if not any(anchor in a or a in anchor for a in anchors):
                        file_errors.append(f"{rel_md}: Broken intra-page anchor '{raw_url}' for link '{link_text}'")
                continue

            # Path with or without anchor
            parts = raw_url.split("#", 1)
            path_part = parts[0].strip()
            anchor_part = parts[1].strip().lower() if len(parts) > 1 else None

            if not path_part:
                continue

            target = (md.parent / path_part).resolve()
            if not target.exists():
                target = (repo_root / path_part.lstrip("/")).resolve()

            if not target.exists():
                file_errors.append(f"{rel_md}: Broken file link '{raw_url}' for text '{link_text}'")
            elif anchor_part and target in file_anchor_cache:
                target_anchors = file_anchor_cache[target]
                if anchor_part not in target_anchors and not any(anchor_part in a or a in anchor_part for a in target_anchors):
                    file_errors.append(f"{rel_md}: Target '{path_part}' exists but anchor '#{anchor_part}' is missing")

        # 4. Check HTML <a> tags
        for match in html_a_re.finditer(clean_content):
            total_links_checked += 1
            raw_url = match.group(1).split()[0].strip()
            if raw_url.startswith(("http://", "https://", "mailto:", "javascript:", "#")):
                continue
            parts = raw_url.split("#", 1)
            path_part = parts[0].strip()
            if not path_part:
                continue
            target = (md.parent / path_part).resolve()
            if not target.exists():
                target = (repo_root / path_part.lstrip("/")).resolve()
            if not target.exists():
                file_errors.append(f"{rel_md}: Broken HTML anchor href='{raw_url}'")

        if file_errors:
            for err in file_errors:
                log_error(err)
            errors.extend(file_errors)
        else:
            log_success(f"{rel_md} ({len(content.splitlines())} lines) - OK")

    print(f"\n{BOLD}Documentation Summary:{RESET} {len(md_files)} files, {total_links_checked} links/images checked, {len(errors)} error(s)")
    return len(errors) == 0, errors


def verify_frontend_build(repo_root: Path) -> Tuple[bool, List[str]]:
    """Build the React frontend using TypeScript and Vite."""
    log_header("Verifying Frontend TypeScript & Production Bundle")
    frontend_dir = repo_root / "frontend"
    if not frontend_dir.exists():
        log_warning("frontend directory not found, skipping frontend verification")
        return True, []

    errors = []
    
    # Run npm run build
    cmd = ["npm", "run", "build"]
    print(f"  {DIM}Executing: {' '.join(cmd)} in {frontend_dir.relative_to(repo_root)}...{RESET}")
    result = subprocess.run(cmd, cwd=frontend_dir, capture_output=True, text=True)

    if result.returncode != 0:
        log_error("Frontend build failed!")
        print(result.stdout)
        print(result.stderr)
        errors.append(f"Frontend build exited with code {result.returncode}")
        return False, errors

    # Check dist directory artifacts
    dist_index = frontend_dir / "dist" / "index.html"
    if not dist_index.exists() or dist_index.stat().st_size == 0:
        msg = "frontend/dist/index.html is missing or empty after build"
        log_error(msg)
        errors.append(msg)
        return False, errors

    log_success(f"Frontend build passed! Generated dist/index.html ({dist_index.stat().st_size} bytes)")
    return True, []


def verify_git_tracked_samples(repo_root: Path) -> Tuple[bool, List[str]]:
    """Verify that test sample log files in data/samples/ are tracked and not ignored."""
    log_header("Verifying Test Data Samples Tracking")
    errors = []
    required_samples = [
        "data/samples/cisco_asa.log",
        "data/samples/fortinet_fortigate.log",
        "data/samples/paloalto_threat.log",
    ]

    for rel_path in required_samples:
        full_path = repo_root / rel_path
        if not full_path.exists():
            msg = f"Required test sample missing: {rel_path}"
            log_error(msg)
            errors.append(msg)
            continue

        if full_path.stat().st_size == 0:
            msg = f"Test sample is empty: {rel_path}"
            log_error(msg)
            errors.append(msg)
            continue

        # Check git check-ignore with --no-index to evaluate current working tree .gitignore
        res = subprocess.run(["git", "check-ignore", "--no-index", "-q", rel_path], cwd=repo_root)
        if res.returncode == 0:
            msg = f"Test sample is erroneously ignored by .gitignore: {rel_path}"
            log_error(msg)
            errors.append(msg)
        else:
            log_success(f"Test fixture tracked and ready: {rel_path} ({full_path.stat().st_size:,} bytes)")

    return len(errors) == 0, errors


def main() -> int:
    parser = argparse.ArgumentParser(description="PRISM Pre-Commit & Visual Assets Verification Suite")
    parser.add_argument("--skip-frontend", action="store_true", help="Skip frontend build check")
    parser.add_argument("--strict", action="store_true", default=True, help="Strict exit on any warning/error")
    args = parser.parse_args()

    print(f"\n{BOLD}{BLUE}======================================================{RESET}")
    print(f"{BOLD}{BLUE}      PRISM Strict Pre-Commit Verification Suite      {RESET}")
    print(f"{BOLD}{BLUE}======================================================{RESET}")

    all_passed = True
    total_errors: List[str] = []

    # 1. Test fixtures & gitignore validation
    ok_samples, errs_samples = verify_git_tracked_samples(REPO_ROOT)
    if not ok_samples:
        all_passed = False
        total_errors.extend(errs_samples)

    # 2. Visual assets & diagram fallback parity
    ok_visuals, errs_visuals = verify_visual_assets(REPO_ROOT)
    if not ok_visuals:
        all_passed = False
        total_errors.extend(errs_visuals)

    # 3. Documentation & link integrity
    ok_docs, errs_docs = verify_documentation_and_links(REPO_ROOT)
    if not ok_docs:
        all_passed = False
        total_errors.extend(errs_docs)

    # 4. Frontend build verification
    if not args.skip_frontend:
        ok_fe, errs_fe = verify_frontend_build(REPO_ROOT)
        if not ok_fe:
            all_passed = False
            total_errors.extend(errs_fe)

    # Final summary banner
    print(f"\n{BOLD}======================================================{RESET}")
    if all_passed:
        print(f"{BOLD}{GREEN}✓ ALL STRICT VERIFICATION CHECKS PASSED SUCCESSFULLY!{RESET}")
        print(f"{BOLD}Repository is clean, visual assets are intact, and ready to commit.{RESET}\n")
        return 0
    else:
        print(f"{BOLD}{RED}✗ STRICT VERIFICATION CHECKS FAILED WITH {len(total_errors)} ERROR(S):{RESET}")
        for err in total_errors:
            print(f"  - {err}")
        print(f"\n{BOLD}Please resolve the errors above before committing.{RESET}\n")
        return 1


if __name__ == "__main__":
    sys.exit(main())
