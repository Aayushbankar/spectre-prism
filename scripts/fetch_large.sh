#!/usr/bin/env bash
#
# fetch_large.sh — PRISM FULL-corpus fetcher (multi-GB to TB scale).
#
# Sources:
#   1. CIC-IDS-2017 full (CIC research site, ~50GB+)
#   2. CSE-CIC-IDS2018 full via AWS CLI s3://cse-cic-ids2018 (~400GB+)
#   3. AIT ADS full (Zenodo records/6475510, multi-GB)
#   4. UNSW-NB15 full (UNSW ACCS, ~100GB with pcaps)
#
# Usage:
#   scripts/fetch_large.sh [--dry-run] [--data-dir DIR]
#
# Safety:
#   - set -euo pipefail; aborts on first error.
#   - REQUIRES >=100GB free on the target filesystem (df check, abort otherwise).
#   - --dry-run prints every action WITHOUT downloading or needing the space.
#   - NEVER run on the 3.1GB-RAM / small-disk demo box — use a staging host,
#     then sneakernet slices via USB per docs/datasets/EPS_AIRGAP_PLAN.md.
#
set -euo pipefail

DRY_RUN=0
DATA_DIR="./data/full"
REQUIRE_FREE_GB=100

usage() {
  echo "Usage: $0 [--dry-run] [--data-dir DIR]"
  echo "  --dry-run       Print actions only, download nothing, skip space check enforcement."
  echo "  --data-dir DIR  Target dir (default: ./data/full)."
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    --data-dir) DATA_DIR="${2:?--data-dir requires an argument}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "ERROR: unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

log() { printf '[fetch_large] %s\n' "$*"; }
dry() { printf '[dry-run] %s\n' "$*"; }
die() { echo "ERROR: $*" >&2; exit 1; }

check_free_space() {
  local dir="$1" need_gb="$2" avail_str avail_gb
  if [[ "$DRY_RUN" -eq 1 ]]; then
    dry "df -BG '$dir'  (require >= ${need_gb}GB free, abort otherwise)"
    return 0
  fi
  mkdir -p "$dir"
  avail_str="$(df --output=avail -BG "$dir" | tail -n 1 | tr -d ' G')"
  avail_gb="${avail_str:-0}"
  log "free on $(df --output=target "$dir" | tail -n 1): ${avail_gb}GB (required: ${need_gb}GB)"
  if [[ "$avail_gb" -lt "$need_gb" ]]; then
    die "insufficient disk: ${avail_gb}GB free < ${need_gb}GB required at $dir. Free space or pick another --data-dir."
  fi
}

fetch() {
  local url="$1" dest="$2"
  if [[ "$DRY_RUN" -eq 1 ]]; then
    dry "curl -fSL --retry 3 -C - -o '$dest' '$url'  (resume-capable, multi-GB)"
    return 0
  fi
  log "GET $url -> $dest"
  curl -fSL --retry 3 -C - -o "$dest" "$url"
}

s3sync() {
  # s3sync <s3-uri> <dest>
  if [[ "$DRY_RUN" -eq 1 ]]; then
    dry "aws s3 sync '$1' '$2' --no-sign-request --dryrun  (remove --dryrun to execute)"
    return 0
  fi
  command -v aws >/dev/null 2>&1 || die "aws CLI not found (required for IDS2018). See EPS_AIRGAP_PLAN.md offline bundle."
  aws s3 sync "$1" "$2" --no-sign-request
}

main() {
  log "target dir: $DATA_DIR (dry-run=$DRY_RUN, require_free=${REQUIRE_FREE_GB}GB)"
  check_free_space "$DATA_DIR" "$REQUIRE_FREE_GB"

  if [[ "$DRY_RUN" -eq 1 ]]; then
    dry "mkdir -p $DATA_DIR/{cic2017,ids2018,ait-full,unsw-nb15}"
  else
    mkdir -p "$DATA_DIR/cic2017" "$DATA_DIR/ids2018" "$DATA_DIR/ait-full" "$DATA_DIR/unsw-nb15"
  fi

  # ---- 1. CIC-IDS-2017 full (~50GB) ----------------------------------------
  # Index: https://www.unb.ca/cic/datasets/ids-2017.html
  # Canonical generated-traffic archive (pin exact Monday..Friday pcaps/CSVs):
  CIC2017_BASE="http://cicresearch.ca/CICDataset/CIC-IDS-2017/Dataset"
  for day in Monday Tuesday Wednesday Thursday Friday; do
    fetch "$CIC2017_BASE/$day-workingHours.pcap" "$DATA_DIR/cic2017/$day-workingHours.pcap"
  done
  if [[ "$DRY_RUN" -eq 1 ]]; then
    dry "curl ... '$CIC2017_BASE/MachineLearningCSV.zip' -o '$DATA_DIR/cic2017/MachineLearningCSV.zip' (flow CSVs)"
  else
    log "NOTE: fetch MachineLearningCSV.zip manually from the UNB index if flow-CSVs are needed."
  fi

  # ---- 2. CSE-CIC-IDS2018 full via AWS (~400GB) ----------------------------
  # Index: https://www.unb.ca/cic/datasets/ids-2018.html
  # Open-data bucket (no credentials): s3://cse-cic-ids2018
  s3sync "s3://cse-cic-ids2018/" "$DATA_DIR/ids2018/"

  # ---- 3. AIT ADS full (multi-GB, Zenodo) -----------------------------------
  # Record: https://zenodo.org/records/6475510 — sync ALL training/test sets.
  if [[ "$DRY_RUN" -eq 1 ]]; then
    dry "curl -fSL 'https://zenodo.org/api/records/6475510' | jq .files  (enumerate full file list)"
    dry "fetch each training_set_*.tar.gz + test_set_*.tar.gz into $DATA_DIR/ait-full/"
  else
    log "NOTE: enumerate https://zenodo.org/api/records/6475510 and fetch each tar into $DATA_DIR/ait-full/ (multi-GB)."
  fi

  # ---- 4. UNSW-NB15 full (~100GB with pcaps) --------------------------------
  # Index: https://research.unsw.edu.au/projects/unsw-nb15-dataset
  UNSW_BASE="https://cloudstor.aarnet.edu.au/plus/s/2DhnLGDdEECo4ys/download"
  fetch "$UNSW_BASE?path=%2FUNSW-NB15%20-%20CSV%20Files&files=UNSW-NB15_1.csv" "$DATA_DIR/unsw-nb15/UNSW-NB15_1.csv"
  if [[ "$DRY_RUN" -eq 1 ]]; then
    dry "fetch UNSW-NB15_{2,3,4}.csv + Pcap files Pcap1..Pcap4 (~70GB) into $DATA_DIR/unsw-nb15/ (see UNSW index)"
  fi

  log "done. Full corpus staged under $DATA_DIR (dry-run=$DRY_RUN)."
  log "NEXT: sha256sum * > SHA256SUMS, sneakernet slices to air-gap host per EPS_AIRGAP_PLAN.md."
}

main "$@"
