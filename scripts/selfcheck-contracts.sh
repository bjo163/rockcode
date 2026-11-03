#!/usr/bin/env sh
set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

cd "$REPO_ROOT"

K=5
DETECTED=0

# ---- extract pristine workdir for mutation testing ----
git archive HEAD | tar -x -C "$WORKDIR"

# ---- helper: run validator from a given directory ----
run_validator() {
  local dir="$1"
  local out
  out="$(cd "$dir" && node "$REPO_ROOT/scripts/validate-contracts.mjs" 2>&1)"
  local rc=$?
  printf '%s' "$out"
  return $rc
}

# ---- helper: apply a mutation to a copy of the pristine workdir ----
apply_mut() {
  local dir="$1"
  shift
  local mut="$1"
  shift
  case "$mut" in
    "delete_tag")
      # (a) delete a tag description
      sed -i '/description: Process liveness and dependency\/readiness probes; unauthenticated so orchestrators and the desktop host can poll them./d' "$dir/contracts/openapi/v1/openapi.yaml"
      ;;
    "remove_4xx")
      # (b) remove getLiveness's 429 block
      node -e "
const fs=require('fs');const p=require('path').join('$dir','contracts/openapi/v1/openapi.yaml');let s=fs.readFileSync(p,'utf8');const block=\"        '429':\n          \\$ref: '#/components/responses/TooManyRequests'\\n\";s=s.replace(block,'');fs.writeFileSync(p,s);
" "$dir/contracts/openapi/v1/openapi.yaml"
      ;;
    "rename_key")
      # (c) rename a required key in one positive fixture
      node -e "
const fs=require('fs');const p=require('path').join('$dir','contracts/fixtures/v1/approval-request.valid.json');let s=fs.readFileSync(p,'utf8');s=s.replace('\"approval_id\"','\"approval_id_renamed\"');fs.writeFileSync(p,s);
" "$dir/contracts/fixtures/v1/approval-request.valid.json"
      ;;
    "add_prop")
      # (d) add a property absent from additionalProperties:false to a schema
      node -e "
const fs=require('fs');const p=require('path').join('$dir','contracts/fixtures/v1/session.valid.json');let s=fs.readFileSync(p,'utf8');s=s.replace('\"session_id\"','\"session_id\", \"extra_prop\"');fs.writeFileSync(p,s);
" "$dir/contracts/fixtures/v1/session.valid.json"
      ;;
    "break_link")
      # (e) break a repository-relative Markdown link in docs/specs/m0/README.md
      sed -i 's|docs/specs/m0/README.md|docs/specs/m0/README-broken.md|' "$dir/docs/specs/m0/README.md"
      ;;
  esac
}

# ---- run all 5 mutations ----
for i in 1 2 3 4 5; do
  CASE="$(mktemp -d)"
  git archive HEAD | tar -x -C "$CASE"

  # Choose mutation by index
  case $i in
    1) mut="delete_tag" ;;   # delete a tag description
    2) mut="remove_4xx" ;;   # remove getLiveness's 429 block
    3) mut="rename_key" ;;   # rename required key in positive fixture
    4) mut="add_prop" ;;     # add property forbidden by additionalProperties:false
    5) mut="break_link" ;;   # break a markdown relative link
  esac

  apply_mut "$CASE" "$mut"

  # Run validator from the mutated copy
  out="$(run_validator "$CASE")"
  rc=$?

  if [ $rc -ne 0 ] && printf '%s' "$out" | grep -qE "FAIL [1-9]"; then
    DETECTED=$((DETECTED+1))
    echo "mutation $i: DETECTED"
  else
    echo "mutation $i: NOT DETECTED (rc=$rc)"
    printf '%s\n' "$out" | tail -3
  fi

  rm -rf "$CASE"
done

# Baseline: re-extract pristine and verify
git archive HEAD | tar -x -C "$WORKDIR"
out="$(run_validator "$WORKDIR")"
rc=$?

if [ $rc -eq 0 ] && printf '%s' "$out" | grep -qE "PASS [0-9]+ / FAIL 0"; then
  echo "baseline: $(printf '%s' "$out" | grep -oE "PASS [0-9]+ / FAIL 0")"
else
  echo "baseline: FAILED (rc=$rc)"
  printf '%s\n' "$out" | tail -10
  exit 1
fi

echo "mutation self-check: $DETECTED/$K detected"
if [ "$DETECTED" -lt "$K" ]; then
  exit 1
fi
exit 0