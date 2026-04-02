/**
 * Nyx account portal auth — session + whoami only. Uzume profile state lives in uzume-web.
 */
import React, { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import AsyncStorage from "@react-native-async-storage/async-storage";
import { authApi, setAuthToken } from "@nyx/api";
import type { WhoAmI } from "@nyx/api";

interface AuthState {
  isLoading: boolean;
  isAuthenticated: boolean;
  user: WhoAmI | null;
  login: (identifier: string, password: string) => Promise<void>;
  register: (email: string, password: string, displayName: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthState | null>(null);
const TOKEN_KEY = "nyx_session_token";

export function AuthProvider({ children }: { children: ReactNode }) {
  const [isLoading, setIsLoading] = useState(true);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [user, setUser] = useState<WhoAmI | null>(null);

  useEffect(() => {
    restoreSession();
  }, []);

  async function restoreSession() {
    try {
      const token = await AsyncStorage.getItem(TOKEN_KEY);
      if (token) {
        setAuthToken(token);
        const whoami = await authApi.whoami();
        setUser(whoami);
        setIsAuthenticated(true);
      }
    } catch {
      await AsyncStorage.removeItem(TOKEN_KEY);
      setAuthToken(null);
    } finally {
      setIsLoading(false);
    }
  }

  async function login(identifier: string, password: string) {
    const session = await authApi.login({ identifier, password });
    await AsyncStorage.setItem(TOKEN_KEY, session.session_token);
    setAuthToken(session.session_token);
    const whoami = await authApi.whoami();
    setUser(whoami);
    setIsAuthenticated(true);
  }

  async function register(email: string, password: string, displayName: string) {
    const session = await authApi.register({ email, password, display_name: displayName });
    await AsyncStorage.setItem(TOKEN_KEY, session.session_token);
    setAuthToken(session.session_token);
    const whoami = await authApi.whoami();
    setUser(whoami);
    setIsAuthenticated(true);
  }

  async function logout() {
    try {
      await authApi.logout();
    } catch {}
    await AsyncStorage.removeItem(TOKEN_KEY);
    setAuthToken(null);
    setUser(null);
    setIsAuthenticated(false);
  }

  return (
    <AuthContext.Provider value={{ isLoading, isAuthenticated, user, login, register, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthState {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
}
