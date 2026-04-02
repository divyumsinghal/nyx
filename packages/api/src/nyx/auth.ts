/**
 * Nyx auth client — maps to Ory Kratos flows via Heimdall.
 * POST /api/nyx/auth/*
 */
import { api, setAuthToken } from "../client";

export interface LoginRequest {
  identifier: string; // email or phone
  password: string;
}

export interface RegisterRequest {
  email: string;
  phone?: string;
  password: string;
  display_name: string;
}

export interface AuthSession {
  session_token: string;
  user_id: string;
  expires_at: string;
}

export interface WhoAmI {
  id: string;
  email: string;
  phone?: string;
  /** Display name from profile / registration when available */
  display_name?: string;
  created_at: string;
}

export const authApi = {
  /**
   * Login with email/phone + password via Kratos.
   * On success, stores token in client.
   */
  async login(req: LoginRequest): Promise<AuthSession> {
    const result = await api.post<AuthSession>("/api/nyx/auth/login", req);
    setAuthToken(result.session_token);
    return result;
  },

  /**
   * Register a new account.
   */
  async register(req: RegisterRequest): Promise<AuthSession> {
    const result = await api.post<AuthSession>("/api/nyx/auth/register", req);
    setAuthToken(result.session_token);
    return result;
  },

  /**
   * Logout — clears token locally (Kratos session revocation optional).
   */
  async logout(): Promise<void> {
    try {
      await api.post<void>("/api/nyx/auth/logout");
    } finally {
      setAuthToken(null);
    }
  },

  /**
   * Get current session identity.
   */
  whoami(): Promise<WhoAmI> {
    return api.get<WhoAmI>("/api/nyx/auth/whoami");
  },

  /**
   * Request password reset email.
   */
  requestPasswordReset(email: string): Promise<void> {
    return api.post<void>("/api/nyx/auth/recovery", { email });
  },
};
