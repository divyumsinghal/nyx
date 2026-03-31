#!/usr/bin/env bash
# =============================================================================
# 02_roles.sh
# Creates least-privilege PostgreSQL roles for each concern.
# Runs as the postgres superuser on first container startup.
#
# Roles created:
#   nyx_app       – application service connections (SELECT/INSERT/UPDATE/DELETE)
#   nyx_migration – xtask migration runner (DDL privileges)
#   kratos_app    – Ory Kratos own database
#
# Passwords come from Docker environment variables injected at runtime.
# In production set these via Docker secrets / a secrets manager.
# Default values here are only for local development. NEVER deploy them.
# =============================================================================

set -euo pipefail

NYX_APP_PASSWORD="${NYX_APP_DB_PASSWORD:-nyx_app_dev_changeme}"
NYX_MIGRATION_PASSWORD="${NYX_MIGRATION_DB_PASSWORD:-nyx_migration_dev_changeme}"
KRATOS_PASSWORD="${KRATOS_DB_PASSWORD:-kratos_dev_changeme}"

# ── nyx database roles ────────────────────────────────────────────────────────
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "nyx" <<SQL

-- nyx_app: used by every microservice at runtime
-- Deliberately excluded: CREATE, DROP, TRUNCATE, REFERENCES, TRIGGER
DO \$\$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'nyx_app') THEN
    CREATE ROLE nyx_app WITH LOGIN ENCRYPTED PASSWORD '${NYX_APP_PASSWORD}';
  END IF;
END
\$\$;

GRANT CONNECT ON DATABASE nyx TO nyx_app;

-- Grant USAGE on schemas now; table-level grants flow from DEFAULT PRIVILEGES
-- as migrations create new tables.
GRANT USAGE ON SCHEMA nyx      TO nyx_app;
GRANT USAGE ON SCHEMA "Uzume"  TO nyx_app;

ALTER DEFAULT PRIVILEGES FOR ROLE ${POSTGRES_USER} IN SCHEMA nyx
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO nyx_app;
ALTER DEFAULT PRIVILEGES FOR ROLE ${POSTGRES_USER} IN SCHEMA nyx
    GRANT USAGE, SELECT ON SEQUENCES TO nyx_app;

ALTER DEFAULT PRIVILEGES FOR ROLE ${POSTGRES_USER} IN SCHEMA "Uzume"
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO nyx_app;
ALTER DEFAULT PRIVILEGES FOR ROLE ${POSTGRES_USER} IN SCHEMA "Uzume"
    GRANT USAGE, SELECT ON SEQUENCES TO nyx_app;

-- nyx_migration: used only by nyx-xtask (migrate / db-reset commands)
-- Granted full DDL so migrations can CREATE TABLE, ALTER, etc.
DO \$\$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'nyx_migration') THEN
    CREATE ROLE nyx_migration WITH LOGIN ENCRYPTED PASSWORD '${NYX_MIGRATION_PASSWORD}';
  END IF;
END
\$\$;

GRANT ALL PRIVILEGES ON DATABASE nyx TO nyx_migration;
GRANT ALL ON SCHEMA nyx     TO nyx_migration;
GRANT ALL ON SCHEMA "Uzume" TO nyx_migration;

SQL

# ── kratos database roles ─────────────────────────────────────────────────────
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "kratos" <<SQL

DO \$\$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'kratos_app') THEN
    CREATE ROLE kratos_app WITH LOGIN ENCRYPTED PASSWORD '${KRATOS_PASSWORD}';
  END IF;
END
\$\$;

GRANT ALL PRIVILEGES ON DATABASE kratos TO kratos_app;

SQL

echo "▶ PostgreSQL roles provisioned successfully."
