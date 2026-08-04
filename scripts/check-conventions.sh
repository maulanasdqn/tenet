#!/usr/bin/env bash
set -uo pipefail

cd "$(dirname "$0")/.."

MAX_LINES=200
status=0

report() {
  printf '%s\n' "$1" >&2
  status=1
}

sources=$(find tenet-*/src -type f -name '*.rs' | sort)

for file in $sources; do
  lines=$(wc -l <"$file" | tr -d '[:space:]')
  if [ "$lines" -gt "$MAX_LINES" ]; then
    report "$file:1 file is $lines lines, limit is $MAX_LINES"
  fi
done

comment_re='^[[:space:]]*(//|/\*)|[[:space:]]//'
for file in $sources; do
  while IFS=: read -r line _; do
    [ -n "$line" ] && report "$file:$line comment found, code must document itself"
  done < <(grep -nE "$comment_re" "$file" | cut -d: -f1 | sed 's/$/:/')
done

domain_re='(^|[^[:alnum:]_])(axum|reqwest|scraper|regex|sha2|chromiumoxide|tenet_browser)::|sqlx::(query|PgPool|Pool)|crate::(infrastructure|application)'
application_re='(^|[^[:alnum:]_])(axum|reqwest|scraper|sha2|chromiumoxide|tenet_browser)::|sqlx::(query|PgPool|Pool)|crate::infrastructure'

check_layer() {
  layer=$1
  pattern=$2
  for dir in tenet-*/src/"$layer"; do
    [ -d "$dir" ] || continue
    for file in $(find "$dir" -type f -name '*.rs' | sort); do
      while IFS= read -r hit; do
        [ -n "$hit" ] && report "$file:${hit%%:*} $layer layer must not reach into infrastructure"
      done < <(grep -nE "$pattern" "$file")
    done
  done
}

check_layer domain "$domain_re"
check_layer application "$application_re"

if [ "$status" -eq 0 ]; then
  printf 'conventions: ok (%s files)\n' "$(printf '%s\n' "$sources" | wc -l | tr -d '[:space:]')"
fi

exit "$status"
