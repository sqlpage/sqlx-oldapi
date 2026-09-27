#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

backend=${1:-all}
if [[ $backend == all ]]; then
    for backend in postgres mssql mysql sqlite odbc; do
        bash ./test.sh "$backend"
    done
    exit 0
fi

features="any,macros,all-types,rustls"
case "$backend" in
    sqlite)
        sqlite_dir=$(mktemp -d "${TMPDIR:-/tmp}/sqlx-local-sqlite.XXXXXX")
        trap 'rm -rf -- "$sqlite_dir"' EXIT
        cp tests/sqlite/sqlite.db "$sqlite_dir/sqlite.db"
        export DATABASE_URL="sqlite://$sqlite_dir/sqlite.db"
        cargo test --locked --no-default-features --features "$features,sqlite,migrate"
        exit
        ;;
    postgres) service=postgres_14; port=5432; driver=postgres ;;
    mysql) service=mysql_8; port=3306; driver=mysql ;;
    mssql) service=mssql_2022; port=1433; driver=mssql ;;
    odbc) service=postgres_16_no_ssl; port=5432; driver=postgres ;;
    *)
        echo "Usage: $0 [all|sqlite|postgres|mysql|mssql|odbc]" >&2
        exit 2
        ;;
esac

project="sqlx-local-test-$$-$RANDOM"
cleanup() {
    status=$?
    docker compose -p "$project" -f tests/docker-compose.yml \
        down --remove-orphans >/dev/null 2>&1 || true
    exit "$status"
}
trap cleanup EXIT

# Each invocation owns one disposable container and a random loopback-only port.
container=$(docker compose -p "$project" -f tests/docker-compose.yml \
    run --rm -d -p "127.0.0.1::$port" "$service")
bash tests/wait-for-db.sh "$container" "$driver"
address=$(docker port "$container" "$port/tcp")
host_port=${address##*:}
if [[ ! $host_port =~ ^[0-9]+$ ]]; then
    echo "Could not determine the database's published port: $address" >&2
    exit 1
fi

case "$backend" in
    postgres) DATABASE_URL="postgres://postgres:password@127.0.0.1:$host_port/sqlx" ;;
    mysql) DATABASE_URL="mysql://root:password@127.0.0.1:$host_port/sqlx" ;;
    mssql) DATABASE_URL="mssql://sa:Password123!@127.0.0.1:$host_port/sqlx" ;;
    odbc)
        # Use only the container we started; do not overwrite ~/.odbc.ini or use a saved DSN.
        DATABASE_URL="Driver={PostgreSQL Unicode};Servername=127.0.0.1;Port=$host_port;Database=sqlx;Uid=postgres;Pwd=password"
        ;;
esac
export DATABASE_URL
if [[ $backend != odbc ]]; then features+=",migrate"; fi
cargo test --locked --no-default-features --features "$features,$backend"
