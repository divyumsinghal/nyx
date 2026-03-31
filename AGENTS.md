# PROJECT KNOWLEDGE BASE

**Generated:**
**Commit:**
**Branch:**

## OVERVIEW


## STRUCTURE

```
```

## WHERE TO LOOK

| Task | Location | Notes |
| ---- | -------- | ----- |
|      |          |       |

## TDD (Test-Driven Development)

**MANDATORY.** RED-GREEN-REFACTOR:
1. **RED**: Write test → `just test` → FAIL
2. **GREEN**: Implement minimum → PASS
3. **REFACTOR**: Clean up → stay GREEN

**Rules:**
- NEVER write implementation before test
- NEVER delete failing tests - fix the code
- Test file: `*.test.ts` alongside source
- BDD comments: `#given`, `#when`, `#then`

## CONVENTIONS

- **Package manager**: Bun only (`bun run`, `bun build`, `bunx`)
- **Types**: bun-types (NEVER @types/node)
- **Build**: `bun build` (ESM) + `tsc --emitDeclarationOnly`
- **Exports**: Barrel pattern via index.ts
- **Naming**: kebab-case dirs, `createXXXHook`/`createXXXTool` factories
- **Testing**: BDD comments, 95 test files
- **Temperature**: 0.1 for code agents, max 0.3

## ANTI-PATTERNS

| Category        | Forbidden                                    |
| --------------- | -------------------------------------------- |
| Package Manager | npm, yarn - Bun exclusively                  |
| Types           | @types/node - use bun-types                  |
| File Ops        | mkdir/touch/rm/cp/mv in code - use bash tool |
| Publishing      | Direct `bun publish` - GitHub Actions only   |
| Versioning      | Local version bump - CI manages              |
| Type Safety     | `as any`, `@ts-ignore`, `@ts-expect-error`   |
| Error Handling  | Empty catch blocks                           |
| Testing         | Deleting failing tests                       |
| Agent Calls     | Sequential - use `delegate_task` parallel    |
| Hook Logic      | Heavy PreToolUse - slows every call          |
| Commits         | Giant (3+ files), separate test from impl    |
| Temperature     | >0.3 for code agents                         |
| Trust           | Agent self-reports - ALWAYS verify           |

## COMMANDS

```bash
```

## DEPLOYMENT

**GitHub Actions workflow_dispatch ONLY**
1. Commit & push changes
2. Trigger: `gh workflow run publish -f bump=patch`
3. Never `publish` directly, never bump version locally

## COMPLEXITY HOTSPOTS

| File | Lines | Description |
| ---- | ----- | ----------- |
|      |       |             |
