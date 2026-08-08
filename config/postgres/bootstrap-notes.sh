#!/bin/sh
set -eu

for password in "${NOTES_APP_PASSWORD:-}" "${NOTES_MIGRATOR_PASSWORD:-}"; do
  case "$password" in
    ''|*[!A-Za-z0-9_~.-]*) echo "Notes database passwords must be nonempty and URL-safe" >&2; exit 2 ;;
  esac
done
export PGPASSWORD="$POSTGRES_PASSWORD"

psql -v ON_ERROR_STOP=1 -h halcyon-postgres -U "$POSTGRES_USER" -d postgres <<SQL
DO \$\$ BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname='notes_app') THEN CREATE ROLE notes_app LOGIN PASSWORD '${NOTES_APP_PASSWORD}'; END IF;
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname='notes_migrator') THEN CREATE ROLE notes_migrator LOGIN PASSWORD '${NOTES_MIGRATOR_PASSWORD}'; END IF;
END \$\$;
SQL

if ! psql -h halcyon-postgres -U "$POSTGRES_USER" -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname='notes'" | grep -q 1; then
  psql -v ON_ERROR_STOP=1 -h halcyon-postgres -U "$POSTGRES_USER" -d postgres -c "CREATE DATABASE notes OWNER notes_migrator"
fi

psql -v ON_ERROR_STOP=1 -h halcyon-postgres -U "$POSTGRES_USER" -d notes <<'SQL'
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS vector;
REVOKE CREATE ON SCHEMA public FROM PUBLIC;
REVOKE ALL ON DATABASE notes FROM PUBLIC;
GRANT CONNECT ON DATABASE notes TO notes_app, notes_migrator;
GRANT USAGE ON SCHEMA public TO notes_app;
SQL
