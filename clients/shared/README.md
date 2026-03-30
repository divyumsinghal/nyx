# Shared

The shared UI component library. Published as `@nyx/ui` within the pnpm workspace. Every app frontend imports from this.

```
shared/
├── package.json               # name: "@nyx/ui"
├── src/
│   ├── components/
│   │   ├── Auth/              # LoginForm, RegisterForm, TwoFactorInput
│   │   ├── Chat/              # ChatWindow, MessageBubble, TypingIndicator
│   │   ├── Media/             # ImageUploader, VideoPlayer, Carousel, Gallery
│   │   ├── Notification/      # NotificationBell, NotificationList
│   │   └── Common/            # Button, Input, Modal, Avatar, Skeleton, Toast
│   ├── stores/                # Shared Svelte stores
│   │   ├── auth.ts            # Current user session, JWT token management
│   │   ├── notifications.ts   # Real-time notification stream (WebSocket)
│   │   └── chat.ts            # Matrix client state, room list, message streams
│   ├── api/                   # Typed API client (generated from OpenAPI or hand-written)
│   │   ├── client.ts          # Base HTTP client: fetch wrapper with JWT injection, error handling
│   │   ├── nyx.ts             # Nyx identity API calls
│   │   ├── Uzume.ts            # Uzume API calls
│   │   ├── Anteros.ts          # Anteros API calls
│   │   └── Themis.ts          # Themis API calls
│   └── matrix/                # Matrix client integration
│       ├── client.ts          # matrix-js-sdk initialization, login with Nyx credentials
│       └── rooms.ts           # App-scoped room filtering, message rendering
└── tsconfig.json
```