#!/usr/bin/env bash
set -euo pipefail

# Wait for SQL Server to be ready for connections
deadline=$((SECONDS + 120))
until /opt/mssql-tools18/bin/sqlcmd -S tcp:127.0.0.1,1433 -U sa -P "$SA_PASSWORD" -d master -Q "SELECT 1;" -No -b -l 2
do
  if ((SECONDS >= deadline)); then
    echo "SQL Server did not become ready within 120 seconds" >&2
    exit 1
  fi
  echo "Waiting for SQL Server to be ready..."
  sleep 1
done

# Run the setup script to create the DB and the schema in the DB
/opt/mssql-tools18/bin/sqlcmd -S tcp:127.0.0.1,1433 -U sa -P "$SA_PASSWORD" -d master -i setup.sql -No -b -l 2
