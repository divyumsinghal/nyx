import React, {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import AsyncStorage from "@react-native-async-storage/async-storage";
import { authApi, profilesApi, setAuthToken } from "@nyx/api";
import type { WhoAmI } from "@nyx/api";
import type { UzumeProfile } from "@nyx/api";

interface AuthState {
  isLoading: boolean;
  isAuthenticated: boolean;
  user: WhoAmI | null;
  profile: UzumeProfile | null;
  login: (identifier: string, password: string) => Promise<void>;
  register: (
    email: string,
    password: string,
    displayName: string,
  ) => Promise<void>;
  logout: () => Promise<void>;
  refreshProfile: () => Promise<void>;
}

const AuthContext = createContext<AuthState | null>(null);
const TOKEN_KEY = "nyx_session_token";

export function AuthProvider({ children }: { children: ReactNode }) {
  const [isLoading, setIsLoading] = useState(true);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [user, setUser] = useState<WhoAmI | null>(null);
  const [profile, setProfile] = useState<UzumeProfile | null>(null);

  useEffect(() => {
    restoreSession();
  }, []);

  async function restoreSession() {
    try {
      const token = await AsyncStorage.getItem(TOKEN_KEY);
      if (token) {
        setAuthToken(token);
        const [whoami, myProfile] = await Promise.all([
          authApi.whoami(),
          profilesApi.getMyProfile(),
        ]);
        setUser(whoami);
        setProfile(myProfile);
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
    const [whoami, myProfile] = await Promise.all([
      authApi.whoami(),
      profilesApi.getMyProfile(),
    ]);
    setUser(whoami);
    setProfile(myProfile);
    setIsAuthenticated(true);
  }

  async function register(
    email: string,
    password: string,
    displayName: string,
  ) {
    const session = await authApi.register({
      email,
      password,
      display_name: displayName,
    });
    await AsyncStorage.setItem(TOKEN_KEY, session.session_token);
    const whoami = await authApi.whoami();
    setUser(whoami);
    setIsAuthenticated(true);
  }

  async function logout() {
    await authApi.logout();
    await AsyncStorage.removeItem(TOKEN_KEY);
    setUser(null);
    setProfile(null);
    setIsAuthenticated(false);
  }

  async function refreshProfile() {
    try {
      const myProfile = await profilesApi.getMyProfile();
      setProfile(myProfile);
    } catch {}
  }

  return (
    <AuthContext.Provider
      value={{
        isLoading,
        isAuthenticated,
        user,
        profile,
        login,
        register,
        logout,
        refreshProfile,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthState {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
}
