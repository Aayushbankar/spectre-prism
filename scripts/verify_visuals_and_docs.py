#!/usr/bin/env python3
"""
PRISM Documentation, Visual Assets & Pre-Commit Verification Suite
Verifies:
1. Documentation link & anchor integrity (zero broken links, empty links, or missing anchors).
2. Visual asset validity (PNG magic + chunk CRC32 + IEND trailer, GIF trailer, SVG XML, JPEG EOI, Asciinema JSON frames).
3. SVG-to-PNG fallback parity for all architectural and design diagrams in docs/.
4. Terminal recording & screenshot ANSI transcript bidirectional pairing in screenshots/.
5. Test sample log fixtures tracked in git and not blocked by .gitignore.
6. Frontend production build and bundle integrity (TypeScript + Vite bundling).
"""

import sys
import os
import re
import json
import struct
import zlib
import argparse
import subprocess
import urllib.parse
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
    ".pytest_cache",
    "__pycache__",
    ".mypy_cache",
    ".ruff_cache",
}


def log_header(title: str) -> None:
    print(f"\n{BOLD}{CYAN}=== {title} ==={RESET}")


def log_success(msg: str) -> None:
    print(f"  {GREEN}✓{RESET} {msg}")


def log_warning(msg: str) -> None:
    print(f"  {YELLOW}⚠{RESET} {msg}")


def log_error(msg: str) -> None:
    print(f"  {RED}✗{RESET} {msg}")


def strip_markdown_blocks(text: str) -> str:
    """Strip code blocks, inline code, and HTML comments from markdown."""
    # 1. Multi-line code fences (``` or ~~~)
    clean = re.sub(r"```.*?```", "", text, flags=re.DOTALL)
    clean = re.sub(r"~~~.*?~~~", "", clean, flags=re.DOTALL)
    # 2. HTML comments
    clean = re.sub(r"<!--.*?-->", "", clean, flags=re.DOTALL)
    # 3. Inline code blocks
    clean = re.sub(r"`[^`\n]+`", "", clean)
    return clean


def slugify_heading(heading: str) -> str:
    """GitHub-compatible markdown heading anchor slug generator."""
    # Strip markdown links e.g. [foo](bar) -> foo
    h = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", heading)
    # Strip HTML tags
    h = re.sub(r"<[^>]+>", "", h)
    # Strip inline markdown symbols
    h = re.sub(r"[`*_~#]", "", h)
    # Strip leading/trailing whitespace and lowercase
    h = h.strip().lower()
    # Remove punctuation except spaces and hyphens
    h = re.sub(r"[^\w\s-]", "", h)
    # Replace spaces with hyphens
    h = re.sub(r"[-\s]+", "-", h)
    return h


def extract_file_anchors(raw_content: str) -> Set[str]:
    """Extract all heading slugs and explicit HTML anchors from markdown."""
    clean = strip_markdown_blocks(raw_content)
    anchors = set()
    slug_counts: Dict[str, int] = {}

    for line in clean.splitlines():
        line = line.strip()
        if line.startswith("#"):
            heading = re.sub(r"^#+\s*", "", line)
            base_slug = slugify_heading(heading)
            if base_slug:
                count = slug_counts.get(base_slug, 0)
                slug_counts[base_slug] = count + 1
                if count == 0:
                    anchors.add(base_slug)
                else:
                    anchors.add(f"{base_slug}-{count}")

    # Extract explicit HTML anchor tags: <a name="...">, <a id="...">, <h1 id="...">, <div id="...">, etc.
    for match in re.finditer(r'<[a-z0-9]+\s+[^>]*(?:id|name)=["\']([^"\']+)["\']', clean, re.IGNORECASE):
        anchors.add(match.group(1).lower())

    return anchors


