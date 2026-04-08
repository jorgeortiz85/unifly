#!/usr/bin/env bash
set -euo pipefail

controller_url="${UNIFLY_E2E_URL:-https://localhost:8443}"
username="${UNIFLY_E2E_USERNAME:-admin}"
password="${UNIFLY_E2E_PASSWORD:-admin}"
timeout_secs="${UNIFLY_E2E_TIMEOUT_SECS:-180}"
poll_secs="${UNIFLY_E2E_POLL_SECS:-5}"
status_url="${controller_url%/}/status"

deadline=$((SECONDS + timeout_secs))
cookie_jar="$(mktemp)"
trap 'rm -f "$cookie_jar"' EXIT

status_ready() {
    local body
    body="$(curl -ksS "$status_url")"

    printf '%s' "$body" | grep -q '"rc":"ok"' &&
        printf '%s' "$body" | grep -q '"up":true'
}

login_ready() {
    local payload
    payload=$(printf '{"username":"%s","password":"%s"}' "$username" "$password")

    for login_path in /api/login /api/auth/login; do
        local body
        body="$(
            curl -ksS \
                -c "$cookie_jar" \
                -H 'Content-Type: application/json' \
                -X POST \
                -d "$payload" \
                "${controller_url%/}${login_path}" || true
        )"

        if printf '%s' "$body" | grep -Eq '"(rc|meta)":"?ok"?|"meta":[^{]*\{[[:space:]]*"rc"[[:space:]]*:[[:space:]]*"ok"'; then
            return 0
        fi
    done

    return 1
}

printf 'Waiting for UniFi controller at %s\n' "$controller_url"

until status_ready; do
    if (( SECONDS >= deadline )); then
        printf 'Timed out waiting for %s\n' "$status_url" >&2
        exit 1
    fi
    sleep "$poll_secs"
done

printf 'Controller status endpoint is ready, verifying login...\n'

until login_ready; do
    if (( SECONDS >= deadline )); then
        printf 'Timed out waiting for login readiness at %s\n' "$controller_url" >&2
        exit 1
    fi
    sleep "$poll_secs"
done

printf 'Controller is ready for e2e tests\n'
