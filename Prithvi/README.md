# Prithvi — Infrastructure

> In Hindu cosmology, Prithvi is the earth goddess — the foundation upon which everything rests. She is the ground (infrastructure) that supports life (the platform).

## Overview

Docker Compose configs and environment configurations for running the entire Nyx stack locally or in production.

```
Prithvi/
├── compose/          # Docker Compose files
├── config/           # Service configs for each infra component
│   ├── nats/
│   ├── postgres/
│   ├── minio/
│   ├── meilisearch/
│   ├── kratos/
│   ├── dragonfly/
│   ├── gorush/
│   ├── grafana/
│   ├── prometheus/
│   ├── loki/
│   └── continwuuity/
└── docker/           # Dockerfiles for building images
```

## Components

| Component | Purpose | Default Port |
|-----------|---------|---------------|
| PostgreSQL | Primary database | 5432 |
| DragonflyDB | Cache + sessions + rate limiting | 6379 |
| NATS | Event bus / message queue | 4222 |
| MinIO | S3-compatible object storage | 9000/9001 |
| Meilisearch | Full-text search engine | 7700 |
| Ory Kratos | Identity + authentication | 4433 |
| Gorush | Push notification dispatch | 8088 |
| Continuwuity | Matrix homeserver | 8008 |
| Prometheus | Metrics | 9090 |
| Grafana | Metrics visualization | 3000 |
| Loki | Log aggregation | 3100 |

## Usage

```bash
# Start all infrastructure
cd Prithvi/compose
docker compose up -d

# Check status
docker compose ps
```

See `Seshat/STEP1-RUNBOOK.md` for the full local setup.
