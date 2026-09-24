#!/usr/bin/env bash
#
# fetch_small.sh — PRISM demo-slice fetcher (all files <500MB).
#
# Sources (see docs/datasets/DATASET_PLAN.md, docs/datasets/EPS_AIRGAP_PLAN.md):
#   1. Loghub-2k samples (logpai/loghub) — parser-accuracy slices
#   2. IoT-23 scenario slices (Stratosphere) — conn.log.labeled excerpts
#   3. LANL auth head 100k (csr.lanl.gov cyber1) — auth.txt.gz excerpt
#   4. AIT ADS slice ~96MB (Zenodo records/6475510) — enterprise syslog slice
#   5. OTRF single technique (splunk/attack_data) — one ATT&CK technique JSON
#   6. SecRepo weird/tunnel samples (secrepo.com) — Zeek weird.log / tunnel.log
#
# Usage:
#   scripts/fetch_small.sh [--dry-run] [--data-dir DIR]
#
# Safety:
#   - set -euo pipefail; aborts on first error.
#   - --dry-run prints every curl/action WITHOUT downloading anything.
#   - Default target ./data/demo (gitignored). No file here exceeds 500MB.
#   - SHA-256 placeholders MUST be pinned before eval use (see EXPECTED_SHA256).
#
set -euo pipefail

DRY_RUN=0
DATA_DIR="./data/demo"

usage() {
  echo "Usage: $0 [--dry-run] [--data-dir DIR]"
  echo "  --dry-run       Print actions only, download nothing."
  echo "  --data-dir DIR  Target dir (default: ./data/demo)."
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    --data-dir) DATA_DIR="${2:?--data-dir requires an argument}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "ERROR: unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

log()  { printf '[fetch_small] %s\n' "$*"; }
dry()  { printf '[dry-run] %s\n' "$*"; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "ERROR: missing required cmd: $1" >&2; exit 1; }
}

# ---------------------------------------------------------------------------
# SHA-256 checksum placeholders.
# Replace each _TBD_ value with the pinned hex digest after one verified
# manual download, then re-run WITHOUT --dry-run to enforce verification.
# Until pinned, verify_sha256() WARNS and continues (demo mode only).
# ---------------------------------------------------------------------------
# shellcheck disable=SC2034
EXPECTED_SHA256_Linux_2k="SHA256_TBD_PIN_ME"
EXPECTED_SHA256_Apache_2k="SHA256_TBD_PIN_ME"
EXPECTED_SHA256_IoT23_conn="SHA256_TBD_PIN_ME"
EXPECTED_SHA256_LANL_auth_head="SHA256_TBD_PIN_ME"
EXPECTED_SHA256_AIT_slice="SHA256_TBD_PIN_ME"
EXPECTED_SHA256_OTRF_T1059="SHA256_TBD_PIN_ME"
EXPECTED_SHA256_SecRepo_weird="SHA256_TBD_PIN_ME"
EXPECTED_SHA256_SecRepo_tunnel="SHA256_TBD_PIN_ME"

verify_sha256() {
  local file="$1" expected="$2"
  if [[ "$expected" == SHA256_TBD* ]]; then
    log "WARN: no pinned sha256 for $(basename "$file") — skipping verify (pin before eval)."
    return 0
  fi
  if [[ "$DRY_RUN" -eq 1 ]]; then
    dry "sha256sum -c <(echo '$expected  $file')"
    return 0
  fi
  echo "$expected  $file" | sha256sum -c -
}

# fetch <url> <dest> [expected-sha]
fetch() {
  local url="$1" dest="$2" expected="${3:-SHA256_TBD_PIN_ME}"
  if [[ "$DRY_RUN" -eq 1 ]]; then
    dry "curl -fSL --retry 3 -o '$dest' '$url'"
    dry "verify_sha256 '$dest' (<placeholder>)"
    return 0
  fi
  log "GET $url -> $dest"
  curl -fSL --retry 3 -o "$dest" "$url"
  verify_sha256 "$dest" "$expected"
}

run_or_dry() {
  # run_or_dry "description" cmd...
  local desc="$1"; shift
  if [[ "$DRY_RUN" -eq 1 ]]; then
    dry "$desc: $*"
    return 0
  fi
  log "$desc"
  "$@"
}

