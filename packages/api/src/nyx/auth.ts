/**
 * Nyx auth — Ory Kratos self-service API via Heimdall proxy.
 *
 * All Kratos endpoints live under /api/nyx/auth/* (Heimdall strips the prefix).
 * The token exchange endpoint /api/nyx/auth/token is handled by Heimdall directly.
 *
 * Registration flow (OTP-first, Instagram-style):
 *   1. initRegistration()         → flowId
 *   2. sendRegistrationOtp()      → flowId (OTP email sent)
 *   3. verifyRegistrationOtp()    → { session_token, identity }
 *   4. exchangeToken()            → { access_token }
 *
 * Login flow:
 *   1. initLogin()                → flowId
 *   2. loginWithPassword()        → { session_token, identity }
 *   3. exchangeToken()            → { access_token }
 */

const BASE = "/api/nyx/auth";

// ── Gateway URL ───────────────────────────────────────────────────────────────

/**
 * Returns the gateway URL (Caddy HTTPS edge).
 * Throws if EXPO_PUBLIC_GATEWAY_URL is not set — there is no fallback.
 * All traffic must go through Caddy (https://localhost:3443 locally).
 * Direct-to-Heimdall (http://localhost:3000) is never acceptable.
 */
function getGatewayUrl(): string {
  const url =
    (typeof process !== "undefined" &&
      (process.env.EXPO_PUBLIC_GATEWAY_URL ?? process.env.GATEWAY_URL)?.trim()) ||
    undefined;
  if (!url) {
    throw new Error(
      "[nyx/auth] EXPO_PUBLIC_GATEWAY_URL is not set.\n" +
      "Create Maya/nyx-web/.env.local with:\n" +
      "  EXPO_PUBLIC_GATEWAY_URL=https://localhost:3443"
    );
  }
  return url;
}

// ── Types ─────────────────────────────────────────────────────────────────────

export interface KratosFlow {
  id: string;
  type: string;
  expires_at: string;
  ui: KratosUi;
}

export interface KratosUi {
  action: string;
  method: string;
  nodes: KratosNode[];
  messages?: KratosMessage[];
}

export interface KratosNode {
  type: string;
  group: string;
  attributes: { name: string; value?: string; [k: string]: unknown };
  messages: KratosMessage[];
  meta: { label?: { id: number; text: string; type: string } };
}

export interface KratosMessage {
  id: number;
  type: "error" | "info" | "success";
  text: string;
}

export interface KratosIdentity {
  id: string;
  traits: {
    email: string;
    nyx_id: string;
    display_name?: string;
  };
  created_at: string;
}

export interface KratosSession {
  id: string;
  identity: KratosIdentity;
  active: boolean;
  expires_at: string;
}

export interface KratosRegistrationResponse {
  session_token: string;
  session: KratosSession;
  identity: KratosIdentity;
}

export interface KratosLoginResponse {
  session_token: string;
  session: KratosSession;
}

export interface NyxTokenResponse {
  access_token: string;
  token_type: "Bearer";
  expires_in: number;
}

/** Simplified user shape stored in AuthContext. */
export interface WhoAmI {
  id: string;
  email: string;
  nyx_id: string;
  display_name?: string;
}

// ── Error handling ────────────────────────────────────────────────────────────

export class KratosError extends Error {
  constructor(
    public status: number,
    public messages: KratosMessage[],
    public flowId?: string,
  ) {
    const text = messages.map((m) => m.text).join("; ") || `HTTP ${status}`;
    super(text);
    this.name = "KratosError";
  }
}

/** Extract human-readable messages from a Kratos error/422 response. */
function extractMessages(body: unknown): KratosMessage[] {
  if (!body || typeof body !== "object") return [];
  const b = body as Record<string, unknown>;

  // Top-level ui messages
  const ui = b.ui as { messages?: KratosMessage[]; nodes?: KratosNode[] } | undefined;
  const msgs: KratosMessage[] = [];

  if (ui?.messages?.length) msgs.push(...ui.messages);

  // Per-node messages
  if (ui?.nodes) {
    for (const node of ui.nodes) {
      if (node.messages?.length) msgs.push(...node.messages);
    }
  }

  // error.reason field
  const err = b.error as { reason?: string; message?: string } | undefined;
  if (err?.reason) msgs.push({ id: 0, type: "error", text: err.reason });
  else if (err?.message) msgs.push({ id: 0, type: "error", text: err.message });

  return msgs;
}

