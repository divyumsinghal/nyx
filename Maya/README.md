# Maya (clients)

> In Hindu philosophy, Maya is the divine power of illusion: specifically the force that causes the phenomenal world to appear real when it is in fact a construct laid over a deeper reality.

Expo (React Native + web) apps in this repo share **`@nyx/api`**, **`@nyx/ui`**, and **`@nyx/config`** from `packages/` at the repository root (pnpm workspace).

## Layout

```
Maya/
├── nyx-web/      # @nyx/nyx-web — Nyx account / auth surfaces (Expo Router)
├── uzume-web/    # @nyx/uzume-web — Uzume social client (Expo Router)
└── shared/       # Notes / non-package assets (optional)
```

Each app has its own `package.json`, Metro/Babel/Tailwind setup, and `app/` routes. Shared UI and API clients live in **`packages/ui`** and **`packages/api`** so both apps stay in sync.

## Commands (from repo root)

- `pnpm dev:nyx` — start Nyx web client
- `pnpm dev:uzume` — start Uzume web client
- `pnpm lint` / `pnpm typecheck` — run via Turbo across workspaces that define those scripts
