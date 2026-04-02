/**
 * Home Feed — stories bar + infinite-scroll post timeline.
 */
import React, { useState, useCallback, useRef } from "react";
import {
  View,
  Text,
  Pressable,
  ScrollView,
  RefreshControl,
  ActivityIndicator,
  useWindowDimensions,
} from "react-native";
import { Image } from "expo-image";
import { FlashList } from "@shopify/flash-list";
import { Link, router } from "expo-router";
import {
  feedApi,
  storiesApi,
  type Post,
  type StoryGroup,
} from "@nyx/api";
import { Avatar, PostSkeleton } from "@nyx/ui";
import {
  HeartIcon,
  HeartFilledIcon,
  CommentIcon,
  ShareIcon,
  BookmarkIcon,
  BookmarkFilledIcon,
  MoreHorizIcon,
  VerifiedIcon,
  UzumeLogoIcon,
} from "@nyx/ui";
import { useAuth } from "../../src/context/AuthContext";

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
  const weeks = Math.floor(days / 7);
  if (weeks < 5) return `${weeks}w ago`;
  return new Date(iso).toLocaleDateString();
}

function formatCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

// ─── Story Circle ─────────────────────────────────────────────────────────────

function StoryCircle({ group }: { group: StoryGroup }) {
  return (
    <Pressable
      className="items-center gap-1.5 cursor-pointer"
      style={{ width: 72 }}
      onPress={() => router.push(`/(main)/story/${group.alias}` as never)}
    >
      <View
        style={
          group.has_unseen
            ? {
                padding: 2,
                borderRadius: 9999,
                background:
                  "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 50%, #FFD93D 100%)",
              }
            : {
                padding: 2,
                borderRadius: 9999,
                borderWidth: 2,
                borderColor: "#2A2460",
              }
        }
      >
        <View className="w-[60px] h-[60px] rounded-full overflow-hidden bg-space-700">
          {group.avatar_url ? (
            <Image
              source={{ uri: group.avatar_url }}
              style={{ width: 60, height: 60 }}
              contentFit="cover"
              transition={200}
            />
          ) : (
            <View className="w-[60px] h-[60px] rounded-full bg-space-600 items-center justify-center">
              <Text className="text-star-300 text-xl font-semibold">
                {group.display_name.charAt(0).toUpperCase()}
              </Text>
            </View>
          )}
        </View>
      </View>
      <Text
        className="text-star-300 text-xs text-center"
        numberOfLines={1}
        style={{ width: 68 }}
      >
        {group.display_name}
      </Text>
    </Pressable>
  );
}

// ─── Add Story Button ─────────────────────────────────────────────────────────

function AddStoryButton({ profile }: { profile: { avatar_url?: string; display_name: string } }) {
  return (
    <Pressable
      className="items-center gap-1.5 cursor-pointer"
      style={{ width: 72 }}
      onPress={() => router.push("/(main)/new-story" as never)}
    >
      <View className="relative">
        <View className="w-[60px] h-[60px] rounded-full bg-space-700 border-2 border-space-500 overflow-hidden items-center justify-center">
          {profile.avatar_url ? (
            <Image
              source={{ uri: profile.avatar_url }}
              style={{ width: 60, height: 60 }}
              contentFit="cover"
              transition={200}
            />
          ) : (
            <Text className="text-star-300 text-xl font-semibold">
              {profile.display_name.charAt(0).toUpperCase()}
            </Text>
          )}
        </View>
        <View
          className="absolute bottom-0 right-0 w-5 h-5 rounded-full border-2 border-space-900 items-center justify-center"
          style={{
            background:
              "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 50%, #FFD93D 100%)",
          } as never}
        >
          <Text className="text-space-900 text-xs font-bold leading-none">+</Text>
        </View>
      </View>
      <Text className="text-star-300 text-xs text-center" numberOfLines={1}>
        Your Story
      </Text>
    </Pressable>
  );
}

// ─── Stories Bar ──────────────────────────────────────────────────────────────

function StoriesBar({ profile }: { profile: { avatar_url?: string; display_name: string } | null }) {
  const [groups, setGroups] = useState<StoryGroup[]>([]);

  React.useEffect(() => {
    storiesApi
      .getFeed()
      .then((res) => setGroups(res.data ?? []))
      .catch(() => {});
  }, []);

  return (
    <View className="bg-space-800 border-b border-space-700">
      <ScrollView
        horizontal
        showsHorizontalScrollIndicator={false}
        contentContainerStyle={{ paddingHorizontal: 12, paddingVertical: 12, gap: 8 }}
      >
        {profile && <AddStoryButton profile={profile} />}
        {groups.map((g) => (
          <StoryCircle key={g.alias} group={g} />
        ))}
      </ScrollView>
    </View>
  );
}

