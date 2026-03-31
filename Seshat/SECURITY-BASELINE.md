# Security-First Baseline (Task 5)

## Scope
This baseline defines the minimum security posture for Nyx Step-1 and is enforced through mandatory local and CI gates.

## Threat Categories
1. **Cross-app unauthorized access** — app-boundary bypass that exposes identity/link data without explicit consent.
2. **Secret exposure in source control** — committed credentials, private keys, or long-lived tokens.
3. **Vulnerable dependencies and policy drift** — known CVEs or dependency policy violations entering the build.
4. **Default-open identity linking** — implicit reveal behavior instead of fail-closed privacy defaults.

## Baseline Controls
- **C1 Dependency policy gate**: `cargo deny check`
- **C2 Vulnerability gate**: `cargo audit`
- **C3 Secret scan gate**: deterministic high-confidence signature scan on tracked files
- **C4 Cross-app unauthorized-access gate**: deterministic invariant checks over migration constraints

## Mandatory Gate Commands

### Local
- `just security` (C1 + C2 + C3)
- `just gate-cross-app-unauthorized` (C4)
- `just ci` (must include both security + cross-app gate)

### CI
CI must execute the same commands in blocking mode:
- `just security`
- `just gate-cross-app-unauthorized`
- `just ci`

Any gate failure is a hard stop.

## Cross-App Unauthorized Access Abuse-Case Expectations
1. **Same-app masquerade** (`source_app == target_app`) is rejected.
   - Guard: `CHECK (source_app <> target_app)`
2. **Self-link escalation** (`source_nyx_identity_id == target_nyx_identity_id`) is rejected.
   - Guard: `CHECK (source_nyx_identity_id <> target_nyx_identity_id)`
3. **Implicit reveal** without explicit consent is denied by default.
   - Guard: `policy JSONB NOT NULL DEFAULT '{"type":"revoked"}'::jsonb`
4. **Out-of-domain app value injection** is rejected.
   - Guard: app check constraints on both `source_app` and `target_app`

## Gate-to-Control Mapping
- `just security-deny` => C1
- `just security-audit` => C2
- `just security-secret-scan` => C3
- `just gate-cross-app-unauthorized` => C4

## Determinism Rules
- Gates are non-interactive and return non-zero on violations.
- Secret scan uses fixed regex signatures for reproducible results.
- Cross-app gate validates explicit migration invariants (no flaky runtime dependency).
