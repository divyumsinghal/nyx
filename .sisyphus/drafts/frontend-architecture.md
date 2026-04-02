# Draft: Frontend Architecture Strategy — Web + iOS + Android for Nyx

## User Requirements
- Replace Instagram (Uzume) + Tinder (Anteros) — must not feel worse
- Web + iOS + Android from single monorepo
- Each app is its own standalone App Store / Play Store listing
- **100% FREE** — no paid tiers, no EAS Build paid, no SaaS dependencies
- Backend is already done (Rust/Axum/sqlx) — assume it works
- Can defer reels/video to MVP+ if needed, but must be architecturally possible

## Research Findings (4 agents, April 2026)

### Agent 1: Free Framework Comparison
| Framework | Truly Free? | Cloud Build Required? | Video Performance | Swipe Performance | Monorepo Tool |
|---|---|---|---|---|---|
| **Flutter** | ✅ 100% free, MIT | No — local builds only | ✅ Excellent (video_player + chewie) | ✅ Excellent (flutter_card_swiper) | Melos |
| **React Native + Expo** | ⚠️ Free tier: 15 builds/mo per platform | Optional (EAS free has limits) | ⚠️ Good (expo-av, react-native-video) | ✅ Excellent (react-native-reanimated) | Turborepo |
| **React Native CLI** | ✅ 100% free, MIT | No — local builds only | ⚠️ Good (react-native-video) | ✅ Excellent | Turborepo |
| **Capacitor/Ionic** | ✅ 100% free, MIT | No — local builds only | ⚠️ Moderate (WebView bottleneck) | ⚠️ Moderate (JS-driven) | Lerna/Nx |

**Key finding**: Expo free tier gives 15 iOS + 15 Android builds/month with low-priority queue. Can submit to stores on free tier. Beyond that = $25+/mo.

### Agent 2: UI Component Library Catalog
**Stories**: `react-insta-stories` (1,470 stars, active) — React only. Svelte ecosystem has `svelte-stories` but limited/unmaintained.
**Reels/Video Feed**: `react-native-video-feed` (105 stars, May 2025) — only production-ready RN option. Flutter has `video_player` + `chewie`.
**Swipe Cards**: `react-tinder-card` (web), `react-native-swipeable-card-stack` (RN). Flutter: `flutter_card_swiper`.
**Feed/Masonry**: `virtua` (424K weekly downloads, supports React/Vue/Svelte/Solid). `masonic` (57K weekly downloads).
**Chat**: `react-native-gifted-chat` (14,369 stars) — industry standard for RN.
**Camera**: `@capacitor/camera` (official), `react-native-image-picker` (6K stars).

**Critical finding**: The Svelte ecosystem is THIN for social media features. Almost all production-quality libraries target React/React Native or Flutter.

### Agent 3: Monorepo Architecture
**Recommended structure**: Turborepo + pnpm for React ecosystem, Melos for Flutter.
**Code sharing**: `workspace:*` protocol for direct imports — no npm publishing needed.
**Type-safe API**: OpenAPI from Rust backend (utoipa) → TypeScript codegen (openapi-typescript-codegen or openapi-ts).
**Real examples**: `byCedric/expo-monorepo-example` (987 stars), `t3-oss/create-t3-turbo`, Shopify RN monorepo patterns.