async function kratosRequest<T>(
  method: string,
  path: string,
  body?: unknown,
  sessionToken?: string,
): Promise<T> {
  const headers: Record<string, string> = {
    Accept: "application/json",
  };
  if (body !== undefined) headers["Content-Type"] = "application/json";
  if (sessionToken) headers["X-Session-Token"] = sessionToken;

  const baseUrl = getGatewayUrl();

  const resp = await fetch(`${baseUrl.replace(/\/+$/, "")}${path}`, {
    method,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });

  if (resp.status === 204) return undefined as T;

  let parsed: unknown;
  try {
    parsed = await resp.json();
  } catch {
    throw new KratosError(resp.status, [{ id: 0, type: "error", text: resp.statusText }]);
  }

  if (!resp.ok) {
    const msgs = extractMessages(parsed);
    const p = parsed as Record<string, unknown>;
    const flowId = (p?.id as string) || ((p?.ui as { action?: string })?.action?.match(/flow=([^&]+)/)?.[1]);
    throw new KratosError(resp.status, msgs.length ? msgs : [{ id: 0, type: "error", text: resp.statusText }], flowId);
  }

  return parsed as T;
}

// ── Registration ──────────────────────────────────────────────────────────────

/** Step 1: Start a new registration flow. Returns the flow ID. */
export async function initRegistration(): Promise<string> {
  const flow = await kratosRequest<KratosFlow>("GET", `${BASE}/self-service/registration/api`);
  return flow.id;
}

/** Step 2: Send OTP to email. Returns same flowId (keep using it for step 3). */
export async function sendRegistrationOtp(flowId: string, email: string): Promise<void> {
  try {
    await kratosRequest<unknown>(
      "POST",
      `${BASE}/self-service/registration?flow=${flowId}`,
      { method: "code", traits: { email } },
    );
  } catch (err) {
    // Kratos returns 422 with state=sent_email — that's the success case for OTP send.
    if (err instanceof KratosError && err.status === 422) {
      const msgs = err.messages.filter((m) => m.type === "error");
      if (msgs.length === 0) return; // no errors = OTP sent successfully
      throw err;
    }
    throw err;
  }
}

/** Step 3: Submit OTP code + nyx_id to complete registration. */
export async function verifyRegistrationOtp(
  flowId: string,
  email: string,
  nyxId: string,
  code: string,
): Promise<KratosRegistrationResponse> {
  return kratosRequest<KratosRegistrationResponse>(
    "POST",
    `${BASE}/self-service/registration?flow=${flowId}`,
    { method: "code", code, traits: { email, nyx_id: nyxId } },
  );
}

// ── Login ─────────────────────────────────────────────────────────────────────

/** Start a new login flow. Returns the flow ID. */
export async function initLogin(): Promise<string> {
  const flow = await kratosRequest<KratosFlow>("GET", `${BASE}/self-service/login/api`);
  return flow.id;
}

/** Submit identifier (email or nyx_id) + password. Returns session_token. */
export async function loginWithPassword(
  flowId: string,
  identifier: string,
  password: string,
): Promise<KratosLoginResponse> {
  return kratosRequest<KratosLoginResponse>(
    "POST",
    `${BASE}/self-service/login?flow=${flowId}`,
    { method: "password", identifier, password },
  );
}

// ── Token exchange ────────────────────────────────────────────────────────────

/** Exchange a Kratos session token for a Nyx JWT (Bearer token for all API calls). */
export async function exchangeToken(sessionToken: string): Promise<NyxTokenResponse> {
  const baseUrl = getGatewayUrl();

  const resp = await fetch(`${baseUrl.replace(/\/+$/, "")}/api/nyx/auth/token`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ session_token: sessionToken }),
  });

  if (!resp.ok) {
    const body = await resp.json().catch(() => ({}));
    throw new KratosError(resp.status, [
      { id: 0, type: "error", text: (body as { error?: string }).error || "Token exchange failed" },
    ]);
  }

  return resp.json();
}

// ── Nyx ID availability ───────────────────────────────────────────────────────

export interface NyxIdCheckResult {
  available: boolean;
  id: string;
  reason?: string;
}

export async function checkNyxIdAvailability(nyxId: string): Promise<NyxIdCheckResult> {
  const baseUrl = getGatewayUrl();

  const resp = await fetch(
    `${baseUrl.replace(/\/+$/, "")}/api/nyx/id/check-availability?id=${encodeURIComponent(nyxId)}`,
  );
  return resp.json();
}

// ── Convenience re-exports ────────────────────────────────────────────────────

/** @deprecated Use the individual functions above. Kept for compatibility. */
export const authApi = {
  initRegistration,
  sendRegistrationOtp,
  verifyRegistrationOtp,
  initLogin,
  loginWithPassword,
  exchangeToken,
  checkNyxIdAvailability,
};
