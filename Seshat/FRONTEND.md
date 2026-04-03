# Frontend — Setup, Architecture, and Dev Guide

This document covers everything needed to run, develop, and extend the Nyx frontend.

---

## Apps

| App            | Package          | Port | URL                     | Purpose                                |
| -------------- | ---------------- | ---- | ----------------------- | -------------------------------------- |
| **Uzume**      | `@nyx/uzume-web` | 8081 | <http://localhost:8081> | Social media app (Instagram-like)      |
| **Nyx Portal** | `@nyx/nyx-web`   | 8082 | <http://localhost:8082> | Account management, cross-app settings |

Both are Expo 52 / React Native Web apps. One codebase, web-first.

---

## Quickstart

```bash
# Start Uzume (social app)
just web

# Start Nyx account portal
just web-nyx

# Start both at once
just web-all
```

The backend does not need to be running for the UI to render. Auth calls will fail gracefully and keep the user on the login/register screens.

---

## Tech Stack

| Layer               | Choice                                        | Why                                           |
| ------------------- | --------------------------------------------- | --------------------------------------------- |
| Framework           | Expo 52 + React Native Web                    | Single codebase for web (and future native)   |
| Routing             | expo-router 4 (file-based)                    | Zero-config routing, typed routes             |
| Styling             | NativeWind 4 + Tailwind CSS 3                 | Utility classes that compile to RN StyleSheet |
| Animation           | react-native-reanimated 3                     | Gestures and transitions                      |
| Navigation          | expo-router/drawer + @react-navigation/drawer | Sidebar nav on desktop, bottom tab on mobile  |
| Package manager     | pnpm 10 (workspace)                           | Shared packages, fast installs                |
| Build orchestration | Turbo 2                                       | Parallel builds, incremental caching          |
| Bundler             | Metro (via Expo)                              | React Native–native bundler                   |

---

## Repository Layout

```
nyx/
├── Maya/
│   ├── uzume-web/          # @nyx/uzume-web — Uzume social app
│   │   ├── app/            # expo-router routes
│   │   │   ├── _layout.tsx        # Root layout (AuthProvider + Stack)
│   │   │   ├── index.tsx          # Root → redirects to /(auth)/login
│   │   │   ├── (auth)/            # Unauthenticated routes
│   │   │   │   ├── login.tsx
│   │   │   │   └── register.tsx
│   │   │   ├── (main)/            # Authenticated routes (requires login)
│   │   │   │   ├── _layout.tsx    # Sidebar (desktop) / bottom-nav (mobile)
│   │   │   │   ├── index.tsx      # Home feed
│   │   │   │   ├── explore.tsx
│   │   │   │   ├── reels.tsx
│   │   │   │   ├── notifications.tsx
│   │   │   │   ├── messages/
│   │   │   │   ├── post/
│   │   │   │   ├── profile/
│   │   │   │   └── reel/
│   │   │   └── (admin)/           # Admin routes
│   │   ├── src/
│   │   │   ├── context/AuthContext.tsx   # Auth state + API calls
│   │   │   └── components/              # Screen-specific components
│   │   ├── assets/                # icon.png, splash-icon.png, favicon.png
│   │   ├── global.css             # Tailwind entry + custom utilities
│   │   ├── babel.config.js
│   │   ├── metro.config.js
│   │   ├── tailwind.config.js
│   │   └── app.json
│   │
│   ├── nyx-web/            # @nyx/nyx-web — Nyx account portal
│   │   ├── app/
│   │   │   ├── _layout.tsx        # Root layout with auth redirect
│   │   │   ├── index.tsx          # Root → redirects to /(auth)/login
│   │   │   ├── (auth)/            # Login / register
│   │   │   └── (main)/            # Account portal (requires login)
│   │   └── ...                    # Same structure as uzume-web
│   │
│   └── shared/             # (unused placeholder — shared code is in packages/)
│
└── packages/
    ├── api/                # @nyx/api — typed HTTP client for all Nyx services
    │   └── src/
    │       ├── client.ts          # Base fetch client (points to Heimdall :3000)
    │       ├── nyx/               # auth.ts, account.ts, messaging.ts
    │       └── uzume/             # feed.ts, profiles.ts, stories.ts, reels.ts, discover.ts
    ├── ui/                 # @nyx/ui — shared React Native component library
    │   └── src/
    │       ├── index.ts           # Re-exports all public components
    │       ├── tokens.ts          # Design tokens (colors, spacing, typography)
    │       ├── icons/             # SVG icon components
    │       └── components/        # Avatar, Button, TextInput, Card, Skeleton
    └── config/             # @nyx/config — shared Tailwind + TypeScript config
        ├── tailwind.config.js
        └── tsconfig.base.json
```

