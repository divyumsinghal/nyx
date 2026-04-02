/**
 * Nyx account client — account management, settings, linked apps.
 * GET/PUT /api/nyx/account/*
 */
import { api, type ApiResponse } from "../client";

export interface NyxAccount {
  id: string;
  email: string;
  phone?: string;
  display_name: string;
  avatar_url?: string;
  created_at: string;
  linked_apps: LinkedApp[];
}

export interface LinkedApp {
  app: "uzume" | "anteros" | "themis";
  alias: string;
  linked_at: string;
}

export interface UpdateAccountRequest {
  display_name?: string;
  phone?: string;
}

export interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}

export interface PushToken {
  token: string;
  platform: "web" | "ios" | "android";
}

export const accountApi = {
  getMe(): Promise<NyxAccount> {
    return api.get<NyxAccount>("/api/nyx/account/me");
  },

  updateMe(req: UpdateAccountRequest): Promise<NyxAccount> {
    return api.put<NyxAccount>("/api/nyx/account/me", req);
  },

  changePassword(req: ChangePasswordRequest): Promise<void> {
    return api.post<void>("/api/nyx/account/password", req);
  },

  getLinkedApps(): Promise<ApiResponse<LinkedApp[]>> {
    return api.get<ApiResponse<LinkedApp[]>>("/api/nyx/account/linked-apps");
  },

  registerPushToken(req: PushToken): Promise<void> {
    return api.post<void>("/api/nyx/account/push-token", req);
  },

  deleteAccount(): Promise<void> {
    return api.delete<void>("/api/nyx/account/me");
  },
};
