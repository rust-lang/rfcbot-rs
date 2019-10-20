#!/usr/bin/env bash

DATABASE_URL="${DATABASE_URL:-postgres://postgres:ughfineokifitsfordebugging@localhost:54320/rfcbot}"

until psql -d "$DATABASE_URL" -c '\q'; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up - executing commands"

set -e
diesel database setup
diesel migration run
set +e

sql_output="$(psql -q -d "$DATABASE_URL" --file ./githubuser-backup.pg)"
sql_status="$?"

if [ ! $sql_status ]; then
  echo "$sql_output"
  exit $sql_status
fi

exec "$@"