---

## Shared Packages

All shared code lives in `packages/`. They are TypeScript source — Metro compiles them directly, no build step required.

| Package       | What it provides                                                       |
| ------------- | ---------------------------------------------------------------------- |
| `@nyx/api`    | All typed HTTP calls. Import `authApi`, `feedApi`, `profilesApi`, etc. |
| `@nyx/ui`     | `Avatar`, `Button`, `TextInput`, `Card`, `Skeleton`, icon set          |
| `@nyx/config` | Base `tailwind.config.js` and `tsconfig.base.json`                     |

**HMR for shared packages**: Both apps set `watchFolders = [workspaceRoot]` in `metro.config.js`. Changing a file in `packages/ui/src/` will hot-reload both running dev servers.

---

## Auth Flow

```
Browser opens /
  └─► app/index.tsx → <Redirect href="/(auth)/login" />
        └─► (auth)/_layout.tsx → renders Stack
              └─► (auth)/login.tsx
                    [user logs in]
                      └─► AuthContext.login() → POST /api/nyx/auth/login
                            └─► stores session token in AsyncStorage
                                  └─► router.replace("/(main)")
                                        └─► (main)/_layout.tsx
                                              └─► renders home feed / account portal
```

If the backend is not running, `authApi.login()` will throw. The UI catches this, shows an error message, and stays on the login screen. All pages render correctly without a backend.

---

## API Client

`@nyx/api` sends all requests through the Heimdall gateway at port 3000.

```typescript
import { authApi, feedApi, profilesApi, setAuthToken } from "@nyx/api";

// Set after login:
setAuthToken(sessionToken);

// Then call any endpoint:
const session = await authApi.login({ identifier, password });
const feed = await feedApi.getHomeFeed({ limit: 20 });
```

The gateway URL is controlled by:

- `EXPO_PUBLIC_GATEWAY_URL` env var (browser-side)
- `GATEWAY_URL` env var (server-side / SSR)
- Falls back to `http://localhost:3000`

---

## Styling

Both apps use NativeWind 4 with a shared Tailwind config from `@nyx/config`.

### Design Tokens (custom colors in tailwind config)

| Token       | Hex       | Usage                     |
| ----------- | --------- | ------------------------- |
| `space-900` | `#060412` | Page background           |
| `space-800` | `#0D0A1E` | Card/panel backgrounds    |
| `space-700` | `#13103A` | Borders, dividers         |
| `dawn-400`  | `#FF6B9D` | Primary accent (pink)     |
| `dawn-500`  | `#FF8C61` | Secondary accent (orange) |
| `dawn-600`  | `#FFD93D` | Tertiary accent (yellow)  |
| `star-200`  | `#F0EBF8` | Primary text              |
| `star-300`  | `#C4B5D4` | Secondary text            |

### Custom CSS utilities (in global.css)

| Class                 | Effect                             |
| --------------------- | ---------------------------------- |
| `.text-dawn-gradient` | Pink→orange→yellow text gradient   |
| `.bg-dawn-gradient`   | Same gradient as background        |
| `.glass-card`         | Translucent dark card with blur    |
| `.star-field`         | Subtle star/dot background pattern |

### NativeWind setup

The `jsxImportSource: "nativewind"` in `babel-preset-expo` is the only babel config needed. Do **not** add `"nativewind/babel"` as a separate preset — it pulls in `react-native-css-interop/babel` which tries to load the deprecated `react-native-worklets` package.

---

## Known Gotchas

### `nativewind/babel` must NOT be in presets

`nativewind/babel` = `react-native-css-interop/babel` which hardcodes `"react-native-worklets/plugin"` (the old unmaintained package). This breaks the Metro bundler with:

```
Cannot find module 'react-native-worklets/plugin'
```

**Fix:** Only use `["babel-preset-expo", { jsxImportSource: "nativewind" }]`.

### pnpm requires shamefully-hoist for Metro

Metro resolves modules differently from Node — it walks up the directory tree and doesn't understand pnpm's isolated virtual store. Without `shamefully-hoist=true` in `.npmrc`, Metro workers fail to resolve babel plugins.

**Fix:** `.npmrc` at workspace root must contain `shamefully-hoist=true`.

### Global expo CLI vs local

`pnpm exec expo` can pick up the globally installed Expo CLI (a different version). Always run via `pnpm run dev` (which uses the local `expo` binary from package.json scripts) or `pnpm --dir Maya/uzume-web run dev`.