// ─── Post Card ────────────────────────────────────────────────────────────────

interface PostCardProps {
  post: Post;
  onLikeToggle: (id: string, liked: boolean) => void;
  onSaveToggle: (id: string, saved: boolean) => void;
}

function PostCard({ post, onLikeToggle, onSaveToggle }: PostCardProps) {
  const [liked, setLiked] = useState(post.has_liked ?? false);
  const [likesCount, setLikesCount] = useState(post.likes_count);
  const [saved, setSaved] = useState(post.has_saved ?? false);
  const [menuOpen, setMenuOpen] = useState(false);
  const { width } = useWindowDimensions();

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
      onLikeToggle(post.id, next);
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
      onSaveToggle(post.id, next);
    } catch {
      setSaved(!next);
    }
  };

  const imageWidth = Math.min(width, 600);

  return (
    <View className="bg-space-900 border-b border-space-700 mb-0">
      {/* Header */}
      <View className="flex-row items-center px-4 py-3">
        <Link href={`/(main)/profile/${post.author.alias}` as never} asChild>
          <Pressable className="cursor-pointer">
            <Avatar uri={post.author.avatar_url} alias={post.author.alias} size="md" />
          </Pressable>
        </Link>
        <View className="flex-1 ml-3">
          <View className="flex-row items-center gap-1">
            <Link href={`/(main)/profile/${post.author.alias}` as never} asChild>
              <Pressable className="cursor-pointer">
                <Text className="text-star-100 font-semibold text-sm">
                  {post.author.display_name}
                </Text>
              </Pressable>
            </Link>
            {post.author.is_verified && (
              <VerifiedIcon size={14} />
            )}
          </View>
          <Text className="text-nyx-text-muted text-xs">@{post.author.alias}</Text>
        </View>
        <Pressable
          onPress={() => setMenuOpen((v) => !v)}
          className="p-2 cursor-pointer rounded-lg active:bg-space-700"
        >
          <MoreHorizIcon size={20} color="#7C6FA0" />
        </Pressable>
      </View>

      {/* Media */}
      {post.media.length > 0 && (
        <View style={{ width: imageWidth, height: imageWidth }}>
          <Image
            source={{ uri: post.media[0].url }}
            style={{ width: imageWidth, height: imageWidth }}
            contentFit="cover"
            transition={300}
            placeholder={{ blurhash: "L6PZfSi_.AyE_3t7t7R**0o#DgR4" }}
          />
          {post.media.length > 1 && (
            <View className="absolute top-3 right-3 bg-space-900/80 rounded-full px-2 py-0.5">
              <Text className="text-star-100 text-xs font-medium">
                1/{post.media.length}
              </Text>
            </View>
          )}
        </View>
      )}

      {/* Actions */}
      <View className="px-4 pt-3 pb-1 flex-row items-center">
        <View className="flex-1 flex-row gap-4">
          <Pressable onPress={handleLike} className="cursor-pointer active:scale-110 flex-row items-center gap-1.5">
            {liked ? <HeartFilledIcon size={24} /> : <HeartIcon size={24} />}
          </Pressable>
          <Link href={`/(main)/post/${post.id}` as never} asChild>
            <Pressable className="cursor-pointer flex-row items-center gap-1.5">
              <CommentIcon size={24} />
            </Pressable>
          </Link>
          <Pressable className="cursor-pointer">
            <ShareIcon size={24} />
          </Pressable>
        </View>
        <Pressable onPress={handleSave} className="cursor-pointer">
          {saved ? <BookmarkFilledIcon size={24} /> : <BookmarkIcon size={24} />}
        </Pressable>
      </View>

      {/* Like count */}
      {likesCount > 0 && (
        <View className="px-4 pb-1">
          <Text className="text-star-100 font-semibold text-sm">
            {formatCount(likesCount)} {likesCount === 1 ? "like" : "likes"}
          </Text>
        </View>
      )}

      {/* Caption */}
      {post.caption && (
        <View className="px-4 pb-1 flex-row flex-wrap">
          <Text className="text-star-100 font-semibold text-sm mr-1">
            {post.author.display_name}
          </Text>
          <Text className="text-star-200 text-sm flex-1">{post.caption}</Text>
        </View>
      )}

      {/* Timestamp */}
      <View className="px-4 pb-4 pt-1">
        <Text className="text-nyx-text-muted text-xs">{timeAgo(post.created_at)}</Text>
      </View>
    </View>
  );
}

