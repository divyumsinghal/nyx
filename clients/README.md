# Clients

```
clients/
├── shared/        # @nyx/ui: Svelte component library (Auth, Chat, Media, Notification, Common)
├── Uzume-web/      # Uzume SvelteKit app
└── nyx-web/       # Nyx account portal (profile, linked apps, settings)
```

SvelteKit, `matrix-js-sdk` for E2EE DMs, Cloudflare Pages (free).

## clients/Uzume-web, Anteros-web, Themis-web, nyx-web

Each is a standalone SvelteKit app. They import `@nyx/ui` for shared components and have their own routes, layouts, and app-specific components.

```
Uzume-web/
├── package.json               # name: "@nyx/Uzume-web", depends on @nyx/ui
├── svelte.config.js
├── vite.config.ts
├── src/
│   ├── routes/                # SvelteKit file-based routing
│   │   ├── +layout.svelte     # App shell: navbar, sidebar
│   │   ├── +page.svelte       # Home feed
│   │   ├── explore/
│   │   ├── reels/
│   │   ├── messages/
│   │   ├── notifications/
│   │   ├── profile/[alias]/
│   │   ├── post/[id]/
│   │   └── settings/
│   ├── lib/                   # App-specific Svelte components
│   │   ├── Feed/
│   │   ├── Stories/
│   │   ├── Reels/
│   │   ├── Profile/
│   │   └── Post/
│   └── app.css                # Uzume-specific theme/branding
├── static/                    # Static assets (favicon, logo, manifest.json)
└── tests/                     # Playwright e2e tests
```

Each web app is independently deployable. In production, each builds to static files + SSR functions and deploys to Cloudflare Pages (free tier: unlimited sites, unlimited bandwidth).