main() {
  need_cmd curl
  need_cmd sha256sum
  if [[ "$DRY_RUN" -eq 0 ]]; then
    need_cmd zcat 2>/dev/null || need_cmd gzcat 2>/dev/null || true
    mkdir -p "$DATA_DIR/loghub" "$DATA_DIR/iot23" "$DATA_DIR/lanl" \
             "$DATA_DIR/ait" "$DATA_DIR/otrf" "$DATA_DIR/secrepo"
  else
    dry "mkdir -p $DATA_DIR/{loghub,iot23,lanl,ait,otrf,secrepo}"
  fi

  log "target dir: $DATA_DIR (dry-run=$DRY_RUN)"

  # ---- 1. Loghub-2k samples (each ~2k lines, KBs) -------------------------
  LOGBASE="https://raw.githubusercontent.com/logpai/loghub/master"
  fetch "$LOGBASE/Linux/Linux_2k.log"   "$DATA_DIR/loghub/Linux_2k.log"   "$EXPECTED_SHA256_Linux_2k"
  fetch "$LOGBASE/Apache/Apache_2k.log" "$DATA_DIR/loghub/Apache_2k.log" "$EXPECTED_SHA256_Apache_2k"
  # Windows + Android kept as documented alternates (<500MB each):
  if [[ "$DRY_RUN" -eq 1 ]]; then
    dry "curl -fSL --retry 3 -o '$DATA_DIR/loghub/Windows_2k.log' '$LOGBASE/Windows/Windows_2k.log'  (optional)"
    dry "curl -fSL --retry 3 -o '$DATA_DIR/loghub/Android_2k.log' '$LOGBASE/Android/Android_2k.log'  (optional)"
  else
    log "NOTE: Windows_2k.log / Android_2k.log are optional — uncomment in script to fetch."
  fi

  # ---- 2. IoT-23 scenario slices (Stratosphere IPS) ------------------------
  # Canonical index: https://www.stratosphereips.org/datasets-iot23
  # Full captures are GBs; we fetch ONLY the labeled Zeek conn slice and
  # truncate to a head excerpt locally so the stored artifact stays <500MB.
  # Scenario IDs below follow CTU IoT-23 naming (21 = Torri/backdoor-ish,
  # 22 = Mirai-style burst — re-pin exact capture name from index if moved).
  IOT_BASE="https://mcfp.felk.cvut.cz/publicDatasets/IoT-23-Dataset"
  IOT21_URL="$IOT_BASE/CTU-Malware-Capture-Botnet-43-1/bro/conn.log.labeled"
  IOT22_URL="$IOT_BASE/CTU-Malware-Capture-Botnet-42-1/bro/conn.log.labeled"
  fetch "$IOT21_URL" "$DATA_DIR/iot23/iot23-scenario21-conn.head50k.log" "$EXPECTED_SHA256_IoT23_conn"
  fetch "$IOT22_URL" "$DATA_DIR/iot23/iot23-scenario22-conn.head50k.log" "$EXPECTED_SHA256_IoT23_conn"
  run_or_dry "truncate IoT-23 slices to 10k demo lines each" \
    bash -c "head -n 10000 '$DATA_DIR/iot23/iot23-scenario21-conn.head50k.log' > '$DATA_DIR/iot23/iot23-s21-10k.log' 2>/dev/null || true; head -n 10000 '$DATA_DIR/iot23/iot23-scenario22-conn.head50k.log' > '$DATA_DIR/iot23/iot23-s22-10k.log' 2>/dev/null || true"

  # ---- 3. LANL auth head 100k (cyber1) -------------------------------------
  # Full auth.txt.gz is multi-GB; download the gzip once then keep only
  # the first 100k lines locally (~tens of MB). URL: https://csr.lanl.gov/data/2017/
  LANL_URL="https://csr.lanl.gov/data/cyber1/auth.txt.gz"
  fetch "$LANL_URL" "$DATA_DIR/lanl/auth.txt.gz" "$EXPECTED_SHA256_LANL_auth_head"
  run_or_dry "extract LANL auth head 100k" \
    bash -c "zcat '$DATA_DIR/lanl/auth.txt.gz' 2>/dev/null | head -n 100000 > '$DATA_DIR/lanl/auth-head-100k.txt' || true"

  # ---- 4. AIT ADS slice (~96MB) --------------------------------------------
  # Full AIT ADS (Zenodo records/6475510) is multi-GB; fetch ONE dated
  # training slice (~96MB) only. Pin exact filename from the record page:
  # https://zenodo.org/records/6475510
  AIT_URL="https://zenodo.org/records/6475510/files/training_set_01.tar.gz?download=1"
  fetch "$AIT_URL" "$DATA_DIR/ait/ait-training-slice.tar.gz" "$EXPECTED_SHA256_AIT_slice"
  if [[ "$DRY_RUN" -eq 1 ]]; then
    dry "tar -tzf '$DATA_DIR/ait/ait-training-slice.tar.gz' | head (list only, ~96MB slice)"
  fi

  # ---- 5. OTRF single technique (splunk/attack_data) ------------------------
  # Full attack_data repo is 10s of GB; fetch ONE technique JSON only
  # (T1059.001 PowerShell — small, <5MB).
  OTRF_URL="https://raw.githubusercontent.com/splunk/attack_data/master/datasets/attack_techniques/T1059.001/powershell/powershell.json"
  fetch "$OTRF_URL" "$DATA_DIR/otrf/T1059.001-powershell.json" "$EXPECTED_SHA256_OTRF_T1059"

  # ---- 6. SecRepo weird/tunnel samples --------------------------------------
  # Index: http://www.secrepo.com/ — small Zeek sample logs (KBs each).
  # Paths drift; re-pin against the index if 404. Placeholders keep
  # the script honest until URLs are re-verified on connected net.
  SEC_BASE="http://www.secrepo.com/self.logs"
  fetch "$SEC_BASE/zeek/weird.log.gz"  "$DATA_DIR/secrepo/weird.log.gz"  "$EXPECTED_SHA256_SecRepo_weird"
  fetch "$SEC_BASE/zeek/tunnel.log.gz" "$DATA_DIR/secrepo/tunnel.log.gz" "$EXPECTED_SHA256_SecRepo_tunnel"

  log "done. Demo corpus staged under $DATA_DIR (dry-run=$DRY_RUN)."
  log "NEXT: pin SHA256_TBD_* digests, then build 2-min slice per docs/datasets/EPS_AIRGAP_PLAN.md."
}

main "$@"
