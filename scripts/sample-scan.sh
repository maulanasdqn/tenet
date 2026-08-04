#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'TEXT'
usage: scripts/sample-scan.sh [target] [engine]

  target   url to scan       (default https://shopee.co.id)
  engine   http | browser    (default browser)

environment:
  TENET_API      gateway base url (default http://127.0.0.1:8080)
  API_KEY        gateway api key  (default dev-key)
  MAX_SCRIPTS    script budget    (default 10)
  OUT_DIR        where the openapi document is written (default .)

the gateway and analyzer must already be running.
TEXT
  exit 2
}

case "${1:-}" in -h | --help) usage ;; esac

TARGET=${1:-https://shopee.co.id}
ENGINE=${2:-browser}
API=${TENET_API:-http://127.0.0.1:8080}
KEY=${API_KEY:-dev-key}
MAX_SCRIPTS=${MAX_SCRIPTS:-10}
OUT_DIR=${OUT_DIR:-.}

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

get() { curl -fsS "$API$1" -H "x-api-key: $KEY"; }

if ! curl -fsS -o /dev/null "$API/healthz"; then
  printf 'tenet-gateway is not answering at %s\n' "$API" >&2
  exit 1
fi

curl -fsS -X POST "$API/v1/scans" \
  -H "x-api-key: $KEY" -H 'content-type: application/json' \
  -d "{\"target\":\"$TARGET\",\"engine\":\"$ENGINE\",\"max_scripts\":$MAX_SCRIPTS}" \
  >"$work/created.json"

scan_id=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["scan_id"])' "$work/created.json")
printf 'scan %s  target=%s  engine=%s\n' "$scan_id" "$TARGET" "$ENGINE"
printf 'waiting'

status=queued
for _ in $(seq 1 90); do
  get "/v1/scans/$scan_id" >"$work/scan.json"
  status=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["status"])' "$work/scan.json")
  case "$status" in
    succeeded | failed) break ;;
    *)
      printf '.'
      sleep 2
      ;;
  esac
done
printf '\n'

if [ "$status" != "succeeded" ]; then
  printf 'scan %s: ' "$status" >&2
  python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["error"])' "$work/scan.json" >&2
  exit 1
fi

get "/v1/scans/$scan_id/findings" >"$work/findings.json"
get "/v1/scans/$scan_id/endpoints" >"$work/endpoints.json"

python3 - "$work/findings.json" "$work/endpoints.json" <<'REPORT'
import json
import sys

findings = json.load(open(sys.argv[1]))["data"]
endpoints = json.load(open(sys.argv[2]))["data"]


def section(title):
    print(f"\n=== {title} ===")


section("technology")
for row in [f for f in findings if f["kind"] == "technology"]:
    name, category = row["name"], str(row["value"])
    print(f"  {name:24s} {category:12s} {row['confidence']:.2f}  via {row['evidence']}")

section("auth")
for row in [f for f in findings if f["kind"] == "auth"]:
    value = str(row["value"])[:44]
    print(f"  {row['name']:16s} {value:46s} {row['confidence']:.2f}")

section("endpoints")
for row in endpoints:
    origin = row["base_url"] or ""
    kind = "runtime" if row["source"] == "runtime" else "static"
    line = f"  {row['method'].upper():6s} {origin}{row['path']}"
    print(f"{line[:92]}  [{kind} {row['confidence']:.2f}]")

observed = sum(1 for row in endpoints if row["source"] == "runtime")
print(f"  -- {len(endpoints)} endpoints, {observed} observed at runtime")
REPORT

spec="$OUT_DIR/openapi-$scan_id.json"
get "/v1/scans/$scan_id/openapi" >"$spec"
printf '\nopenapi written to %s\n' "$spec"
