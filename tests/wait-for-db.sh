#!/usr/bin/env bash
set -euo pipefail

container=${1:?usage: wait-for-db.sh CONTAINER postgres|mysql|mssql [TIMEOUT_SECONDS]}
driver=${2:?missing database driver}
wait_seconds=${3:-120}
if [[ ! $wait_seconds =~ ^[1-9][0-9]*$ ]]; then
    echo "Timeout must be a positive number of seconds" >&2
    exit 2
fi

# Check the test schema over TCP, not just the server process or temporary
# initialization socket. SQL Server creates its schema in a background script.
case "$driver" in
    postgres)
        probe=(env PGPASSWORD=password psql -h 127.0.0.1 -U postgres -d sqlx
            -v ON_ERROR_STOP=1 -c 'SELECT 1 FROM tweet LIMIT 0')
        ;;
    mysql)
        probe=(sh -c '
            client=$(command -v mariadb || command -v mysql)
            exec "$client" --protocol=TCP -h 127.0.0.1 -u root -ppassword -D sqlx \
                -e "SELECT 1 FROM tweet LIMIT 0"
        ')
        ;;
    mssql)
        probe=(bash -c '
            sqlcmd=/opt/mssql-tools18/bin/sqlcmd
            if [[ ! -x "$sqlcmd" ]]; then sqlcmd=/opt/mssql-tools/bin/sqlcmd; fi
            exec "$sqlcmd" -S tcp:127.0.0.1,1433 -U sa -P "$SA_PASSWORD" -d sqlx \
                -Q "SELECT TOP 0 * FROM dbo.tweet" -b -C -l 2
        ')
        ;;
    *)
        echo "Unsupported database driver: $driver" >&2
        exit 2
        ;;
esac

deadline=$((SECONDS + wait_seconds))
while ((SECONDS < deadline)); do
    remaining=$((deadline - SECONDS))
    if ((remaining <= 0)); then break; fi
    if ((remaining > 5)); then remaining=5; fi
    if timeout --foreground "${remaining}s" docker exec "$container" "${probe[@]}" >/dev/null 2>&1; then
        echo "$container test schema is ready"
        exit 0
    fi
    if [[ $(timeout --foreground 5s docker inspect --format '{{.State.Running}}' "$container" 2>/dev/null) != true ]]; then
        break
    fi
    sleep 1
done

echo "$container did not become ready within ${wait_seconds}s (or stopped)" >&2
timeout --foreground 5s docker logs --tail 100 "$container" >&2 || true
exit 1
