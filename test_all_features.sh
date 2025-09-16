#!/bin/bash

. "/usr/local/cargo/env"

DATABASES=("postgres" "mysql" "sqlite" "mssql" "any")
TOTAL=0
PASSED=0
FAILED=0

echo "Testing clippy for all tokio-rustls + database combinations..."

for DATABASE in "${DATABASES[@]}"; do
    ((TOTAL++))
    echo -n "Testing runtime-tokio-rustls + $DATABASE... "
    
    if cargo +nightly clippy -p sqlx-core-oldapi --no-default-features --features "runtime-tokio-rustls,$DATABASE" -- -D warnings >/dev/null 2>&1; then
        echo "✓"
        ((PASSED++))
    else
        echo "✗"
        ((FAILED++))
    fi
done

echo ""
echo "Summary: $PASSED/$TOTAL passed"

if [ $FAILED -eq 0 ]; then
    echo "✓ All tests passed!"
    exit 0
else
    echo "✗ $FAILED tests failed"
    exit 1
fi