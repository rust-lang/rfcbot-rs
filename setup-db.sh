#!/usr/bin/env bash

set -xe

export DATABASE_URL="postgres://postgres:ughfineokifitsfordebugging@localhost:54320/rfcbot"

until psql -d "$DATABASE_URL" -c '\q'; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up - executing commands"

diesel database setup
diesel migration run
psql -q -d "$DATABASE_URL" --file ./githubuser-backup.pg