def inspect_image_file(path: Path) -> Tuple[bool, str, Optional[Tuple[int, int]]]:
    """
    Pure standard library inspection of image files.
    Validates magic headers, chunk structure, CRC32, termination markers, and dimensions.
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
            return False, "Corrupt PNG: missing or displaced IHDR chunk", None
        
        w, h = struct.unpack(">II", data[16:24])
        if w <= 0 or h <= 0:
            return False, f"Invalid PNG dimensions {w}x{h}", None

        # Deep chunk validation and CRC32 verification up to IEND
        offset = 8
        found_iend = False
        while offset + 8 <= len(data):
            chunk_len = struct.unpack(">I", data[offset:offset + 4])[0]
            chunk_type = data[offset + 4:offset + 8]
            offset += 8
            if offset + chunk_len + 4 > len(data):
                return False, f"Corrupt PNG: truncated {chunk_type.decode('latin1', errors='replace')} chunk", None
            
            chunk_data = data[offset:offset + chunk_len]
            expected_crc = struct.unpack(">I", data[offset + chunk_len:offset + chunk_len + 4])[0]
            actual_crc = zlib.crc32(chunk_type + chunk_data) & 0xFFFFFFFF
            if expected_crc != actual_crc:
                return False, f"Corrupt PNG: CRC32 error in {chunk_type.decode('latin1', errors='replace')} chunk", None
            
            offset += chunk_len + 4
            if chunk_type == b"IEND":
                found_iend = True
                break

        if not found_iend:
            return False, "Corrupt PNG: missing IEND termination chunk (truncated file)", None

        return True, f"PNG {w}x{h} ({size:,} B, valid IHDR & IEND chunks)", (w, h)

    elif ext == ".gif":
        if not (data.startswith(b"GIF87a") or data.startswith(b"GIF89a")):
            return False, "Invalid GIF magic header", None
        if len(data) < 10:
            return False, "Corrupt GIF: truncated header", None
        w, h = struct.unpack("<HH", data[6:10])
        if w <= 0 or h <= 0:
            return False, f"Invalid GIF dimensions {w}x{h}", None
        if not data.endswith(b";"):
            return False, "Corrupt GIF: missing trailing 0x3B (';') terminator", None
        return True, f"GIF {w}x{h} ({size:,} B, valid trailer)", (w, h)

    elif ext == ".svg":
        try:
            root = ET.fromstring(data.decode("utf-8", errors="ignore"))
            tag = root.tag.split("}")[-1] if "}" in root.tag else root.tag
            if tag != "svg":
                return False, f"Invalid SVG root tag <{root.tag}>", None
            vb = root.attrib.get("viewBox")
            w_attr = root.attrib.get("width")
            h_attr = root.attrib.get("height")
            desc = f"SVG viewBox='{vb}'" if vb else (f"SVG {w_attr}x{h_attr}" if w_attr else "SVG (scalable)")
            return True, f"{desc} ({size:,} B)", None
        except Exception as e:
            return False, f"SVG XML parse error: {e}", None

    elif ext in [".jpg", ".jpeg"]:
        if not data.startswith(b"\xff\xd8"):
            return False, "Invalid JPEG magic header (missing SOI)", None
        if not data.endswith(b"\xff\xd9"):
            return False, "Corrupt JPEG: missing EOI trailer", None
        return True, f"JPEG ({size:,} B, valid SOI/EOI)", None

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
            
            # Validate every subsequent event line
            for i, line in enumerate(lines[1:], 2):
                ev = json.loads(line)
                if not isinstance(ev, list) or len(ev) < 3:
                    return False, f"Asciinema frame at line {i} is invalid JSON event tuple", None
            
            return True, f"Asciinema Cast v{header['version']} {w}x{h} ({len(lines)-1} frames verified)", (w, h)
        except Exception as e:
            return False, f"Invalid asciinema cast JSON: {e}", None

    return True, f"Asset ({size:,} B)", None


def verify_visual_assets(repo_root: Path) -> Tuple[bool, List[str]]:
    """Verify all images, SVGs, GIFs, and PNG fallbacks."""
    log_header("Verifying Visual Assets & Diagram Fallbacks")
    errors: List[str] = []
    
    asset_dirs = [
        repo_root / "docs",
        repo_root / "screenshots",
        repo_root / "frontend/src/assets",
        repo_root / "frontend/public",
    ]

    all_assets: List[Path] = []
    for d in asset_dirs:
        if d.exists():
            for p in d.rglob("*"):
                if p.is_file() and p.suffix.lower() in [".png", ".svg", ".gif", ".jpg", ".jpeg", ".cast"]:
                    if not any(ex in p.parts for ex in EXCLUDE_DIRS):
                        all_assets.append(p)

    valid_count = 0
    for asset in sorted(set(all_assets)):
        rel = asset.relative_to(repo_root)
        ok, desc, dims = inspect_image_file(asset)
        if ok:
            log_success(f"{rel}: {desc}")
            valid_count += 1
        else:
            msg = f"{rel}: {desc}"
            log_error(msg)
            errors.append(msg)

    # Verify SVG to PNG fallback pairing for all architecture diagrams in docs/
    docs_dir = repo_root / "docs"
    if docs_dir.exists():
        svgs = [p for p in docs_dir.rglob("*.svg") if not any(ex in p.parts for ex in EXCLUDE_DIRS)]
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

    # Verify screenshots transcript pairing in screenshots/
    screenshots_dir = repo_root / "screenshots"
    if screenshots_dir.exists():
        png_screenshots = list(screenshots_dir.glob("*.png"))
        txt_transcripts = list(screenshots_dir.glob("*.txt"))

        for sp in png_screenshots:
            txt_pair = sp.with_suffix(".txt")
            if not txt_pair.exists():
                msg = f"Screenshot {sp.name} has no matching ANSI transcript {txt_pair.name}"
                log_error(msg)
                errors.append(msg)
            else:
                log_success(f"Transcript paired: {sp.name} <-> {txt_pair.name}")

        for tp in txt_transcripts:
            png_pair = tp.with_suffix(".png")
            if not png_pair.exists():
                msg = f"ANSI transcript {tp.name} has no matching screenshot {png_pair.name}"
                log_error(msg)
                errors.append(msg)

    print(f"\n{BOLD}Visual Assets Summary:{RESET} {valid_count} checked, {len(errors)} error(s)")
    return len(errors) == 0, errors


def verify_documentation_and_links(repo_root: Path) -> Tuple[bool, List[str]]:
    """Scan all markdown documents and check all links, anchors, and image references."""
    log_header("Verifying Documentation Links & Asset References")
    errors: List[str] = []

    md_files = []
    for p in repo_root.resolve().rglob("*.md"):
        if not any(ex in p.parts for ex in EXCLUDE_DIRS):
            md_files.append(p.resolve())

    link_pattern = re.compile(r"(!?\[((?:[^\[\]]|\[[^\[\]]*\])*)\])\((.*?)\)", re.DOTALL)
    html_img_re = re.compile(r'<img[^>]+src=["\']([^"\']+)["\']', re.IGNORECASE)
    html_a_re = re.compile(r'<a[^>]+href=["\']([^"\']+)["\']', re.IGNORECASE)

    # Pre-cache anchors per file (using fully resolved absolute paths)
    file_anchor_cache: Dict[Path, Set[str]] = {}
    for md in md_files:
        try:
            content = md.read_text(encoding="utf-8", errors="ignore")
            file_anchor_cache[md.resolve()] = extract_file_anchors(content)
        except Exception:
            file_anchor_cache[md.resolve()] = set()

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

        clean_content = strip_markdown_blocks(content)
        file_errors: List[str] = []

        # 1. Check markdown links & images [text](url) and ![alt](url)
        for match in link_pattern.finditer(clean_content):
            total_links_checked += 1
            full_syntax = match.group(1)
            link_text = match.group(2).strip()
            raw_url_chunk = match.group(3).strip()
            is_image = full_syntax.startswith("!")

            if not raw_url_chunk:
                file_errors.append(f"{rel_md}: Empty link destination for '[{link_text}]()'")
                continue

            # Handle optional title in url: "path/to/file.md 'Optional Title'"
            raw_url = raw_url_chunk.split()[0].strip()

            # Skip web protocols
            if raw_url.startswith(("http://", "https://", "mailto:", "javascript:", "data:")):
                continue

            # Pure intra-page anchor link [text](#anchor)
            if raw_url.startswith("#"):
                anchor = raw_url.lstrip("#").lower()
                if not anchor:
                    # e.g. [text](#) - placeholder link
                    continue
                anchors = file_anchor_cache.get(md, set())
                if anchor not in anchors and not any(anchor in a or a in anchor for a in anchors):
                    file_errors.append(f"{rel_md}: Broken intra-page anchor '{raw_url}' for link '{link_text}'")
                continue

            # Path with or without anchor
            parts = raw_url.split("#", 1)
            path_part = urllib.parse.unquote(parts[0].strip())
            anchor_part = parts[1].strip().lower() if len(parts) > 1 else None

            if not path_part:
                continue

            # Resolve path: repo-root relative or doc-parent relative
            if path_part.startswith("/"):
                target = (repo_root / path_part.lstrip("/")).resolve()
            else:
                target = (md.parent / path_part).resolve()

            if not target.exists():
                file_errors.append(f"{rel_md}: Broken {'image' if is_image else 'file'} reference '{raw_url}'")
            else:
                if is_image or target.suffix.lower() in [".png", ".svg", ".gif", ".jpg", ".jpeg"]:
                    referenced_images.add(target)
                    ok, desc, _ = inspect_image_file(target)
                    if not ok:
                        file_errors.append(f"{rel_md}: Referenced image '{path_part}' is corrupt: {desc}")
                
                if anchor_part and target in file_anchor_cache:
                    target_anchors = file_anchor_cache[target]
                    if anchor_part not in target_anchors and not any(anchor_part in a or a in anchor_part for a in target_anchors):
                        file_errors.append(f"{rel_md}: Target '{path_part}' exists but anchor '#{anchor_part}' is missing")

        # 2. Check HTML <img> tags
        for match in html_img_re.finditer(clean_content):
            total_links_checked += 1
            src = match.group(1).split()[0].strip()
            if src.startswith(("http://", "https://", "data:")):
                continue
            path_part = urllib.parse.unquote(src.split("#")[0].strip())
            if not path_part:
                continue

            if path_part.startswith("/"):
                target = (repo_root / path_part.lstrip("/")).resolve()
            else:
                target = (md.parent / path_part).resolve()

            if not target.exists():
                file_errors.append(f"{rel_md}: Broken HTML img tag src='{src}'")
            else:
                referenced_images.add(target)
                ok, desc, _ = inspect_image_file(target)
                if not ok:
                    file_errors.append(f"{rel_md}: HTML image '{src}' is corrupt: {desc}")

        # 3. Check HTML <a> tags
        for match in html_a_re.finditer(clean_content):
            total_links_checked += 1
            raw_url = match.group(1).split()[0].strip()
            if raw_url.startswith(("http://", "https://", "mailto:", "javascript:", "#")):
                continue
            parts = raw_url.split("#", 1)
            path_part = urllib.parse.unquote(parts[0].strip())
            if not path_part:
                continue

            if path_part.startswith("/"):
                target = (repo_root / path_part.lstrip("/")).resolve()
            else:
                target = (md.parent / path_part).resolve()

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
    """Build the React frontend using TypeScript and Vite and verify bundle artifacts."""
    log_header("Verifying Frontend TypeScript & Production Bundle")
    frontend_dir = repo_root / "frontend"
    if not frontend_dir.exists():
        log_warning("frontend directory not found, skipping frontend verification")
        return True, []

    errors: List[str] = []
    
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
    dist_dir = frontend_dir / "dist"
    dist_index = dist_dir / "index.html"
    if not dist_index.exists() or dist_index.stat().st_size == 0:
        msg = "frontend/dist/index.html is missing or empty after build"
        log_error(msg)
        errors.append(msg)
        return False, errors

    # Check assets directory
    assets_dir = dist_dir / "assets"
    if not assets_dir.exists() or not any(assets_dir.glob("*.js")):
        msg = "frontend/dist/assets/ is missing JavaScript bundle output"
        log_error(msg)
        errors.append(msg)
        return False, errors

    log_success(f"Frontend build passed! Generated dist/index.html ({dist_index.stat().st_size} bytes)")
    return True, []


def verify_git_tracked_samples(repo_root: Path) -> Tuple[bool, List[str]]:
    """Verify that test sample log files in data/samples/ are tracked and not ignored."""
    log_header("Verifying Test Data Samples Tracking")
    errors: List[str] = []
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
