/**
 * Uzume feed client — posts, timeline, likes, comments, saves.
 * /api/uzume/feed/*  →  Uzume-feed :3002
 */
import { api, type ApiResponse } from "../client";

export interface MediaAttachment {
  id: string;
  url: string;
  thumbnail_url?: string;
  width: number;
  height: number;
  mime_type: string;
  alt_text?: string;
}

export interface Post {
  id: string;
  author: PostAuthor;
  caption?: string;
  media: MediaAttachment[];
  likes_count: number;
  comments_count: number;
  saves_count: number;
  has_liked?: boolean;
  has_saved?: boolean;
  is_pinned?: boolean;
  created_at: string;
  updated_at: string;
}

export interface PostAuthor {
  alias: string;
  display_name: string;
  avatar_url?: string;
  is_verified?: boolean;
}

export interface Comment {
  id: string;
  author: PostAuthor;
  body: string;
  likes_count: number;
  has_liked?: boolean;
  parent_id?: string;
  replies_count: number;
  created_at: string;
}

export interface CreatePostRequest {
  caption?: string;
  media_ids: string[];
}

export interface CreateCommentRequest {
  body: string;
  parent_id?: string;
}

export const feedApi = {
  // ── Timeline ─────────────────────────────────────────────────────────────

  getHomeTimeline(cursor?: string): Promise<ApiResponse<Post[]>> {
    const q = cursor ? `?cursor=${cursor}` : "";
    return api.get<ApiResponse<Post[]>>(`/api/uzume/feed/timeline${q}`);
  },

  getProfilePosts(
    alias: string,
    cursor?: string,
  ): Promise<ApiResponse<Post[]>> {
    const q = cursor ? `?cursor=${cursor}` : "";
    return api.get<ApiResponse<Post[]>>(
      `/api/uzume/feed/profiles/${alias}/posts${q}`,
    );
  },

  // ── Posts ─────────────────────────────────────────────────────────────────

  getPost(postId: string): Promise<Post> {
    return api.get<Post>(`/api/uzume/feed/posts/${postId}`);
  },

  createPost(req: CreatePostRequest): Promise<Post> {
    return api.post<Post>("/api/uzume/feed/posts", req);
  },

  deletePost(postId: string): Promise<void> {
    return api.delete<void>(`/api/uzume/feed/posts/${postId}`);
  },

  getUploadUrl(
    mime_type: string,
  ): Promise<{ upload_url: string; media_id: string }> {
    return api.post<{ upload_url: string; media_id: string }>(
      "/api/uzume/feed/media/upload-url",
      { mime_type },
    );
  },

  // ── Interactions ──────────────────────────────────────────────────────────

  likePost(postId: string): Promise<{ likes_count: number }> {
    return api.post<{ likes_count: number }>(
      `/api/uzume/feed/posts/${postId}/like`,
    );
  },

  unlikePost(postId: string): Promise<{ likes_count: number }> {
    return api.delete<{ likes_count: number }>(
      `/api/uzume/feed/posts/${postId}/like`,
    );
  },

  savePost(postId: string): Promise<void> {
    return api.post<void>(`/api/uzume/feed/posts/${postId}/save`);
  },

  unsavePost(postId: string): Promise<void> {
    return api.delete<void>(`/api/uzume/feed/posts/${postId}/save`);
  },

  getSavedPosts(cursor?: string): Promise<ApiResponse<Post[]>> {
    const q = cursor ? `?cursor=${cursor}` : "";
    return api.get<ApiResponse<Post[]>>(`/api/uzume/feed/saved${q}`);
  },

  // ── Comments ──────────────────────────────────────────────────────────────

  getComments(postId: string, cursor?: string): Promise<ApiResponse<Comment[]>> {
    const q = cursor ? `?cursor=${cursor}` : "";
    return api.get<ApiResponse<Comment[]>>(
      `/api/uzume/feed/posts/${postId}/comments${q}`,
    );
  },

  createComment(postId: string, req: CreateCommentRequest): Promise<Comment> {
    return api.post<Comment>(
      `/api/uzume/feed/posts/${postId}/comments`,
      req,
    );
  },

  deleteComment(postId: string, commentId: string): Promise<void> {
    return api.delete<void>(
      `/api/uzume/feed/posts/${postId}/comments/${commentId}`,
    );
  },

  likeComment(postId: string, commentId: string): Promise<void> {
    return api.post<void>(
      `/api/uzume/feed/posts/${postId}/comments/${commentId}/like`,
    );
  },
};
