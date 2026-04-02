/**
 * Reels — full-screen vertical feed (TikTok-style).
 */
import React, { useState, useCallback, useRef } from "react";
import {
  View,
  Text,
  Pressable,
  useWindowDimensions,
  ActivityIndicator,
} from "react-native";
import { Image } from "expo-image";
import { FlashList } from "@shopify/flash-list";
import { Link, router } from "expo-router";
import { reelsApi, type Reel } from "@nyx/api";
import { Avatar } from "@nyx/ui";
import {
  HeartIcon,
  HeartFilledIcon,
  CommentIcon,
  ShareIcon,
  BookmarkIcon,
  BookmarkFilledIcon,
  PlayIcon,
  MusicIcon,
  VerifiedIcon,
} from "@nyx/ui";

// ─── Utilities ────────────────────────────────────────────────────────────────

function formatCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

// ─── Reel Item ────────────────────────────────────────────────────────────────

interface ReelItemProps {
  reel: Reel;
  height: number;
  width: number;
  visible: boolean;
}

function ReelItem({ reel, height, width, visible }: ReelItemProps) {
  const [liked, setLiked] = useState(reel.has_liked ?? false);
  const [likesCount, setLikesCount] = useState(reel.likes_count);
  const [saved, setSaved] = useState(reel.has_saved ?? false);
  const [following, setFollowing] = useState(reel.author.is_following ?? false);

  const handleLike = async () => {
    const next = !liked;
    setLiked(next);
    setLikesCount((c) => c + (next ? 1 : -1));
    try {
      if (next) {
        const res = await reelsApi.likeReel(reel.id);
        setLikesCount(res.likes_count);
      } else {
        const res = await reelsApi.unlikeReel(reel.id);
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
      if (next) await reelsApi.saveReel(reel.id);
      else await reelsApi.unsaveReel(reel.id);
    } catch {
      setSaved(!next);
    }
  };

  const handleFollow = async () => {
    const next = !following;
    setFollowing(next);
    try {
      const { profilesApi } = await import("@nyx/api");
      if (next) await profilesApi.follow(reel.author.alias);
      else await profilesApi.unfollow(reel.author.alias);
    } catch {
      setFollowing(!next);
    }
  };

  return (
    <View style={{ width, height }} className="relative bg-space-950">
      {/* Thumbnail */}
      <Image
        source={{ uri: reel.thumbnail_url ?? reel.video_url }}
        style={{ width, height }}
        contentFit="cover"
        transition={300}
        placeholder={{ blurhash: "L6PZfSi_.AyE_3t7t7R**0o#DgR4" }}
      />

      {/* Dark gradient overlay — bottom */}
      <View
        className="absolute inset-0"
        style={{
          background:
            "linear-gradient(to top, rgba(6,4,18,0.92) 0%, rgba(6,4,18,0.4) 40%, transparent 70%)",
        } as never}
        pointerEvents="none"
      />

      {/* Play icon overlay */}
      <View className="absolute inset-0 items-center justify-center" pointerEvents="none">
        <View
          className="w-16 h-16 rounded-full items-center justify-center"
          style={{ background: "rgba(6,4,18,0.5)" } as never}
        >
          <PlayIcon size={32} color="#F0EBF8" />
        </View>
      </View>

      {/* Right-side action buttons */}
      <View className="absolute right-3 items-center gap-5" style={{ bottom: 100 }}>
        {/* Author avatar + follow */}
        <View className="items-center gap-1">
          <Pressable
            onPress={() => router.push(`/(main)/profile/${reel.author.alias}` as never)}
            className="cursor-pointer"
          >
            <Avatar uri={reel.author.avatar_url} alias={reel.author.alias} size="lg" />
          </Pressable>
          {!following && (
            <Pressable
              onPress={handleFollow}
              className="w-6 h-6 rounded-full items-center justify-center -mt-3 border-2 border-space-900"
              style={{
                background:
                  "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 100%)",
              } as never}
            >
              <Text className="text-space-900 text-base font-bold leading-none">+</Text>
            </Pressable>
          )}
        </View>

        {/* Like */}
        <View className="items-center gap-1">
          <Pressable
            onPress={handleLike}
            className="cursor-pointer active:scale-110"
          >
            {liked ? (
              <HeartFilledIcon size={30} />
            ) : (
              <HeartIcon size={30} color="#F0EBF8" />
            )}
          </Pressable>
          <Text className="text-star-100 text-xs font-semibold">
            {formatCount(likesCount)}
          </Text>
        </View>

        {/* Comment */}
        <View className="items-center gap-1">
          <Pressable
            onPress={() => router.push(`/(main)/reel/${reel.id}` as never)}
            className="cursor-pointer"
          >
            <CommentIcon size={30} color="#F0EBF8" />
          </Pressable>
          <Text className="text-star-100 text-xs font-semibold">
            {formatCount(reel.comments_count)}
          </Text>
        </View>

        {/* Save */}
        <View className="items-center gap-1">
          <Pressable onPress={handleSave} className="cursor-pointer">
            {saved ? (
              <BookmarkFilledIcon size={30} />
            ) : (
              <BookmarkIcon size={30} color="#F0EBF8" />
            )}
          </Pressable>
          <Text className="text-star-100 text-xs font-semibold">
            {formatCount(reel.saves_count)}
          </Text>
        </View>

        {/* Share */}
        <View className="items-center gap-1">
          <Pressable className="cursor-pointer">
            <ShareIcon size={30} color="#F0EBF8" />
          </Pressable>
          <Text className="text-star-100 text-xs font-semibold">Share</Text>
        </View>
      </View>

      {/* Bottom info */}
      <View className="absolute left-3 right-16" style={{ bottom: 32 }}>
        {/* Author */}
        <View className="flex-row items-center gap-2 mb-2">
          <Pressable
            onPress={() => router.push(`/(main)/profile/${reel.author.alias}` as never)}
            className="cursor-pointer"
          >
            <Text className="text-star-100 font-bold text-sm">
              @{reel.author.alias}
            </Text>
          </Pressable>
          {reel.author.is_following !== undefined && (
            <VerifiedIcon size={13} />
          )}
        </View>

        {/* Caption */}
        {reel.caption && (
          <Text
            className="text-star-200 text-sm mb-2 leading-5"
            numberOfLines={2}
          >
            {reel.caption}
          </Text>
        )}

        {/* Tags */}
        {reel.tags.length > 0 && (
          <Text className="text-dawn-400 text-sm mb-2" numberOfLines={1}>
            {reel.tags.slice(0, 5).map((t) => `#${t}`).join(" ")}
          </Text>
        )}

        {/* Audio */}
        {(reel.audio_title || reel.audio_artist) && (
          <View className="flex-row items-center gap-2">
            <MusicIcon size={14} color="#C4B5E8" />
            <Text className="text-star-300 text-xs" numberOfLines={1}>
              {[reel.audio_title, reel.audio_artist].filter(Boolean).join(" · ")}
            </Text>
          </View>
        )}
      </View>
    </View>
  );
}

// ─── Screen ───────────────────────────────────────────────────────────────────

export default function ReelsScreen() {
  const { width, height } = useWindowDimensions();
  const [reels, setReels] = useState<Reel[]>([]);
  const [loading, setLoading] = useState(true);
  const [cursor, setCursor] = useState<string | undefined>(undefined);
  const [hasMore, setHasMore] = useState(true);
  const [fetchingMore, setFetchingMore] = useState(false);
  const [visibleIndex, setVisibleIndex] = useState(0);

  const reelHeight = height;

  const fetchFeed = useCallback(async (reset = false) => {
    try {
      const res = await reelsApi.getFeed(reset ? undefined : cursor);
      const newReels = res.data ?? [];
      if (reset) setReels(newReels);
      else setReels((prev) => [...prev, ...newReels]);
      setCursor(res.pagination?.next_cursor ?? undefined);
      setHasMore(!!res.pagination?.has_more);
    } catch {}
  }, [cursor]);

  React.useEffect(() => {
    fetchFeed(true).finally(() => setLoading(false));
  }, []);

  const handleEndReached = useCallback(async () => {
    if (!hasMore || fetchingMore) return;
    setFetchingMore(true);
    await fetchFeed(false);
    setFetchingMore(false);
  }, [hasMore, fetchingMore, fetchFeed]);

  if (loading) {
    return (
      <View className="flex-1 bg-space-950 items-center justify-center">
        <ActivityIndicator color="#FF6B9D" size="large" />
      </View>
    );
  }

  if (reels.length === 0) {
    return (
      <View className="flex-1 bg-space-950 items-center justify-center px-8">
        <View
          className="w-20 h-20 rounded-full items-center justify-center mb-5"
          style={{
            background: "linear-gradient(135deg, #FF6B9D22 0%, #A78BFA22 100%)",
          } as never}
        >
          <PlayIcon size={36} color="#7C6FA0" />
        </View>
        <Text className="text-star-100 text-xl font-bold mb-2 text-center">
          No reels yet
        </Text>
        <Text className="text-nyx-text-muted text-sm text-center">
          Follow more creators to see their reels here.
        </Text>
      </View>
    );
  }

  return (
    <View className="flex-1 bg-space-950">
      <FlashList
        data={reels}
        keyExtractor={(item) => item.id}
        estimatedItemSize={reelHeight}
        pagingEnabled
        snapToInterval={reelHeight}
        snapToAlignment="start"
        decelerationRate="fast"
        showsVerticalScrollIndicator={false}
        renderItem={({ item, index }) => (
          <ReelItem
            reel={item}
            height={reelHeight}
            width={width}
            visible={index === visibleIndex}
          />
        )}
        onViewableItemsChanged={({ viewableItems }) => {
          if (viewableItems[0]) {
            setVisibleIndex(viewableItems[0].index ?? 0);
          }
        }}
        viewabilityConfig={{ itemVisiblePercentThreshold: 60 }}
        onEndReached={handleEndReached}
        onEndReachedThreshold={0.3}
        ListFooterComponent={
          fetchingMore ? (
            <View style={{ height: reelHeight }} className="items-center justify-center bg-space-950">
              <ActivityIndicator color="#FF6B9D" size="large" />
            </View>
          ) : null
        }
      />
    </View>
  );
}