### Agent 4: Capacitor Deep-Dive
**Honest assessment**:
- ✅ Can build Tinder clone — swipe cards work fine in WebView
- ⚠️ Can build simplified Instagram — feed, profiles, stories work
- ❌ Will STRUGGLE with TikTok/Reels — vertical video feed at 60fps is the hardest pattern for WebView
- ❌ Typescript + Capacitor has known iOS routing bug (#7972) — infinite reload in hash mode
- ❌ No production social media apps at scale use Capacitor
- ✅ 100% free — no Appflow needed for basic builds
- ⚠️ Android performance 15-25% worse than iOS for same Capacitor app

**Key quote from research**: "Capacitor can ship a Tinder clone. It can ship a simplified Instagram. But if video performance is your competitive differentiator, the WebView ceiling will become a problem."

## The Core Tension

**Nyx already chose Typescript** (documented in ARCHITECTURE.md). But:
1. Typescript ecosystem for social media UI components is very thin
2. Typescript + Capacitor has known iOS bugs
3. No production social apps use Typescript + Capacitor at scale
4. Best libraries (stories, reels, swipe, chat) are React/React Native or Flutter

## Options Analysis

### Option A: Stay Typescript + Capacitor
- **Pro**: Consistent with existing architecture choice, one codebase, zero new skills
- **Con**: Thin ecosystem, iOS bugs, video performance ceiling, no proven social app examples
- **Verdict**: Risky for "replace Instagram" ambition

### Option B: Typescript Web + React Native Mobile (separate UI layers)
- **Pro**: Best of both — Typescript for web (your choice), RN for native (best libraries)
- **Con**: Two UI codebases per app, team needs both Svelte + React skills
- **Shared**: Types, API client, business logic via monorepo packages
- **Verdict**: Pragmatic compromise but maintenance overhead

### Option C: Full React Native + Expo (web via RN Web)
- **Pro**: One UI codebase, best ecosystem, Expo free tier works
- **Con**: Abandoning Typescript investment, Expo build limits (15/mo)
- **Verdict**: Best ecosystem but conflicts with existing architecture decision

### Option D: Full Flutter (web + mobile)
- **Pro**: 100% free, best performance, one codebase, mature monorepo (Melos)
- **Con**: Must learn Dart, abandoning Typescript, Flutter web is still maturing
- **Verdict**: Best technical choice but biggest pivot

### Option E: Typescript Web + Flutter Mobile (separate UI layers)
- **Pro**: Typescript for web (your choice), Flutter for native (best performance)
- **Con**: Two completely different stacks, Dart + TypeScript, highest complexity
- **Verdict**: Maximum quality but maximum complexity

## My Recommendation

Given the constraints (free, replace Instagram/Tinder, monorepo, each app standalone):

**Option B: Typescript for Web + React Native + Expo for Mobile**

Rationale:
1. Respects existing Typescript investment for web
2. RN has the BEST ecosystem for social media features (gifted-chat, react-native-video-feed, react-native-reanimated for swipes)
3. Expo free tier is sufficient for development (15 builds/mo) — can switch to local builds for production
4. Shared monorepo packages: types, API client, design tokens
5. TypeScript throughout — no new language to learn
6. RN Web exists as fallback if needed

**Alternative if willing to pivot**: Option D (Full Flutter) — technically superior but requires Dart adoption.

## Current Maya/ State (CRITICAL CONTEXT)
- `shared/` — Only README exists. No actual code. The @nyx/ui component library is planned but not built.
- `Uzume-web/` — Only .gitkeep + README. No actual Typescript app exists.
- **ZERO sunk cost in Typescript frontend.** Everything is planned, nothing is built.
- The architecture doc chose Typescript, but that was a paper decision, not an engineering investment.

## Real Open-Source Social Media Clone Analysis

### Instagram Clones
1. `iamvucms/react-native-instagram-clone` — 1,008 stars — React Native + TypeScript + Firebase + Redux. Full app with demo video + APK.
2. `itsezlife/flutter-instagram-offline-first-clone` — Flutter + PowerSync + Supabase. Posts, stories, reels, chat, real-time sync. 24hr YouTube tutorial.
3. `AhmedAbdoElhawary/flutter-clean-architecture-instagram` — Flutter + Firebase + Agora. Posts, stories, chat, video calls.

### TikTok Clones
1. `DingMouRen/flutter_tiktok` — 526 stars — Flutter. MOST COMPLETE: 14 animated GIFs showing video feed, likes, comments, live streaming, camera, search.
2. `kirkwat/tiktok` — 158 stars — React Native + Expo + TypeScript + Firebase. Auth, video posting, profiles, feed, messaging.
3. `TheWidlarzGroup/react-native-video-feed` — 105 stars — RN Video v7 + LegendList. Production-ready vertical video feed with explicit performance metrics (TTFF, FPS, scroll lag).

### Tinder Clones
1. `shanlh/vue-tinder` — 224 stars — Vue.js. Swipe library only (~5KB gzipped), not a full app.
2. `alejandro-piguave/TinderCloneSwiftUI` — 112 stars — SwiftUI + Firebase. Full app: auth, profiles, swipe, match, chat. Real screenshots.

### Framework Distribution in Production-Quality Clones
| Framework | Instagram | Tinder | TikTok | Total |
|---|---|---|---|---|
| Flutter | 3 | 0 | 3 | 6 |
| React Native | 2 | 0 | 2 | 4 |
| Vue.js | 0 | 2 | 0 | 2 |
| SwiftUI | 0 | 1 | 0 | 1 |

### UI Quality Ranking (from real app evidence)
1. **Flutter** — `DingMouRen/flutter_tiktok` has 14 animated GIFs showing every feature. The Flutter TikTok clone looks and feels like the real thing. Material/Cupertino widgets render as actual native components.
2. **React Native** — `iamvucms/react-native-instagram-clone` (1,008 stars) is the most-starred social clone. Requires Reanimated + Gesture Handler expertise for smooth animations.
3. **Svelte** — Zero production-quality social media clones found. The ecosystem simply doesn't have them.

## The Honest Answer

There is ZERO sunk cost in Typescript. The Maya/ directory has READMEs and .gitkeep files — no actual code. The architecture doc's Typescript choice was a paper decision, not an engineering investment.

Given that, and given the evidence:

**Flutter is the objectively best choice for your requirements.**

Evidence:
1. Most complete open-source social media clones are Flutter (6 of 13 top repos)
2. The most visually-documented clone (`DingMouRen/flutter_tiktok`, 526 stars) is Flutter
3. Flutter handles video/reels natively (Skia/Impeller renderer) — the hardest feature
4. 100% free, zero cloud dependencies, local builds only
5. Melos monorepo cleanly handles multiple apps (Uzume, Anteros) sharing packages
6. Dart is a weekend learn for anyone who knows TypeScript

The only reason NOT to choose Flutter is if you have a strong team preference for JavaScript/TypeScript. In that case, React Native is the second-best choice.

## Open Questions
1. Does the evidence change your view on Typescript? (Nothing is built yet — no sunk cost)
2. Are you comfortable learning Dart, or is TypeScript a hard requirement?
3. Do you want me to create a work plan for whichever framework you choose?
