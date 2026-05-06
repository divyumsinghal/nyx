/**
 * Nyx auth context.
 *
 * Manages registration (OTP-first, Instagram-style) and login state.
 * Stores the Nyx JWT + user profile in AsyncStorage.
 * The JWT is a Bearer token for all protected API calls via Heimdall.
 */
import React, { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import AsyncStorage from "@react-native-async-storage/async-storage";
import { setAuthToken } from "@nyx/api";
import {
  initRegistration,
  sendRegistrationOtp,
  verifyRegistrationOtp,
  initLogin,
  loginWithPassword,
  exchangeToken,
  KratosError,
  type WhoAmI,
} from "@nyx/api";

// ── Persisted session shape ───────────────────────────────────────────────────

interface PersistedSession {
  token: string;
  user: WhoAmI;
  expires_at: number; // unix ms
}

const SESSION_KEY = "nyx_session_v2";

// ── Registration state machine ────────────────────────────────────────────────

export type RegistrationStep = "email" | "otp" | "username" | "password" | "done";

export interface RegistrationState {
  step: RegistrationStep;
  flowId: string;
  email: string;
}

// ── Context interface ─────────────────────────────────────────────────────────

interface AuthContextValue {
  // Session
  isLoading: boolean;
  isAuthenticated: boolean;
  user: WhoAmI | null;

  // Registration (multi-step)
  registration: RegistrationState | null;
  startRegistration: (email: string) => Promise<void>;
  verifyOtp: (code: string, nyxId: string) => Promise<void>;
  completeRegistration: () => void; // called after optional password step

  // Login
  login: (identifier: string, password: string) => Promise<void>;

  // Session
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

// ── Provider ──────────────────────────────────────────────────────────────────

export function AuthProvider({ children }: { children: ReactNode }) {
  const [isLoading, setIsLoading] = useState(true);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [user, setUser] = useState<WhoAmI | null>(null);
  const [registration, setRegistration] = useState<RegistrationState | null>(null);

  useEffect(() => {
    restoreSession();
  }, []);

  // ── Session restore ────────────────────────────────────────────────────────

  async function restoreSession() {
    try {
      const raw = await AsyncStorage.getItem(SESSION_KEY);
      if (!raw) return;
      const session: PersistedSession = JSON.parse(raw);

      // Treat expired sessions as logged out (5 min grace period)
      if (session.expires_at < Date.now() - 5 * 60_000) {
        await AsyncStorage.removeItem(SESSION_KEY);
        return;
      }

      setAuthToken(session.token);
      setUser(session.user);
      setIsAuthenticated(true);
    } catch {
      await AsyncStorage.removeItem(SESSION_KEY);
      setAuthToken(null);
    } finally {
      setIsLoading(false);
    }
  }

  // ── Save session ───────────────────────────────────────────────────────────

  async function saveSession(token: string, user: WhoAmI, expiresIn: number) {
    const session: PersistedSession = {
      token,
      user,
      expires_at: Date.now() + expiresIn * 1000,
    };
    await AsyncStorage.setItem(SESSION_KEY, JSON.stringify(session));
    setAuthToken(token);
    setUser(user);
    setIsAuthenticated(true);
  }

  // ── Registration steps ─────────────────────────────────────────────────────

  async function startRegistration(email: string) {
    const flowId = await initRegistration();
    await sendRegistrationOtp(flowId, email);
    setRegistration({ step: "otp", flowId, email });
  }

  async function verifyOtp(code: string, nyxId: string) {
    if (!registration) throw new Error("No registration in progress");
    const { flowId, email } = registration;

    const result = await verifyRegistrationOtp(flowId, email, nyxId, code);

    // Exchange Kratos session token → Nyx JWT immediately
    const { access_token, expires_in } = await exchangeToken(result.session_token);

    const whoami: WhoAmI = {
      id: result.identity.id,
      email: result.identity.traits.email,
      nyx_id: result.identity.traits.nyx_id,
      display_name: result.identity.traits.display_name,
    };

    await saveSession(access_token, whoami, expires_in);
    setRegistration(null);
  }

  function completeRegistration() {
    setRegistration(null);
  }

  // ── Login ──────────────────────────────────────────────────────────────────

  async function login(identifier: string, password: string) {
    const flowId = await initLogin();
    const result = await loginWithPassword(flowId, identifier, password);

    const { access_token, expires_in } = await exchangeToken(result.session_token);

    const whoami: WhoAmI = {
      id: result.session.identity.id,
      email: result.session.identity.traits.email,
      nyx_id: result.session.identity.traits.nyx_id,
      display_name: result.session.identity.traits.display_name,
    };

    await saveSession(access_token, whoami, expires_in);
  }

  // ── Logout ─────────────────────────────────────────────────────────────────

  async function logout() {
    await AsyncStorage.removeItem(SESSION_KEY);
    setAuthToken(null);
    setUser(null);
    setIsAuthenticated(false);
    setRegistration(null);
  }

  return (
    <AuthContext.Provider
      value={{
        isLoading,
        isAuthenticated,
        user,
        registration,
        startRegistration,
        verifyOtp,
        completeRegistration,
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
}

// Helper: format Kratos errors for display
export function formatAuthError(err: unknown): string {
  if (err instanceof KratosError) return err.message;
  if (err instanceof Error) return err.message;
  return "Something went wrong. Please try again.";
}