// ─── Empty State ──────────────────────────────────────────────────────────────

function EmptyFeed() {
  return (
    <View className="flex-1 items-center justify-center py-24 px-8">
      <View
        className="w-24 h-24 rounded-full items-center justify-center mb-6"
        style={{
          background:
            "linear-gradient(135deg, #FF6B9D22 0%, #A78BFA22 100%)",
        } as never}
      >
        <UzumeLogoIcon size={40} />
      </View>
      <Text className="text-star-100 text-xl font-bold mb-2 text-center">
        Your feed is empty
      </Text>
      <Text className="text-nyx-text-muted text-sm text-center leading-relaxed">
        Follow people to see their posts here.{"\n"}Discover new creators on the Explore page.
      </Text>
      <Link href="/(main)/explore" asChild>
        <Pressable
          className="mt-6 h-11 px-8 rounded-2xl items-center justify-center"
          style={{
            background:
              "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 60%, #FFD93D 100%)",
          } as never}
        >
          <Text className="text-space-900 font-bold text-sm">Explore Creators</Text>
        </Pressable>
      </Link>
    </View>
  );
}

// ─── Screen ───────────────────────────────────────────────────────────────────

export default function HomeScreen() {
  const { profile } = useAuth();
  const [posts, setPosts] = useState<Post[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [cursor, setCursor] = useState<string | undefined>(undefined);
  const [hasMore, setHasMore] = useState(true);
  const [fetchingMore, setFetchingMore] = useState(false);

  const fetchTimeline = useCallback(async (reset = false) => {
    try {
      const res = await feedApi.getHomeTimeline(reset ? undefined : cursor);
      const newPosts = res.data ?? [];
      if (reset) {
        setPosts(newPosts);
      } else {
        setPosts((prev) => [...prev, ...newPosts]);
      }
      setCursor(res.pagination?.next_cursor ?? undefined);
      setHasMore(!!res.pagination?.has_more);
    } catch {
      // network error — keep current posts
    }
  }, [cursor]);

  React.useEffect(() => {
    fetchTimeline(true).finally(() => setLoading(false));
  }, []);

  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    setCursor(undefined);
    setHasMore(true);
    await fetchTimeline(true);
    setRefreshing(false);
  }, []);

  const handleEndReached = useCallback(async () => {
    if (!hasMore || fetchingMore) return;
    setFetchingMore(true);
    await fetchTimeline(false);
    setFetchingMore(false);
  }, [hasMore, fetchingMore, fetchTimeline]);

  const handleLikeToggle = useCallback((id: string, liked: boolean) => {
    // optimistic update already done in PostCard; nothing to sync here
  }, []);

  const handleSaveToggle = useCallback((id: string, saved: boolean) => {}, []);

  if (loading) {
    return (
      <View className="flex-1 bg-space-900">
        <View className="bg-space-800 border-b border-space-700 h-[88px]" />
        {[0, 1, 2].map((i) => (
          <PostSkeleton key={i} />
        ))}
      </View>
    );
  }

  return (
    <View className="flex-1 bg-space-900">
      <FlashList
        data={posts}
        keyExtractor={(item) => item.id}
        estimatedItemSize={480}
        renderItem={({ item }) => (
          <PostCard
            post={item}
            onLikeToggle={handleLikeToggle}
            onSaveToggle={handleSaveToggle}
          />
        )}
        ListHeaderComponent={
          <StoriesBar profile={profile} />
        }
        ListEmptyComponent={<EmptyFeed />}
        ListFooterComponent={
          fetchingMore ? (
            <View className="py-8 items-center">
              <ActivityIndicator color="#FF6B9D" />
            </View>
          ) : null
        }
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={handleRefresh}
            tintColor="#FF6B9D"
            colors={["#FF6B9D"]}
          />
        }
        onEndReached={handleEndReached}
        onEndReachedThreshold={0.4}
        showsVerticalScrollIndicator={false}
      />
    </View>
  );
}
