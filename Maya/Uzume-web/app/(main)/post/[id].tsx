/**
 * Post detail — full-size media carousel, comments, actions.
 */
import React, { useState, useCallback, useEffect, useRef } from "react";
import {
  View,
  Text,
  Pressable,
  ScrollView,
  TextInput,
  KeyboardAvoidingView,
  Platform,
  ActivityIndicator,
  useWindowDimensions,
} from "react-native";
import { Image } from "expo-image";
import { FlashList } from "@shopify/flash-list";
import { useLocalSearchParams, router } from "expo-router";
import {
  feedApi,
  type Post,
  type Comment,
  type MediaAttachment,
} from "@nyx/api";
import { Avatar, Skeleton } from "@nyx/ui";
import {
  HeartIcon,
  HeartFilledIcon,
  CommentIcon,
  ShareIcon,
  BookmarkIcon,
  BookmarkFilledIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  SendIcon,
  MoreHorizIcon,
  VerifiedIcon,
} from "@nyx/ui";
import { useAuth } from "../../../src/context/AuthContext";

// ─── Utilities ────────────────────────────────────────────────────────────────

function timeAgo(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60_000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  const days = Math.floor(hrs / 24);
  if (days < 7) return `${days}d ago`;
  return new Date(iso).toLocaleDateString();
}

function formatCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

// ─── Media Carousel ───────────────────────────────────────────────────────────

function MediaCarousel({
  media,
  width,
}: {
  media: MediaAttachment[];
  width: number;
}) {
  const [index, setIndex] = useState(0);
  const imageSize = Math.min(width, 600);

  if (media.length === 0) return null;

  return (
    <View style={{ width: imageSize, height: imageSize }} className="relative">
      <Image
        source={{ uri: media[index].url }}
        style={{ width: imageSize, height: imageSize }}
        contentFit="cover"
        transition={250}
        placeholder={{ blurhash: "L6PZfSi_.AyE_3t7t7R**0o#DgR4" }}
        alt={media[index].alt_text}
      />

      {/* Prev/Next arrows */}
      {media.length > 1 && index > 0 && (
        <Pressable
          className="absolute left-2 top-1/2 -translate-y-1/2 w-8 h-8 rounded-full bg-space-900/70 items-center justify-center cursor-pointer"
          onPress={() => setIndex((i) => i - 1)}
        >
          <ChevronLeftIcon size={18} color="#F0EBF8" />
        </Pressable>
      )}
      {media.length > 1 && index < media.length - 1 && (
        <Pressable
          className="absolute right-2 top-1/2 -translate-y-1/2 w-8 h-8 rounded-full bg-space-900/70 items-center justify-center cursor-pointer"
          onPress={() => setIndex((i) => i + 1)}
        >
          <ChevronRightIcon size={18} color="#F0EBF8" />
        </Pressable>
      )}

      {/* Dot indicators */}
      {media.length > 1 && (
        <View className="absolute bottom-3 left-0 right-0 flex-row justify-center gap-1.5">
          {media.map((_, i) => (
            <View
              key={i}
              className={`rounded-full ${
                i === index
                  ? "w-4 h-1.5 bg-dawn-400"
                  : "w-1.5 h-1.5 bg-star-100/40"
              }`}
            />
          ))}
        </View>
      )}
    </View>
  );
}

// ─── Comment Row ──────────────────────────────────────────────────────────────

function CommentRow({ comment, postId }: { comment: Comment; postId: string }) {
  const [liked, setLiked] = useState(comment.has_liked ?? false);
  const [likesCount, setLikesCount] = useState(comment.likes_count);

  const handleLike = async () => {
    const next = !liked;
    setLiked(next);
    setLikesCount((c) => c + (next ? 1 : -1));
    try {
      await feedApi.likeComment(postId, comment.id);
    } catch {
      setLiked(!next);
      setLikesCount((c) => c + (next ? -1 : 1));
    }
  };

  return (
    <View className="flex-row px-4 py-3 gap-3">
      <Pressable
        onPress={() => router.push(`/(main)/profile/${comment.author.alias}` as never)}
        className="cursor-pointer"
      >
        <Avatar uri={comment.author.avatar_url} alias={comment.author.alias} size="sm" />
      </Pressable>
      <View className="flex-1">
        <View className="flex-row flex-wrap items-baseline">
          <Pressable
            onPress={() => router.push(`/(main)/profile/${comment.author.alias}` as never)}
            className="cursor-pointer mr-1"
          >
            <Text className="text-star-100 font-semibold text-sm">
              {comment.author.display_name}
            </Text>
          </Pressable>
          <Text className="text-star-200 text-sm leading-5">{comment.body}</Text>
        </View>
        <View className="flex-row items-center gap-4 mt-1">
          <Text className="text-nyx-text-muted text-xs">{timeAgo(comment.created_at)}</Text>
          {likesCount > 0 && (
            <Text className="text-nyx-text-muted text-xs">
              {formatCount(likesCount)} likes
            </Text>
          )}
        </View>
      </View>
      <Pressable onPress={handleLike} className="cursor-pointer pt-0.5">
        {liked ? (
          <HeartFilledIcon size={14} color="#FF6B9D" />
        ) : (
          <HeartIcon size={14} color="#7C6FA0" />
        )}
      </Pressable>
    </View>
  );
}

