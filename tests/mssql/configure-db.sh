#!/usr/bin/env bash

# Wait for SQL Server to be ready for connections
until /opt/mssql-tools18/bin/sqlcmd -S localhost -U sa -P $SA_PASSWORD -d master -Q "SELECT 1;" -No
do
  echo "Waiting for SQL Server to be ready..."
  sleep 1
done

# Run the setup script to create the DB and the schema in the DB
/opt/mssql-tools18/bin/sqlcmd -S localhost -U sa -P $SA_PASSWORD -d master -i setup.sql -No
