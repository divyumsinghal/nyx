# Paid

## Paid Service Issues Found

**1. Twilio (definitely paid)** — in `Prithvi/compose/infra.yml`, Kratos is configured with:
```
TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, TWILIO_PHONE_NUMBER
```
Kratos uses Twilio for SMS/phone OTP verification. Twilio has no free tier — it's pay-per-SMS.

**2. Apple Push Notifications (APNs)** — Gorush is configured with `APNS_KEY_ID`, `APNS_TEAM_ID`. Sending push to iOS requires an Apple Developer account ($99/year). Not free.

**3. FCM (Firebase Cloud Messaging)** — Gorush has `FCM_API_KEY`. FCM itself is free but requires a Google Firebase project.

**4. DragonflyDB** — BSL 1.1 license. Technically free to run but not truly open source (converts to Apache after 4 years). It's a drop-in Redis replacement — we could swap it for plain Redis (BSD licensed) with zero code changes since the client speaks Redis protocol.

**5. SMTP** — `SMTP_CONNECTION_URI` in Kratos config. You'd need an SMTP server. If self-hosted (Postfix/Mailcow), it's free. But if using SendGrid/Mailgun etc., those are paid.

---

**My recommendations for discussion:**

| Issue | Options |
|-------|---------|
| Twilio SMS OTP | (a) Drop phone auth, use email-only in Kratos. (b) Use self-hosted SMS gateway (requires SIM hardware). (c) Keep Twilio opt-in for prod but don't require it |
| APNs | (a) Remove iOS push for now, use WebSockets/SSE. (b) Treat as opt-in for users who have a dev account |
| FCM | Free to use, keep it |
| DragonflyDB | Swap for Redis (identical API, truly open source) |

**Before we start the service-by-service audit:** What's your call on Twilio and APNs? These affect how Kratos (auth) and Ushas (notifications) are configured, which cascades to every service. I want to sort the paid-service issues first since fixing them changes the infrastructure config, then we do the service testing on a clean foundation.

Also confirming: **which service do you want to start with?** My suggestion is `Uzume-profiles` (port 3001) — it's the first real user-facing service and everything else builds on it.

Build confirmed clean. Waiting on your decisions about the paid service issues before we proceed.

Still waiting on your call. To recap the two decisions that matter most:

1. **Twilio** — drop phone auth entirely (email-only in Kratos), or keep it opt-in for prod deployments?
2. **APNs/iOS push** — remove for now and use WebSockets/SSE, or keep as opt-in?

Everything else (DragonflyDB → Redis, SMTP self-hosted) I can swap without asking — those are straightforward. Just need your answer on those two before we start service testing.

Still waiting on your answer — the background builds keep coming back clean, which is good. Just need those two decisions:

1. **Twilio/SMS** — drop phone auth (email-only) or keep opt-in?
2. **APNs/iOS push** — remove (use WebSockets) or keep as opt-in?