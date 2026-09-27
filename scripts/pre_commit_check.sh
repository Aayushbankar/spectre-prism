#!/usr/bin/env bash
set -euo pipefail

# ANSI color codes
BOLD="\033[1m"
GREEN="\033[32m"
RED="\033[31m"
YELLOW="\033[33m"
CYAN="\033[36m"
RESET="\033[0m"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo -e "${BOLD}${CYAN}======================================================${RESET}"
echo -e "${BOLD}${CYAN}     PRISM Strict Pre-Commit Verification Gate        ${RESET}"
echo -e "${BOLD}${CYAN}======================================================${RESET}"

# Stage 0: Staged file sanity check (no accidentally staged keys, giant binaries, or secrets)
echo -e "\n${BOLD}[Stage 1/4] Checking Staged Files Integrity...${RESET}"
STAGED_FILES=$(git diff --cached --name-only || true)
if [ -n "$STAGED_FILES" ]; then
    BLOCKED_PATTERNS=(".env" "id_rsa" "id_ed25519" ".pem" "key.json")
    for file in $STAGED_FILES; do
        for pattern in "${BLOCKED_PATTERNS[@]}"; do
            if [[ "$file" == *"$pattern"* ]]; then
                echo -e "  ${RED}✗ ERROR: Blocked sensitive file detected in staging: $file${RESET}"
                exit 1
            fi
        done
        # Check if file is > 25MB (GitHub limit warning)
        if [ -f "$file" ]; then
            FILE_SIZE=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null || echo 0)
            if [ "$FILE_SIZE" -gt 26214400 ]; then
                echo -e "  ${RED}✗ ERROR: File exceeds 25MB: $file (${FILE_SIZE} bytes)${RESET}"
                exit 1
            fi
        fi
    done
fi
echo -e "  ${GREEN}✓${RESET} Staged files passed security and size limits."

# Stage 1: Visual Assets, Markdown Links, Diagram Fallbacks, and Frontend Build
echo -e "\n${BOLD}[Stage 2/4] Verifying Visuals, Links, Fallbacks & Frontend...${RESET}"
python3 "$REPO_ROOT/scripts/verify_visuals_and_docs.py"

# Stage 2: Rust Workspace Compilation & Syntax Check
echo -e "\n${BOLD}[Stage 3/4] Verifying Rust Workspace Compilation (cargo check)...${RESET}"
cargo check --workspace --all-targets

# Stage 3: Fast Rust Unit Tests
echo -e "\n${BOLD}[Stage 4/4] Running Rust Unit & Integration Tests (fast suite)...${RESET}"
cargo test --workspace --bins --lib

echo -e "\n${BOLD}${GREEN}======================================================${RESET}"
echo -e "${BOLD}${GREEN}   ✓ ALL PRE-COMMIT VERIFICATION CHECKS PASSED!       ${RESET}"
echo -e "${BOLD}${GREEN}   Commit is authorized to proceed.                   ${RESET}"
echo -e "${BOLD}${GREEN}======================================================${RESET}\n"
exit 0
