/**
 * Uzume discover client — explore, trending, search.
 * /api/uzume/discover/*  →  Uzume-discover :3005
 */
import { api, type ApiResponse } from "../client";
import type { Post } from "./feed";
import type { UzumeProfile } from "./profiles";
import type { Reel } from "./reels";

export interface TrendingTag {
  tag: string;
  posts_count: number;
  trend_score: number;
}

export interface SearchResults {
  profiles: UzumeProfile[];
  posts: Post[];
  reels: Reel[];
  tags: TrendingTag[];
}

export interface ExploreGrid {
  posts: Post[];
  reels: Reel[];
}

export const discoverApi = {
  getExplore(cursor?: string): Promise<ApiResponse<ExploreGrid>> {
    const q = cursor ? `?cursor=${cursor}` : "";
    return api.get<ApiResponse<ExploreGrid>>(`/api/uzume/discover/explore${q}`);
  },

  getTrending(): Promise<TrendingTag[]> {
    return api.get<TrendingTag[]>("/api/uzume/discover/trending");
  },

  search(query: string, cursor?: string): Promise<SearchResults> {
    const q = new URLSearchParams({ q: query });
    if (cursor) q.set("cursor", cursor);
    return api.get<SearchResults>(`/api/uzume/discover/search?${q}`);
  },

  searchProfiles(query: string, cursor?: string): Promise<ApiResponse<UzumeProfile[]>> {
    const q = new URLSearchParams({ q: query });
    if (cursor) q.set("cursor", cursor);
    return api.get<ApiResponse<UzumeProfile[]>>(
      `/api/uzume/discover/search/profiles?${q}`,
    );
  },

  getByTag(tag: string, cursor?: string): Promise<ApiResponse<Post[]>> {
    const q = cursor ? `?cursor=${cursor}` : "";
    return api.get<ApiResponse<Post[]>>(
      `/api/uzume/discover/tags/${encodeURIComponent(tag)}${q}`,
    );
  },

  getSuggestedProfiles(): Promise<UzumeProfile[]> {
    return api.get<UzumeProfile[]>("/api/uzume/discover/suggested");
  },
};
