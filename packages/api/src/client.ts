/**
 * Base HTTP client for Heimdall gateway.
 * All requests go through /api/* — never direct service URLs.
 *
 * Browser (Expo web / RN): set `EXPO_PUBLIC_GATEWAY_URL` to the gateway origin (no `/api` suffix).
 * Non-browser (SSR/tests/Node): `EXPO_PUBLIC_GATEWAY_URL` or `GATEWAY_URL` is read; same shape.
 */

function stripTrailingSlash(url: string): string {
  return url.replace(/\/+$/, "");
}

let warnedMissingGatewayUrl = false;

function resolveGatewayUrl(): string {
  const isBrowser = typeof window !== "undefined";
  const fromEnv = isBrowser
    ? process.env.EXPO_PUBLIC_GATEWAY_URL
    : process.env.EXPO_PUBLIC_GATEWAY_URL ?? process.env.GATEWAY_URL;

  const trimmed = fromEnv?.trim();
  if (trimmed) {
    return stripTrailingSlash(trimmed);
  }

  const fallback = "http://localhost:3000";
  const requireGatewayUrl =
    process.env.CI === "true" ||
    process.env.NYX_REQUIRE_GATEWAY_URL === "1" ||
    process.env.EXPO_PUBLIC_REQUIRE_GATEWAY_URL === "1";

  if (process.env.NODE_ENV === "production" && requireGatewayUrl) {
    throw new Error(
      "EXPO_PUBLIC_GATEWAY_URL must be set when CI=true, NYX_REQUIRE_GATEWAY_URL=1, or EXPO_PUBLIC_REQUIRE_GATEWAY_URL=1 (production bundle). Use the Heimdall gateway origin only, e.g. https://api.example.com",
    );
  }

  if (process.env.NODE_ENV === "production" && !requireGatewayUrl) {
    if (!warnedMissingGatewayUrl && typeof console !== "undefined" && console.warn) {
      warnedMissingGatewayUrl = true;
      console.warn(
        `[nyx/api] EXPO_PUBLIC_GATEWAY_URL is unset in a production bundle; using ${fallback}. ` +
          "Set EXPO_PUBLIC_GATEWAY_URL before shipping, or run CI with EXPO_PUBLIC_GATEWAY_URL set.",
      );
    }
    return fallback;
  }

  if (!warnedMissingGatewayUrl && typeof console !== "undefined" && console.warn) {
    warnedMissingGatewayUrl = true;
    console.warn(
      `[nyx/api] EXPO_PUBLIC_GATEWAY_URL (or GATEWAY_URL in Node) is unset; using ${fallback}. Set it for real deployments.`,
    );
  }
  return fallback;
}

export const GATEWAY_URL = resolveGatewayUrl();

export interface ApiResponse<T> {
  data: T;
  pagination?: CursorResponse;
}

export interface CursorResponse {
  next_cursor: string | null;
  prev_cursor: string | null;
  has_more: boolean;
}

export interface ErrorResponse {
  error: string;
  code: string;
  request_id: string;
}

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    public requestId: string,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

let _token: string | null = null;

export function setAuthToken(token: string | null) {
  _token = token;
}

export function getAuthToken(): string | null {
  return _token;
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  extraHeaders?: Record<string, string>,
): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...extraHeaders,
  };

  if (_token) {
    headers["Authorization"] = `Bearer ${_token}`;
  }

  const response = await fetch(`${GATEWAY_URL}${path}`, {
    method,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });

  if (!response.ok) {
    let errorBody: ErrorResponse;
    try {
      errorBody = await response.json();
    } catch {
      errorBody = {
        error: response.statusText,
        code: "UNKNOWN",
        request_id: response.headers.get("x-request-id") ?? "",
      };
    }
    throw new ApiError(
      response.status,
      errorBody.code,
      errorBody.request_id,
      errorBody.error,
    );
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json();
}

export const api = {
  get: <T>(path: string) => request<T>("GET", path),
  post: <T>(path: string, body?: unknown) => request<T>("POST", path, body),
  put: <T>(path: string, body?: unknown) => request<T>("PUT", path, body),
  patch: <T>(path: string, body?: unknown) => request<T>("PATCH", path, body),
  delete: <T>(path: string) => request<T>("DELETE", path),
};