// ─── Post Actions ─────────────────────────────────────────────────────────────

interface PostActionsProps {
  post: Post;
}

function PostActions({ post }: PostActionsProps) {
  const [liked, setLiked] = useState(post.has_liked ?? false);
  const [likesCount, setLikesCount] = useState(post.likes_count);
  const [saved, setSaved] = useState(post.has_saved ?? false);

  const handleLike = async () => {
    const next = !liked;
    setLiked(next);
    setLikesCount((c) => c + (next ? 1 : -1));
    try {
      if (next) {
        const res = await feedApi.likePost(post.id);
        setLikesCount(res.likes_count);
      } else {
        const res = await feedApi.unlikePost(post.id);
        setLikesCount(res.likes_count);
      }
    } catch {
      setLiked(!next);
      setLikesCount((c) => c + (next ? -1 : 1));
    }
  };

  const handleSave = async () => {
    const next = !saved;
    setSaved(next);
    try {
      if (next) await feedApi.savePost(post.id);
      else await feedApi.unsavePost(post.id);
    } catch {
      setSaved(!next);
    }
  };

  return (
    <View>
      <View className="px-4 pt-3 pb-1 flex-row items-center">
        <View className="flex-1 flex-row gap-4">
          <Pressable onPress={handleLike} className="cursor-pointer">
            {liked ? <HeartFilledIcon size={26} /> : <HeartIcon size={26} />}
          </Pressable>
          <Pressable className="cursor-pointer">
            <CommentIcon size={26} />
          </Pressable>
          <Pressable className="cursor-pointer">
            <ShareIcon size={26} />
          </Pressable>
        </View>
        <Pressable onPress={handleSave} className="cursor-pointer">
          {saved ? <BookmarkFilledIcon size={26} /> : <BookmarkIcon size={26} />}
        </Pressable>
      </View>
      {likesCount > 0 && (
        <View className="px-4 pb-1">
          <Text className="text-star-100 font-bold text-sm">
            {formatCount(likesCount)} {likesCount === 1 ? "like" : "likes"}
          </Text>
        </View>
      )}
      {post.caption && (
        <View className="px-4 pb-1 flex-row flex-wrap">
          <Pressable
            onPress={() => router.push(`/(main)/profile/${post.author.alias}` as never)}
            className="cursor-pointer mr-1"
          >
            <Text className="text-star-100 font-bold text-sm">
              {post.author.display_name}
            </Text>
          </Pressable>
          <Text className="text-star-200 text-sm flex-1">{post.caption}</Text>
        </View>
      )}
      <View className="px-4 pb-3">
        <Text className="text-nyx-text-muted text-xs">{timeAgo(post.created_at)}</Text>
      </View>
    </View>
  );
}

// ─── Comment Input ────────────────────────────────────────────────────────────

interface CommentInputProps {
  postId: string;
  profile: { avatar_url?: string; display_name: string; alias?: string } | null;
  onCommentPosted: (comment: Comment) => void;
}

function CommentInput({ postId, profile, onCommentPosted }: CommentInputProps) {
  const [text, setText] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async () => {
    const body = text.trim();
    if (!body || submitting) return;
    setSubmitting(true);
    try {
      const comment = await feedApi.createComment(postId, { body });
      onCommentPosted(comment);
      setText("");
    } catch {}
    setSubmitting(false);
  };

  return (
    <View className="flex-row items-center gap-3 px-4 py-3 border-t border-space-700 bg-space-800">
      <Avatar uri={profile?.avatar_url} alias={profile?.alias ?? profile?.display_name} size="sm" />
      <View className="flex-1 bg-space-700 rounded-2xl px-4 h-9 justify-center border border-space-600">
        <TextInput
          value={text}
          onChangeText={setText}
          placeholder="Add a comment…"
          placeholderTextColor="#7C6FA0"
          className="text-star-100 text-sm bg-transparent outline-none"
          style={{ outline: "none" } as never}
          multiline={false}
          returnKeyType="send"
          onSubmitEditing={handleSubmit}
        />
      </View>
      <Pressable
        onPress={handleSubmit}
        disabled={!text.trim() || submitting}
        className="cursor-pointer disabled:opacity-40"
      >
        {submitting ? (
          <ActivityIndicator size="small" color="#FF6B9D" />
        ) : (
          <SendIcon size={22} color={text.trim() ? "#FF6B9D" : "#7C6FA0"} />
        )}
      </Pressable>
    </View>
  );
}

// ─── Screen ───────────────────────────────────────────────────────────────────

