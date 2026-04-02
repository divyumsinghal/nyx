/**
 * Base HTTP client for Heimdall gateway.
 * All requests go through /api/* — never direct service URLs.
 */

const GATEWAY_URL =
  typeof window !== "undefined"
    ? (process.env.EXPO_PUBLIC_GATEWAY_URL ?? "http://localhost:3000")
    : (process.env.GATEWAY_URL ?? "http://localhost:3000");

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
