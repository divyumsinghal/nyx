# Uzume-web — SvelteKit Frontend

> The Uzume social media web client.

## Status

Placeholder directory. Will contain the SvelteKit app when implemented.

## Planned Structure

```
Uzume-web/
├── src/
│   ├── routes/           # SvelteKit file-based routing
│   │   ├── +layout.svelte
│   │   ├── +page.svelte
│   │   ├── explore/
│   │   ├── reels/
│   │   ├── messages/
│   │   ├── notifications/
│   │   ├── profile/[alias]/
│   │   ├── post/[id]/
│   │   └── settings/
│   └── lib/              # App-specific components
├── static/
├── tests/
└── package.json
```

## Dependencies

- `@nyx/ui` (shared component library)
- `matrix-js-sdk` (for DMs)
- SvelteKit

See `Maya/README.md` for more details.
