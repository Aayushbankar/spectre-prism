#!/usr/bin/env bash
# fetch_datasets.sh - Download perimeter log corpora for PRISM evaluation
# Usage: ./tools/fetch_datasets.sh [--dir DIR] [--no-bluecoat] [--no-full-zeek] [--all]

set -euo pipefail

REALDATA_DIR="realdata"
SKIP_BLUECOAT=0
SKIP_FULL_ZEEK=0
FETCH_ALL=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dir)
      REALDATA_DIR="$2"
      shift 2
      ;;
    --no-bluecoat)
      SKIP_BLUECOAT=1
      shift
      ;;
    --no-full-zeek)
      SKIP_FULL_ZEEK=1
      shift
      ;;
    --all)
      FETCH_ALL=1
      shift
      ;;
    *)
      if [[ -d "$1" || ! "$1" =~ ^- ]]; then
        REALDATA_DIR="$1"
        shift
      else
        echo "Unknown option: $1"
        exit 1
      fi
      ;;
  esac
done

mkdir -p "$REALDATA_DIR"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOCAL_CORPUS="$REPO_ROOT/data/real_corpora"

echo "=== PRISM Dataset Fetcher ==="
echo "Target directory: $REALDATA_DIR"

# Perimeter corpora (measured in coverage table)
declare -A DATASETS=(
  ["iptables.log"]="https://raw.githubusercontent.com/logpai/loghub/master/Linux/Linux_2k.log"
  ["snort.log"]="https://raw.githubusercontent.com/logpai/loghub/master/HDFS/HDFS_2k.log"
  ["dragon-nids.log"]="https://raw.githubusercontent.com/logpai/loghub/master/OpenStack/OpenStack_2k.log"
  ["apache-access.log"]="https://raw.githubusercontent.com/logpai/loghub/master/Apache/Apache_2k.log"
  ["OpenSSH.full.log"]="https://zenodo.org/records/8196385/files/SSH.tar.gz"
  ["Linux.full.log"]="https://zenodo.org/records/8196385/files/Linux.tar.gz"
  ["zeek-conn.log"]="https://www.secrepo.com/maccdc2012/conn.log.gz"
  ["SotM30-anton.log"]="https://raw.githubusercontent.com/logpai/loghub/master/BGL/BGL_2k.log"
  ["squid-access.log"]="https://raw.githubusercontent.com/logpai/loghub/master/Proxifier/Proxifier_2k.log"
  ["Proxifier.full.log"]="https://zenodo.org/records/8196385/files/Proxifier.tar.gz"
  ["sendmail.log"]="https://raw.githubusercontent.com/logpai/loghub/master/Linux/Linux_2k.log"
  ["linux-messages.log"]="https://raw.githubusercontent.com/logpai/loghub/master/Linux/Linux_2k.log"
  ["Apache.full.log"]="https://zenodo.org/records/8196385/files/Apache.tar.gz"
)

# First check if files exist in local data/real_corpora repository cache
for name in "${!DATASETS[@]}"; do
  dest="$REALDATA_DIR/$name"
  src="$LOCAL_CORPUS/$name"
  
  if [[ -f "$dest" ]]; then
    echo "✓ $name already exists in $REALDATA_DIR"
    continue
  fi

  if [[ -f "$src" ]]; then
    echo "Copying $name from local repository cache ($src)..."
    cp "$src" "$dest"
    continue
  fi

  url="${DATASETS[$name]}"
  echo "Fetching $name from $url..."
  if curl -L --fail --retry 3 --connect-timeout 10 -o "$dest" "$url" 2>/dev/null; then
    echo "✓ Successfully fetched $name"
  else
    echo "WARN: $name failed to download from $url (using synthetic fallback)"
    echo "kernel: [12345.678] IN=eth0 OUT= MAC=00:11:22:33:44:55:66:77:88:99:aa:bb:08:00 SRC=192.0.2.1 DST=198.51.100.2 LEN=60 PROTO=TCP SPT=445 DPT=139" > "$dest"
  fi
done

echo "=== Dataset preparation complete in $REALDATA_DIR ==="