### Metro transform cache

Metro caches compiled JS in `%TEMP%/metro-*`. If babel config changes don't take effect, delete these directories. The `--reset-cache` flag clears Metro's internal cache but not all OS-level caches.

---

## Turbo Pipeline

```
turbo.json tasks:
  build     → depends on ^build (shared packages build first)
  dev       → persistent=true, no cache (interactive dev servers)
  test      → parallel, outputs coverage/
  lint      → parallel, no outputs
  typecheck → parallel, no outputs
  clean     → no cache
```

Run all frontends via Turbo:

```bash
pnpm run dev          # all packages with a dev script
pnpm run dev:uzume    # only @nyx/uzume-web
pnpm run dev:nyx      # only @nyx/nyx-web
```

---

## Adding a New App Frontend

1. Copy `Maya/uzume-web/` to `Maya/{app}-web/`
2. Update `package.json`: set `name`, change port in `dev` script
3. Clear `app/` routes — keep only `_layout.tsx` and `index.tsx`
4. Add to `pnpm-workspace.yaml` (already covered by `Maya/*`)
5. Add a `just web-{app}` recipe in the justfile
6. Add to `.claude/launch.json` with the new port

---

## Ports Reference

| Service           | Port |
| ----------------- | ---- |
| Uzume web (dev)   | 8081 |
| Nyx Portal (dev)  | 8082 |
| Heimdall gateway  | 3000 |
| Kratos (identity) | 4433 |
| Grafana           | 3030 |
| MinIO console     | 9001 |
| Mailhog           | 8025 |

## Routes

The frontend of the Nyx ecosystem consists of two primary web applications built using **Expo Router (React Native)**: **Uzume Web** (the social platform) and **Nyx Web** (the account portal).

### 1. Uzume Web (Social Platform)

This is the main consumer-facing application with a "starry night/dawn" aesthetic. It provides a full social media experience.

#### **Main Routes & Capabilities**

| Route              | Name              | Key Capabilities                                                                                |
| :----------------- | :---------------- | :---------------------------------------------------------------------------------------------- |
| `/`                | **Home Feed**     | Infinite scroll timeline, horizontal **Stories** bar, post engagement (likes, saves, comments). |
| `/explore`         | **Explore**       | Content discovery feed with suggested posts and creators.                                       |
| `/reels`           | **Reels**         | Vertical short-form video feed (similar to TikTok/Instagram Reels).                             |
| `/notifications`   | **Notifications** | Activity log for likes, comments, and new followers.                                            |
| `/profile/[alias]` | **User Profile**  | View avatars, bios, shared content, and follower statistics.                                    |
| `/post/[id]`       | **Post View**     | Deep dive into a single post with its associated comment thread.                                |
| `/reel/[id]`       | **Reel View**     | Detailed view for specific short-form videos.                                                   |
| `/new-post`        | **Create Post**   | Workspace for uploading media and writing captions (currently a basic implementation).          |

#### **Auth & Admin Routes**

* **`/login` & `/register`**: Full authentication flow integrated with the Nyx backend.
- **`/admin`**: A moderation console showing "System Health" and "Open Reports" counts.
- **`/admin/reports`**: Interface for moderators to review flagged content.

---

### 2. Nyx Web (Account Portal)

The central hub for managing your global identity across the Nyx platform.

#### **Routes & Capabilities**

* **`/` (Dashboard)**: Displays user profile info (Email, Display Name) and provides quick access cards for "Personal Information" and "Security."
- **`/login` & `/register`**: Unified authentication for the account portal.
- **Cross-App Navigation**: Includes a primary action button to jump directly to the Uzume social platform.

---

### 3. Visual & Technical Features

* **Design Language**: Deep space tones (`#060412`), "dawn" gradients (pink/orange/yellow), and glassmorphism effects.
- **Responsive Layouts**:
  - **Desktop**: Permanent sidebar navigation with Uzume branding.
  - **Mobile**: Bottom navigation bar for core social features.
- **Shared Infrastructure**: Both apps consume `@nyx/api` for backend communication and `@nyx/ui` for high-fidelity components like avatars, icons, and skeletons.

### ⚠️ Current Status / Gaps

Based on the current codebase, some routes are referenced in the UI but are not yet fully implemented or are missing files:
- **`/messages`**: The folder exists but is currently empty.
- **`/settings`**: Linked in the sidebar, but the route file is missing from both apps.
- **`/story/[alias]`**: The feed has a "view story" link, but the viewing page is not yet implemented.
- **`/edit-profile`**: Current implementation is a placeholder.
