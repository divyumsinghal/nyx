# Nyx Step-1 Single-Region Runbook

## Scope
This runbook covers Step-1 operations for the current repository state:
- Core Rust workspace validation (`Nun`, `Heka`, `nyx-events`, `Uzume-profiles`, `Uzume-feed`)
- Local single-region infrastructure bring-up via Docker Compose
- Health checks, incident triage, and backup/restore drill procedures

This runbook does not implement multi-region routing, failover automation, or provider-specific event backbone operations.

## Preconditions
1. Required tools:
   - `cargo`
   - `just`
   - `docker` with `docker compose`
2. Optional file for local overrides:
   - `.env` or `.env.local`
3. Repository root:
   - Run commands from repository root (`/home/sin/nyx`)

## Step-1 Bring-Up (Fresh Environment)
1. Validate Rust workspace baseline.

```bash
cargo build --workspace
cargo test --workspace
```

2. Validate Compose manifests before startup.

```bash
just compose-validate
```

3. Start infrastructure layer.

```bash
just dev-infra
```

4. Start full stack (infra + platform + Uzume services).

```bash
just dev-up
```

5. Check container status.

```bash
docker compose \
  -f Prithvi/compose/infra.yml \
  -f Prithvi/compose/platform.yml \
  -f Prithvi/compose/uzume.yml \
  -f Prithvi/compose/dev.yml \
  ps
```

6. Run gateway and service health probes.

```bash
curl -fsS http://localhost:3000/healthz
curl -fsS http://localhost:3001/healthz
curl -fsS http://localhost:3002/healthz
curl -fsS http://localhost:4433/health/ready
```

Expected result: all services report healthy and return non-error responses.

## Incident Triage
Use this decision path when bring-up or runtime checks fail.

1. A container is not healthy.

```bash
docker compose \
  -f Prithvi/compose/infra.yml \
  -f Prithvi/compose/platform.yml \
  -f Prithvi/compose/uzume.yml \
  -f Prithvi/compose/dev.yml \
  logs --tail=200 <service-name>
```

2. Database connectivity symptoms (`timeout`, `connection refused`, `pool exhausted`).

```bash
docker compose -f Prithvi/compose/infra.yml ps postgres
docker exec -it nyx-postgres pg_isready -U postgres -d nyx
```

3. Identity/auth symptoms (`401`, Kratos readiness errors).

```bash
docker compose -f Prithvi/compose/infra.yml logs --tail=200 kratos
curl -fsS http://localhost:4433/health/ready
```

4. Cross-app privacy regressions.

```bash
cargo test -p Heka --test integration_privacy_matrix
```

5. Chronological feed regressions.

```bash
cargo test -p Uzume-feed --test step1_feed_chronological
cargo test -p Uzume-feed --test step1_feed_mode_handling
```

## Backup/Restore Drill (PostgreSQL)
Run this drill in local single-region environment.

1. Create backup directory.

```bash
mkdir -p .sisyphus/tmp/backups
```

2. Produce `pg_dump` backup from running postgres container.

```bash
docker exec nyx-postgres \
  pg_dump -U postgres -d nyx -Fc -f /tmp/nyx-step1.dump
docker cp nyx-postgres:/tmp/nyx-step1.dump \
  .sisyphus/tmp/backups/nyx-step1.dump
```

3. Verify backup file integrity.

```bash
ls -lh .sisyphus/tmp/backups/nyx-step1.dump
sha256sum .sisyphus/tmp/backups/nyx-step1.dump
```

4. Restore into drill database.

```bash
docker exec nyx-postgres psql -U postgres -c "DROP DATABASE IF EXISTS nyx_restore_drill;"
docker exec nyx-postgres psql -U postgres -c "CREATE DATABASE nyx_restore_drill;"
docker cp .sisyphus/tmp/backups/nyx-step1.dump nyx-postgres:/tmp/nyx-step1.dump
docker exec nyx-postgres \
  pg_restore -U postgres -d nyx_restore_drill --clean --if-exists /tmp/nyx-step1.dump
```

5. Validate restored schema.

```bash
docker exec nyx-postgres psql -U postgres -d nyx_restore_drill -c "SELECT schema_name FROM information_schema.schemata WHERE schema_name IN ('nyx', 'Uzume');"
```

6. Cleanup drill database.

```bash
docker exec nyx-postgres psql -U postgres -c "DROP DATABASE IF EXISTS nyx_restore_drill;"
```

## Shutdown
1. Graceful stack shutdown.

```bash
just dev-down
```

2. Full reset for clean re-test (destructive).

```bash
just dev-nuke
```

## Step-2+ Assumptions (Documented, Not Implemented)
1. Single-region assumptions today:
   - One region, one primary database, one NATS cluster
   - No active-active traffic shaping
2. Multi-region evolution later should introduce:
   - Region-aware gateway routing
   - Cross-region data replication and recovery RPO/RTO policy
   - Region-local event transport adapters behind the same `nyx-events` abstraction