export default function PostDetailScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const { profile: myProfile } = useAuth();
  const { width } = useWindowDimensions();

  const [post, setPost] = useState<Post | null>(null);
  const [comments, setComments] = useState<Comment[]>([]);
  const [loadingPost, setLoadingPost] = useState(true);
  const [loadingComments, setLoadingComments] = useState(true);
  const [commentsError, setCommentsError] = useState(false);

  useEffect(() => {
    const load = async () => {
      try {
        const [p, commentsRes] = await Promise.all([
          feedApi.getPost(id),
          feedApi.getComments(id),
        ]);
        setPost(p);
        setComments(commentsRes.data ?? []);
      } catch {
        setCommentsError(true);
      }
      setLoadingPost(false);
      setLoadingComments(false);
    };
    load();
  }, [id]);

  const handleCommentPosted = useCallback((comment: Comment) => {
    setComments((prev) => [comment, ...prev]);
  }, []);

  if (loadingPost) {
    return (
      <View className="flex-1 bg-space-900">
        {/* Header skeleton */}
        <View className="flex-row items-center gap-3 px-4 py-3">
          <Skeleton width={40} height={40} rounded="full" />
          <View className="flex-1 gap-2">
            <Skeleton width={120} height={14} />
            <Skeleton width={80} height={12} />
          </View>
        </View>
        <Skeleton width="100%" height={Math.min(width, 600)} rounded="none" />
      </View>
    );
  }

  if (!post) {
    return (
      <View className="flex-1 bg-space-900 items-center justify-center px-8">
        <Text className="text-star-100 text-lg font-bold mb-2">Post not found</Text>
        <Text className="text-nyx-text-muted text-sm text-center mb-6">
          This post may have been deleted or is unavailable.
        </Text>
        <Pressable onPress={() => router.back()} className="cursor-pointer">
          <Text className="text-dawn-400 font-medium">Go back</Text>
        </Pressable>
      </View>
    );
  }

  return (
    <KeyboardAvoidingView
      className="flex-1 bg-space-900"
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      keyboardVerticalOffset={80}
    >
      {/* Back button */}
      <Pressable
        onPress={() => router.back()}
        className="absolute top-4 left-4 z-10 w-9 h-9 rounded-full bg-space-800/80 border border-space-600 items-center justify-center cursor-pointer"
      >
        <ChevronLeftIcon size={20} />
      </Pressable>

      <ScrollView showsVerticalScrollIndicator={false}>
        {/* Post author header */}
        <View className="flex-row items-center px-4 py-3 gap-3 mt-2">
          <Pressable
            onPress={() => router.push(`/(main)/profile/${post.author.alias}` as never)}
            className="cursor-pointer"
          >
            <Avatar uri={post.author.avatar_url} alias={post.author.alias} size="md" />
          </Pressable>
          <View className="flex-1">
            <View className="flex-row items-center gap-1">
              <Pressable
                onPress={() => router.push(`/(main)/profile/${post.author.alias}` as never)}
                className="cursor-pointer"
              >
                <Text className="text-star-100 font-semibold text-sm">
                  {post.author.display_name}
                </Text>
              </Pressable>
              {post.author.is_verified && <VerifiedIcon size={14} />}
            </View>
            <Text className="text-nyx-text-muted text-xs">@{post.author.alias}</Text>
          </View>
          <Pressable className="p-2 cursor-pointer rounded-lg">
            <MoreHorizIcon size={20} color="#7C6FA0" />
          </Pressable>
        </View>

        {/* Media */}
        <MediaCarousel media={post.media} width={width} />

        {/* Actions + caption */}
        <PostActions post={post} />

        {/* Divider */}
        <View className="h-px bg-space-700 mx-0" />

        {/* Comments header */}
        <View className="flex-row items-center justify-between px-4 py-3">
          <Text className="text-star-300 font-semibold text-sm">
            {comments.length > 0
              ? `${formatCount(post.comments_count)} comments`
              : "No comments yet"}
          </Text>
        </View>

        {/* Comments list */}
        {loadingComments ? (
          <View className="gap-0">
            {[0, 1, 2].map((i) => (
              <View key={i} className="flex-row px-4 py-3 gap-3">
                <Skeleton width={32} height={32} rounded="full" />
                <View className="flex-1 gap-1.5">
                  <Skeleton width={100} height={13} />
                  <Skeleton width="85%" height={13} />
                </View>
              </View>
            ))}
          </View>
        ) : commentsError ? (
          <View className="px-4 py-6 items-center">
            <Text className="text-nyx-text-muted text-sm">
              Failed to load comments.
            </Text>
          </View>
        ) : (
          <View>
            {comments.map((c) => (
              <CommentRow key={c.id} comment={c} postId={post.id} />
            ))}
            <View className="h-6" />
          </View>
        )}
      </ScrollView>

      {/* Comment input */}
      <CommentInput
        postId={post.id}
        profile={myProfile}
        onCommentPosted={handleCommentPosted}
      />
    </KeyboardAvoidingView>
  );
}
