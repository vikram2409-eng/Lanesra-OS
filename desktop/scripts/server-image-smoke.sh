#!/usr/bin/env bash
# Smoke test for the Team Workspace server image (WP-0020).
# Starts the image on an empty volume, completes first-run setup over HTTP,
# signs in, restarts the container and checks the data survived.
# Usage: desktop/scripts/server-image-smoke.sh <image-tag>
set -euo pipefail

IMAGE="${1:?usage: server-image-smoke.sh <image-tag>}"
NAME="lanesra-smoke-$$"
VOLUME="lanesra-smoke-data-$$"
PORT="${SMOKE_PORT:-18080}"
BASE="http://127.0.0.1:${PORT}"
JAR="$(mktemp)"

cleanup() {
  docker logs "$NAME" >"${SMOKE_LOG:-/tmp/lanesra-smoke.log}" 2>&1 || true
  docker rm -f "$NAME" >/dev/null 2>&1 || true
  docker volume rm "$VOLUME" >/dev/null 2>&1 || true
  rm -f "$JAR"
}
trap cleanup EXIT

start() {
  docker run -d --name "$NAME" -p "${PORT}:8080" -v "${VOLUME}:/data" "$IMAGE" >/dev/null
  for _ in $(seq 1 60); do
    if curl -fsS "$BASE/api/health" >/dev/null 2>&1; then return 0; fi
    sleep 1
  done
  echo "server did not become healthy within 60s" >&2
  exit 1
}

invoke() { # invoke <command> <json-args>
  curl -fsS -b "$JAR" -c "$JAR" -H 'Content-Type: application/json' \
    -X POST "$BASE/api/invoke/$1" -d "$2"
}

expect_ok() { # expect_ok <label> <json>
  if ! printf '%s' "$2" | python3 -c 'import json,sys; sys.exit(0 if json.load(sys.stdin).get("ok") is True else 1)'; then
    echo "FAIL: $1 -> $2" >&2
    exit 1
  fi
  echo "ok: $1"
}

start
echo "ok: health"

status=$(invoke workspace_status '{}')
expect_ok "workspace_status on an empty volume" "$status"

setup=$(invoke first_run_setup '{"setup":{"business_name":"Smoke Co","legal_name":null,"currency_code":"USD","locale":"en-US","timezone":"UTC","default_tax_rate_bp":0,"admin_username":"admin","admin_display_name":"Admin","admin_password":"smoke-test-password","load_sample_data":false}}')
expect_ok "first_run_setup" "$setup"

login=$(invoke login '{"credentials":{"username":"admin","password":"smoke-test-password"}}')
expect_ok "login" "$login"

created=$(invoke create_company '{"input":{"name":"Persisted Ltd","status":"Prospect"}}' || echo '{"ok":false}')
expect_ok "create a record" "$created"

docker restart "$NAME" >/dev/null
for _ in $(seq 1 60); do curl -fsS "$BASE/api/health" >/dev/null 2>&1 && break; sleep 1; done

rm -f "$JAR"; JAR="$(mktemp)"
login=$(invoke login '{"credentials":{"username":"admin","password":"smoke-test-password"}}')
expect_ok "login after restart" "$login"

companies=$(invoke list_companies '{}')
expect_ok "list records after restart" "$companies"
if ! printf '%s' "$companies" | grep -q 'Persisted Ltd'; then
  echo "FAIL: record created before the restart is missing" >&2
  exit 1
fi
echo "ok: data survived a container restart"
echo "server image smoke test passed"
