/**
 * Uzume profiles client — profiles, follow graph, block/mute.
 * /api/uzume/profiles/*  →  Uzume-profiles :3001
 */
import { api, type ApiResponse, type CursorResponse } from "../client";

export interface UzumeProfile {
  id: string;
  alias: string;
  display_name: string;
  bio?: string;
  avatar_url?: string;
  header_url?: string;
  website?: string;
  location?: string;
  followers_count: number;
  following_count: number;
  posts_count: number;
  is_following?: boolean;
  is_follower?: boolean;
  is_blocked?: boolean;
  is_muted?: boolean;
  is_verified?: boolean;
  is_private?: boolean;
  created_at: string;
}

export interface UpdateProfileRequest {
  display_name?: string;
  bio?: string;
  website?: string;
  location?: string;
  is_private?: boolean;
}

export interface FollowResult {
  is_following: boolean;
  followers_count: number;
}

export interface FollowerEntry {
  profile: UzumeProfile;
  followed_at: string;
}

export const profilesApi = {
  getProfile(alias: string): Promise<UzumeProfile> {
    return api.get<UzumeProfile>(`/api/uzume/profiles/${alias}`);
  },

  getMyProfile(): Promise<UzumeProfile> {
    return api.get<UzumeProfile>("/api/uzume/profiles/me");
  },

  updateMyProfile(req: UpdateProfileRequest): Promise<UzumeProfile> {
    return api.put<UzumeProfile>("/api/uzume/profiles/me", req);
  },

  getAvatarUploadUrl(): Promise<{ upload_url: string; media_id: string }> {
    return api.post<{ upload_url: string; media_id: string }>(
      "/api/uzume/profiles/me/avatar",
    );
  },

  getHeaderUploadUrl(): Promise<{ upload_url: string; media_id: string }> {
    return api.post<{ upload_url: string; media_id: string }>(
      "/api/uzume/profiles/me/header",
    );
  },

  follow(alias: string): Promise<FollowResult> {
    return api.post<FollowResult>(`/api/uzume/profiles/${alias}/follow`);
  },

  unfollow(alias: string): Promise<FollowResult> {
    return api.delete<FollowResult>(`/api/uzume/profiles/${alias}/follow`);
  },

  getFollowers(
    alias: string,
    cursor?: string,
  ): Promise<ApiResponse<FollowerEntry[]>> {
    const q = cursor ? `?cursor=${cursor}` : "";
    return api.get<ApiResponse<FollowerEntry[]>>(
      `/api/uzume/profiles/${alias}/followers${q}`,
    );
  },

  getFollowing(
    alias: string,
    cursor?: string,
  ): Promise<ApiResponse<FollowerEntry[]>> {
    const q = cursor ? `?cursor=${cursor}` : "";
    return api.get<ApiResponse<FollowerEntry[]>>(
      `/api/uzume/profiles/${alias}/following${q}`,
    );
  },

  block(alias: string): Promise<void> {
    return api.post<void>(`/api/uzume/profiles/${alias}/block`);
  },

  unblock(alias: string): Promise<void> {
    return api.delete<void>(`/api/uzume/profiles/${alias}/block`);
  },

  mute(alias: string): Promise<void> {
    return api.post<void>(`/api/uzume/profiles/${alias}/mute`);
  },

  unmute(alias: string): Promise<void> {
    return api.delete<void>(`/api/uzume/profiles/${alias}/mute`);
  },
};
