#!/usr/bin/env bash
set -euo pipefail

app_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
workspace_root=$(cd -- "$app_root/.." && pwd)
postgres_bin="$workspace_root/.tools/pgsql/bin"
data_directory="$workspace_root/.cache/pgdata"
log_file="$workspace_root/.cache/postgres.log"

if [[ -x "$postgres_bin/pg_ctl.exe" ]]; then
  extension=.exe
elif [[ -x "$postgres_bin/pg_ctl" ]]; then
  extension=
else
  printf '%s\n' 'Portable PostgreSQL is unavailable. Use docker compose -f compose.development.yml up -d postgres.' >&2
  exit 1
fi

mkdir -p -- "$workspace_root/.cache"
if [[ ! -f "$data_directory/PG_VERSION" ]]; then
  "$postgres_bin/initdb$extension" --pgdata "$data_directory" --username teacher_oa --auth-host trust --auth-local trust --encoding UTF8 --locale C
fi
if ! "$postgres_bin/pg_ctl$extension" -D "$data_directory" status; then
  "$postgres_bin/pg_ctl$extension" -D "$data_directory" -l "$log_file" -o '-h 127.0.0.1 -p 55432' -w start
fi
for database in teacher_oa teacher_oa_test; do
  exists=$("$postgres_bin/psql$extension" -h 127.0.0.1 -p 55432 -U teacher_oa -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname='$database'")
  exists=${exists//$'\r'/}
  if [[ "$exists" != 1 ]]; then
    "$postgres_bin/createdb$extension" -h 127.0.0.1 -p 55432 -U teacher_oa "$database"
  fi
done
printf '%s\n' 'TeacherOA PostgreSQL is ready at 127.0.0.1:55432.'
