#!/usr/bin/env bash
set -euo pipefail

DOMAIN="apps/backend/src/domain"
APP="apps/backend/src/application"
PORTS="apps/backend/src/ports"

FORBIDDEN_IN_DOMAIN=("axum" "sqlx" "tokio" "serde")
FORBIDDEN_IN_APP=("axum" "sqlx")
FORBIDDEN_IN_PORTS=("axum" "sqlx")

check() {
    local dir="$1"
    shift
    local forbidden=("$@")
    local found=0

    for pkg in "${forbidden[@]}"; do
        matches=$(grep -rn "use ${pkg}" "$dir" 2>/dev/null || true)
        if [[ -n "$matches" ]]; then
            echo "❌ BOUNDARY VIOLATION: '$pkg' imported in $dir"
            echo "$matches"
            found=1
        fi
    done
    return $found
}

echo "Checking hexagonal layer boundaries..."
check "$DOMAIN" "${FORBIDDEN_IN_DOMAIN[@]}"
check "$APP"    "${FORBIDDEN_IN_APP[@]}"
check "$PORTS"  "${FORBIDDEN_IN_PORTS[@]}"
echo "✅ All boundary checks passed"
