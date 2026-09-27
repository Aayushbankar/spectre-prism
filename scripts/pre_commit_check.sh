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

# Stage 1: Staged file security, sensitive patterns & size check
echo -e "\n${BOLD}[Stage 1/4] Checking Staged Files Integrity...${RESET}"
BLOCKED_PATTERNS=(".env" "id_rsa" "id_ed25519" "id_ecdsa" ".pem" ".key" ".pfx" ".p12" "credentials.json")
has_staged_error=0

while IFS= read -r -d '' file; do
    [ -z "$file" ] && continue
    for pattern in "${BLOCKED_PATTERNS[@]}"; do
        if [[ "$file" == *"$pattern"* ]]; then
            echo -e "  ${RED}✗ ERROR: Blocked sensitive file detected in staging: $file${RESET}"
            has_staged_error=1
        fi
    done
    if [ -f "$file" ]; then
        FILE_SIZE=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null || echo 0)
        if [ "$FILE_SIZE" -gt 26214400 ]; then
            echo -e "  ${RED}✗ ERROR: File exceeds 25MB: $file (${FILE_SIZE} bytes)${RESET}"
            has_staged_error=1
        fi
    fi
done < <(git diff --cached --name-only -z || true)

if [ "$has_staged_error" -ne 0 ]; then
    echo -e "  ${RED}Pre-commit aborted due to staging violations.${RESET}"
    exit 1
fi
echo -e "  ${GREEN}✓${RESET} Staged files passed security patterns and size limits."

# Stage 2: Visual Assets, Markdown Links, Diagram Fallbacks, and Frontend Build
echo -e "\n${BOLD}[Stage 2/4] Verifying Visuals, Links, Fallbacks & Frontend...${RESET}"
python3 "$REPO_ROOT/scripts/verify_visuals_and_docs.py"

# Stage 3: Rust Workspace Compilation & Syntax Check
echo -e "\n${BOLD}[Stage 3/4] Verifying Rust Workspace Compilation (cargo check)...${RESET}"
cargo check --workspace --all-targets

# Stage 4: Rust Unit & Critical Integration Tests
if [ "${1:-}" = "--full" ]; then
    echo -e "\n${BOLD}[Stage 4/4] Running Full Rust Workspace Test Suite (--full)...${RESET}"
    cargo test --workspace
else
    echo -e "\n${BOLD}[Stage 4/4] Running Rust Unit & Critical Integration Tests (including real_data)...${RESET}"
    cargo test --workspace --bins --lib --test real_data
fi

echo -e "\n${BOLD}${GREEN}======================================================${RESET}"
echo -e "${BOLD}${GREEN}   ✓ ALL PRE-COMMIT VERIFICATION CHECKS PASSED!       ${RESET}"
echo -e "${BOLD}${GREEN}   Commit is authorized to proceed.                   ${RESET}"
echo -e "${BOLD}${GREEN}======================================================${RESET}\n"
exit 0
