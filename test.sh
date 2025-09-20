#!/bin/sh
docker compose -f tests/docker-compose.yml run -it -p 5432:5432 --name postgres_16 postgres_16
DATABASE_URL="postgres://postgres@localhost:5432/sqlx?sslmode=verify-ca&sslrootcert=./tests/certs/ca.crt&sslcert=./tests/certs/client.crt&sslkey=./tests/keys/client.key" cargo test --features any,postgres,macros,all-types,runtime-actix-rustls -- 

docker compose -f tests/docker-compose.yml run -it -p 1433:1433 --rm --name mssql_2022 mssql_2022
DATABASE_URL='mssql://sa:Password123!@localhost/sqlx' cargo test --features any,mssql,macros,all-types,runtime-actix-rustls --

docker compose -f tests/docker-compose.yml run -it -p 3306:3306 --name mysql_8 mysql_8
DATABASE_URL='mysql://root:password@localhost/sqlx' cargo test --features any,mysql,macros,all-types,runtime-actix-rustls --

DATABASE_URL='sqlite://./tests/sqlite/sqlite.db' cargo test --features any,sqlite,macros,all-types,runtime-actix-rustls --


# Copy odbc config from tests/odbc.ini to ~/.odbc.ini
docker compose -f tests/docker-compose.yml run -it -p 5432:5432 --name postgres_16_no_ssl postgres_16_no_ssl
DATABASE_URL='DSN=SQLX_PG_5432;UID=postgres;PWD=password' cargo test --no-default-features --features any,odbc,all-types,macros,runtime-tokio-rustls --test odbc