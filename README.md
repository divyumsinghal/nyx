# Nyx Root

> Own your digital life. An open-source, privacy-first ecosystem of apps for the People.

## Directory Structure

```
nyx/
├── Monad/        # Platform - shared Rust libraries and services
├── apps/          # Application microservices (Uzume, Anteros, Themis)
├── Maya/         # Frontend - SvelteKit UI clients
├── Seshat/       # Documentation - architecture, ADRs, runbooks
├── Prithvi/      # Infrastructure - Docker Compose, configs
├── migrations/   # Database migrations (SQL)
├── contracts/    # Lockfiles for compatibility checks
├── tools/        # Development tools (seed data)
└── tests/        # Integration/e2e tests
```

## Key Concepts

- **Nyx** = Monad (platform libraries) + apps that compose them
- **Dual workspace**: Cargo (Rust) + pnpm (JavaScript)
- **Multi-app model**: One identity works across apps via app-scoped aliases (privacy isolation)
- **8 deployable processes**: 1 gateway + 5 Uzume services + 2 workers

## Getting Started

1. Read `Seshat/ARCHITECTURE.md` for the full picture
2. Read `Seshat/STEP1-RUNBOOK.md` to run the stack
3. Check `justfile` for common dev commands
