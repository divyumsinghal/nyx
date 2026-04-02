/**
 * Uzume stories client — stories, highlights, 24h TTL.
 * /api/uzume/stories/*  →  Uzume-stories :3003
 */
import { api, type ApiResponse } from "../client";

export interface Story {
  id: string;
  author: {
    alias: string;
    display_name: string;
    avatar_url?: string;
  };
  media_url: string;
  thumbnail_url?: string;
  media_type: "image" | "video";
  duration_ms?: number;
  caption?: string;
  link?: string;
  views_count: number;
  has_viewed?: boolean;
  reactions_count: number;
  my_reaction?: string;
  expires_at: string;
  created_at: string;
}

export interface StoryGroup {
  alias: string;
  display_name: string;
  avatar_url?: string;
  has_unseen: boolean;
  latest_story_at: string;
  stories: Story[];
}

export interface Highlight {
  id: string;
  alias: string;
  title: string;
  cover_url?: string;
  stories_count: number;
  created_at: string;
}

export const storiesApi = {
  getFeed(): Promise<ApiResponse<StoryGroup[]>> {
    return api.get<ApiResponse<StoryGroup[]>>("/api/uzume/stories/feed");
  },

  getProfileStories(alias: string): Promise<Story[]> {
    return api.get<Story[]>(`/api/uzume/stories/profiles/${alias}`);
  },

  createStory(req: {
    media_id: string;
    caption?: string;
    link?: string;
  }): Promise<Story> {
    return api.post<Story>("/api/uzume/stories", req);
  },

  viewStory(storyId: string): Promise<void> {
    return api.post<void>(`/api/uzume/stories/${storyId}/view`);
  },

  reactToStory(storyId: string, emoji: string): Promise<void> {
    return api.post<void>(`/api/uzume/stories/${storyId}/react`, { emoji });
  },

  deleteStory(storyId: string): Promise<void> {
    return api.delete<void>(`/api/uzume/stories/${storyId}`);
  },

  getHighlights(alias: string): Promise<Highlight[]> {
    return api.get<Highlight[]>(`/api/uzume/stories/highlights/${alias}`);
  },

  createHighlight(req: {
    title: string;
    story_ids: string[];
    cover_story_id?: string;
  }): Promise<Highlight> {
    return api.post<Highlight>("/api/uzume/stories/highlights", req);
  },

  getUploadUrl(mime_type: string): Promise<{ upload_url: string; media_id: string }> {
    return api.post<{ upload_url: string; media_id: string }>(
      "/api/uzume/stories/media/upload-url",
      { mime_type },
    );
  },
};
