Ask me questions and let us plan this: Your task is to build a fully functional web UI for the platform - web for now, mobile later.

Focus on Uzume and nyx - the web version for both is your current deliverable in this session.

This is the phase 4 for the @e2eplan, the phase 0, 1,2,3 are done.

WHen you are done, I should be able to run the service go to the site (on my computer) and then use the app.

## The recommended stack

**Monorepo structure** — pnpm workspaces + Turborepo. This combination uses pnpm for installation performance via the global store cache and Turborepo for task caching, resulting in the best performance possible. Expo has first-class support for monorepos managed with package managers supporting workspaces: Bun, npm, pnpm, and Yarn.

**Routing** — Expo Router v4 (file-based, universal across all 3 platforms, guarded auth groups built in). Expo Router now supports experimental SplitView for tablet layouts, guarded groups for auth at the folder level, and synchronous layouts that eliminate visual flicker during tab transitions.

**Performance** — With Expo SDK 55 and React Native 0.83+, the Legacy Architecture (the Bridge) is officially removed. The New Architecture uses JSI (JavaScript Interface), which allows JavaScript to hold direct references to C++ native objects — no more JSON serialization. This is critical for the Tinder swipe animations specifically.

**Key packages for your two apps:**

For the **Instagram clone** (feed + stories):
- `@shopify/flash-list` — 10x faster than FlatList for infinite scroll feeds
- `expo-av` or `expo-video` — video playback in Reels-style content
- `react-native-reanimated` — story progress bar animations

For the **Tinder clone** (swipe cards) - Not now:
- `react-native-reanimated` v3 + `react-native-gesture-handler` — smooth card deck with spring physics
- `expo-camera` + `expo-image-picker` — profile photos
- `expo-location` — proximity matching
-
**Styling** — NativeWind v4 (Tailwind CSS that compiles to React Native StyleSheet, works identically on web). Expo has signaled first-party support for native CSS and Tailwind-like styling, aiming to bring a web-style authoring experience to React Native developers out of the box.

**State** — Zustand (tiny, no boilerplate) for local state + React Query / TanStack Query for server state.

**CI/CD** — GitHub Actions with `eas build --local` or direct Gradle/Xcode builds. No spend, no account walls.

The short version: **Expo + pnpm + Turborepo + Expo Router + NativeWind + Reanimated + FlashList**. Everything MIT licensed, nothing gated behind a paywall, one codebase shipping web + iOS + Android for both apps from the same monorepo.

## Anti-slop UI: the V0 workflow

V0 by Vercel (v0.dev). V0 is a text-to-UI generator for React components with Tailwind CSS and Next.js export. Metana It's free-tiered and generates genuinely high-quality UI from a text prompt. The workflow: describe a component visually in V0, get polished Tailwind JSX, then manually port the Tailwind classes to NativeWind equivalents (they're ~90% compatible). This gives you web-quality, designer-calibrated starting points that you actually tweak — not AI-generated gray slop.
For reference design: 21st.dev (Magic UI) is a growing library of animated, non-generic React components you can copy-paste as inspiration for what to prompt Cline/V0 to build. Not React Native, but the patterns translate.

## Design system rules
- ALWAYS use NativeWind utility classes, never inline styles or StyleSheet.create
- Color tokens: brand-primary=#YourHex, surface=#..., use ONLY these
- Border radius: rounded-2xl for cards, rounded-full for avatars, rounded-lg for buttons
- Typography: font-sans for body, font-display for headings (custom font configured in tailwind.config)
- Never use gray-100/gray-200 as backgrounds — use surface-* tokens instead
- Shadow: NO box shadows. Use border-b border-border/10 for dividers

