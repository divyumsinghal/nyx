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
# Passwords come from .secrets/bootstrap.env (required — no defaults).
# All passwords must be set before first postgres startup.
# =============================================================================

set -euo pipefail

NYX_APP_PASSWORD="${NYX_APP_DB_PASSWORD:?NYX_APP_DB_PASSWORD must be set in .secrets/bootstrap.env}"
NYX_MIGRATION_PASSWORD="${NYX_MIGRATION_DB_PASSWORD:?NYX_MIGRATION_DB_PASSWORD must be set in .secrets/bootstrap.env}"
KRATOS_PASSWORD="${KRATOS_DB_PASSWORD:?KRATOS_DB_PASSWORD must be set in .secrets/bootstrap.env}"
INFISICAL_PASSWORD="${INFISICAL_DB_PASSWORD:?INFISICAL_DB_PASSWORD must be set in .secrets/bootstrap.env}"

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

-- Default privileges for tables created by postgres superuser
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

-- Default privileges for tables created by nyx_migration role.
-- Must come AFTER the role is created above.
-- Migrations run as nyx_migration, not as postgres, so both sets are needed.
ALTER DEFAULT PRIVILEGES FOR ROLE nyx_migration IN SCHEMA nyx
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO nyx_app;
ALTER DEFAULT PRIVILEGES FOR ROLE nyx_migration IN SCHEMA nyx
    GRANT USAGE, SELECT ON SEQUENCES TO nyx_app;

ALTER DEFAULT PRIVILEGES FOR ROLE nyx_migration IN SCHEMA "Uzume"
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO nyx_app;
ALTER DEFAULT PRIVILEGES FOR ROLE nyx_migration IN SCHEMA "Uzume"
    GRANT USAGE, SELECT ON SEQUENCES TO nyx_app;

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
-- PostgreSQL 15+ revoked default CREATE on public schema from PUBLIC.
-- Kratos migrations create tables in the public schema of the kratos DB,
-- so kratos_app needs explicit schema-level CREATE/USAGE.
GRANT ALL ON SCHEMA public TO kratos_app;

SQL

# ── infisical database roles ──────────────────────────────────────────────────
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "infisical" <<SQL

DO \$\$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'infisical_app') THEN
    CREATE ROLE infisical_app WITH LOGIN ENCRYPTED PASSWORD '${INFISICAL_PASSWORD}';
  END IF;
END
\$\$;

GRANT ALL PRIVILEGES ON DATABASE infisical TO infisical_app;
GRANT ALL ON SCHEMA public TO infisical_app;

SQL

echo "▶ PostgreSQL roles provisioned successfully."
