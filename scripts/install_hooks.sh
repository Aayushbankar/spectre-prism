#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

mkdir -p "$REPO_ROOT/.githooks"
mkdir -p "$REPO_ROOT/.git/hooks"

cat << 'EOF' > "$REPO_ROOT/.githooks/pre-commit"
#!/usr/bin/env bash
exec "$(git rev-parse --show-toplevel)/scripts/pre_commit_check.sh"
EOF

chmod +x "$REPO_ROOT/.githooks/pre-commit"
cp -f "$REPO_ROOT/.githooks/pre-commit" "$REPO_ROOT/.git/hooks/pre-commit"
chmod +x "$REPO_ROOT/.git/hooks/pre-commit"

# Also configure git to use .githooks if supported
git config core.hooksPath .githooks

echo "✓ Strict PRISM pre-commit hook installed to .git/hooks/pre-commit and configured via core.hooksPath=.githooks"
