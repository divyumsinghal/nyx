-- =============================================================================
-- 01_extensions_and_databases.sql
-- Runs on first PostgreSQL container startup (POSTGRES_DB=nyx by default).
-- Creates the Kratos auth database and enables required extensions.
-- =============================================================================

-- ── Kratos gets its own isolated database ────────────────────────────────────
-- Keeping identity data (Kratos) separate from app data (nyx schema) is a
-- deliberate security boundary: a compromised app credential cannot read
-- raw identity rows.
CREATE DATABASE kratos
    WITH OWNER     postgres
         ENCODING  'UTF8'
         LC_COLLATE 'en_US.utf8'
         LC_CTYPE   'en_US.utf8'
         TEMPLATE   template0;

-- ── Extensions for the nyx app database ──────────────────────────────────────
\connect nyx

-- Create all application schemas up-front so that 02_roles.sh can GRANT on them
-- immediately, before Rust migrations run.
-- NOTE: migrations/Monad/0001_create_schemas.up.sql also creates these — idempotent.
CREATE SCHEMA IF NOT EXISTS nyx;
CREATE SCHEMA IF NOT EXISTS "Uzume";

-- pgcrypto: gen_random_uuid(), crypt(), encode/decode helpers
CREATE EXTENSION IF NOT EXISTS pgcrypto;
-- pg_trgm: fast ILIKE / similarity search on text columns (alias, display_name)
CREATE EXTENSION IF NOT EXISTS pg_trgm;
-- btree_gin: multi-column GIN indexes (JSONB + timestamptz queries)
CREATE EXTENSION IF NOT EXISTS btree_gin;

-- ── Extensions for the Kratos database ───────────────────────────────────────
\connect kratos

CREATE EXTENSION IF NOT EXISTS pgcrypto;
