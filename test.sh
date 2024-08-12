#!/bin/sh
docker-compose -f tests/docker-compose.yml run -d -p 5432:5432 --name postgres_16 postgres_16
DATABASE_URL="postgres://postgres@localhost:5432/sqlx?sslmode=verify-ca&sslrootcert=./tests/certs/ca.crt&sslcert=./tests/certs/client.crt&sslkey=./tests/keys/client.key" cargo test --features any,postgres,macros,all-types,runtime-actix-rustls -- numeric