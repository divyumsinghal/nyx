# Seshat — Documentation

> In Egyptian mythology, Seshat is the goddess of writing, literature, and knowledge. She records the events of life and maintains the library of all that is known.

## Overview

This directory contains the architecture documentation, ADRs, runbooks, and technical specs for Nyx.

## Contents

```
Seshat/
├── ARCHITECTURE.md       # Full system architecture (start here)
├── Uzume.md              # Uzume-specific documentation
├── ADR-*.md              # Architectural Decision Records
├── STEP1-RUNBOOK.md      # Local development setup
├── TESTING.md            # Testing strategy
├── SECURITY-BASELINE.md  # Security requirements
├── CONTRIBUTING.md       # Contributing guidelines
├── FUTURE.md             # Future plans and ideas
├── e2ePlan.md            # End-to-end testing plan
└── consolidation-status.md  # Status tracking
```

## Key Docs

| Document | Purpose |
|----------|---------|
| `ARCHITECTURE.md` | The definitive source of truth for directory structure, crates, tools |
| `ADR-001/002/003` | Core architectural decisions (identity, app isolation, boundaries) |
| `STEP1-RUNBOOK.md` | How to run the full stack locally |
| ` Uzume.md` | Uzume-specific implementation details |

## How to Use

1. **New to Nyx?** Start with `ARCHITECTURE.md`
2. **Want to run locally?** Read `STEP1-RUNBOOK.md`
3. **Making architectural decisions?** Check existing ADRs first
4. **Contributing?** Read `CONTRIBUTING.md`
