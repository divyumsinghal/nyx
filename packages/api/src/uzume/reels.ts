/**
 * Uzume reels client — short-form video, algorithmic feed.
 * /api/uzume/reels/*  →  Uzume-reels :3004
 */
import { api, type ApiResponse } from "../client";

export interface Reel {
  id: string;
  author: {
    alias: string;
    display_name: string;
    avatar_url?: string;
    is_following?: boolean;
  };
  video_url: string;
  thumbnail_url?: string;
  audio_title?: string;
  audio_artist?: string;
  audio_url?: string;
  caption?: string;
  tags: string[];
  likes_count: number;
  comments_count: number;
  saves_count: number;
  views_count: number;
  shares_count: number;
  has_liked?: boolean;
  has_saved?: boolean;
  duration_ms: number;
  created_at: string;
}

export interface ReelComment {
  id: string;
  author: { alias: string; display_name: string; avatar_url?: string };
  body: string;
  likes_count: number;
  has_liked?: boolean;
  created_at: string;
}

export const reelsApi = {
  getFeed(cursor?: string): Promise<ApiResponse<Reel[]>> {
    const q = cursor ? `?cursor=${cursor}` : "";
    return api.get<ApiResponse<Reel[]>>(`/api/uzume/reels/feed${q}`);
  },

  getReel(reelId: string): Promise<Reel> {
    return api.get<Reel>(`/api/uzume/reels/${reelId}`);
  },

  getProfileReels(alias: string, cursor?: string): Promise<ApiResponse<Reel[]>> {
    const q = cursor ? `?cursor=${cursor}` : "";
    return api.get<ApiResponse<Reel[]>>(
      `/api/uzume/reels/profiles/${alias}${q}`,
    );
  },

  likeReel(reelId: string): Promise<{ likes_count: number }> {
    return api.post<{ likes_count: number }>(`/api/uzume/reels/${reelId}/like`);
  },

  unlikeReel(reelId: string): Promise<{ likes_count: number }> {
    return api.delete<{ likes_count: number }>(`/api/uzume/reels/${reelId}/like`);
  },

  saveReel(reelId: string): Promise<void> {
    return api.post<void>(`/api/uzume/reels/${reelId}/save`);
  },

  unsaveReel(reelId: string): Promise<void> {
    return api.delete<void>(`/api/uzume/reels/${reelId}/save`);
  },

  recordView(reelId: string, watched_ms: number): Promise<void> {
    return api.post<void>(`/api/uzume/reels/${reelId}/view`, { watched_ms });
  },

  getComments(reelId: string, cursor?: string): Promise<ApiResponse<ReelComment[]>> {
    const q = cursor ? `?cursor=${cursor}` : "";
    return api.get<ApiResponse<ReelComment[]>>(
      `/api/uzume/reels/${reelId}/comments${q}`,
    );
  },

  createComment(reelId: string, body: string): Promise<ReelComment> {
    return api.post<ReelComment>(`/api/uzume/reels/${reelId}/comments`, { body });
  },

  getUploadUrl(): Promise<{ upload_url: string; media_id: string }> {
    return api.post<{ upload_url: string; media_id: string }>(
      "/api/uzume/reels/media/upload-url",
    );
  },
};