- Swipe cards: use react-native-reanimated interpolateColor + withSpring only
- Lists: ALWAYS FlashList, never FlatList
- Images: expo-image (not Image from react-native)
- Icons: use the /packages/ui/icons folder, not any icon library directly
```

## Anti-slop UI strategy

The reason AI-generated UI looks generic is that AI defaults to whatever the most common patterns are in its training data — which means flat gray cards, Inter font at 16px, blue primary buttons, and border-radius of 8px on everything.

The fix is to define your design system *before* you start vibing, and bake it into every context file the AI sees. Here's the specific stack:

NativeWind compiles Tailwind classes into native stylesheets during the build process — almost no runtime overhead compared to older CSS-in-JS libraries. It lets you share styles directly between your React web app and React Native mobile app using the same utility classes. Your entire design vocabulary lives in `tailwind.config.js` as custom tokens.

gluestack-ui v3 launched in 2025 with a modular, unbundled component structure — unstyled, accessible elements that you style with Tailwind CSS utility classes. This provides full flexibility and control to tailor each UI element. The key point: you copy-paste the components into your repo. They're not a dependency you're locked into. You own the code, you mutate it freely, and the AI will learn your patterns from it.

For the "not boring" part: pick a real aesthetic direction *before* opening the editor. Two directions that work well for these apps and are underrepresented in AI-generated UIs:

For the Tinder clone — dark editorial. Deep charcoal surfaces, large serif or display typeface for names, thin 1px amber/coral accent strokes, photography-first cards that bleed to edges, minimal chrome.

For the Instagram clone — high-contrast minimal. Pure white or true black depending on mode, tight 4px spacing grid, a strong custom sans that isn't Inter, content-first with zero decorative UI elements.

Bring in a free typeface from Google Fonts that isn't Inter or Roboto (Bricolage Grotesque, Syne, DM Sans, Instrument Serif), configure it in `tailwind.config`, and put it in your AGENT.md. The AI will use it in every component it generates.

## Dev environment (local first)

Android SDK + emulator runs cleanly via Android Studio (just the SDK, not the IDE). Install Node via `nvm`, pnpm globally, and set up Metro to hot-reload to your actual Android device over USB or WiFi using `adb`.

Expo Go testing — scan the QR code, see the app on the real device immediately. No build needed, no account needed.

## Phase plan (expanded)

install `nvm`, Node 22, pnpm, Android Studio SDK only (not the full IDE).
Scaffold the monorepo: `pnpm create turbo`, add two Expo apps inside `apps/`, add shared `packages/ui`, `packages/api`, `packages/config`.
Verify Metro starts and Expo Go connects on a physical device.

Choose your aesthetic direction.
Pick your typeface.
Define 10–15 color tokens in `tailwind.config`. -> nice and modern (not ai-slop)
Configure NativeWind v4 in both apps.
Install gluestack primitives and rename/restyle the base components to match your tokens.
Write your AGENT.md files.
This is the foundation everything else sits on — time here pays off 10x.

**Instagram clone.** Feed first with FlashList — this is performance-critical. Stories bar above, post detail on tap, profile grid using FlashList in column mode, camera/upload sheet using `expo-image-picker`. The stories animation (progress bar + story-to-story transition) is the showpiece here — Reanimated handles it.

`react-native-reanimated` + `react-native-gesture-handler` for the physics. Start with a static stack of 3 cards, wire the swipe gesture, add like/nope overlays with color interpolation, then add spring-back. Chat list, screen, and profile are straightforward after that.

**Phase 5 — Backend wiring + deploy (week 3–4).** Plug your existing API into the shared layer using TanStack Query. Add auth (your backend's auth flow via `expo-secure-store` for tokens). At this point you have two fully functional apps running on web, Android, and iOS from one monorepo.

---

## Quick reference: every tool in the stack

| Layer | Tool | Why |
|---|---|---|
| Context files | `AGENT.md` + `CLAUDE.md` | Anti-slop design rules for the AI |
| Styling | NativeWind v4 | Tailwind in React Native, zero runtime cost |
| Components | gluestack-ui v3 | Copy-paste, fully ownable, NativeWind-native |
| Gestures | Reanimated 3 + Gesture Handler | Tinder swipes, story animations |
| Feed perf | FlashList | 10x FlatList for infinite scroll |
| Images | expo-image | Better caching + progressive loading |
| Monorepo | pnpm + Turborepo | Task caching, workspace linking |
| Routing | Expo Router v4 | File-based, universal web+mobile |
| Android dist | `eas build --local` | APK sideload, zero EAS cost |
| iOS dev | Expo Go | Free, no Apple account needed for dev |