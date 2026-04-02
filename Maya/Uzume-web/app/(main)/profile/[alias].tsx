/**
 * Profile page — own profile or any user profile.
 * Route: /(main)/profile/[alias]  (alias "me" → own profile)
 */
import React, { useState, useCallback, useEffect } from "react";
import {
  View,
  Text,
  Pressable,
  ScrollView,
  useWindowDimensions,
  ActivityIndicator,
  RefreshControl,
} from "react-native";
import { Image } from "expo-image";
import { FlashList } from "@shopify/flash-list";
import { useLocalSearchParams, router, Link } from "expo-router";
import {
  profilesApi,
  feedApi,
  reelsApi,
  storiesApi,
  type UzumeProfile,
  type Post,
  type Reel,
  type Highlight,
} from "@nyx/api";
import { Avatar, ProfileSkeleton } from "@nyx/ui";
import {
  VerifiedIcon,
  GridIcon,
  ReelsIcon,
  BookmarkIcon,
  BookmarkFilledIcon,
  EditIcon,
  FollowIcon,
  UnfollowIcon,
  ChevronLeftIcon,
} from "@nyx/ui";
import { useAuth } from "../../../src/context/AuthContext";

// ─── Types ────────────────────────────────────────────────────────────────────

type TabId = "posts" | "reels" | "saved";

// ─── Utilities ────────────────────────────────────────────────────────────────

function formatCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

// ─── Highlights Row ───────────────────────────────────────────────────────────

function HighlightsRow({ highlights }: { highlights: Highlight[] }) {
  if (highlights.length === 0) return null;

  return (
    <View className="border-b border-space-700">
      <ScrollView
        horizontal
        showsHorizontalScrollIndicator={false}
        contentContainerStyle={{ paddingHorizontal: 16, paddingVertical: 12, gap: 16 }}
      >
        {highlights.map((h) => (
          <Pressable
            key={h.id}
            className="items-center gap-1.5 cursor-pointer"
            style={{ width: 68 }}
            onPress={() => router.push(`/(main)/highlight/${h.id}` as never)}
          >
            <View
              style={{
                padding: 2,
                borderRadius: 9999,
                background:
                  "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 50%, #FFD93D 100%)",
              } as never}
            >
              <View className="w-[60px] h-[60px] rounded-full overflow-hidden bg-space-700">
                {h.cover_url ? (
                  <Image
                    source={{ uri: h.cover_url }}
                    style={{ width: 60, height: 60 }}
                    contentFit="cover"
                    transition={200}
                  />
                ) : (
                  <View className="w-[60px] h-[60px] bg-space-600 items-center justify-center">
                    <BookmarkIcon size={24} color="#7C6FA0" />
                  </View>
                )}
              </View>
            </View>
            <Text
              className="text-star-300 text-xs text-center"
              numberOfLines={1}
              style={{ width: 64 }}
            >
              {h.title}
            </Text>
          </Pressable>
        ))}
      </ScrollView>
    </View>
  );
}

// ─── Stats Row ────────────────────────────────────────────────────────────────

interface StatsRowProps {
  profile: UzumeProfile;
  isOwn: boolean;
}

function StatsRow({ profile, isOwn }: StatsRowProps) {
  const navigate = (to: "followers" | "following") => {
    router.push(`/(main)/profile/${profile.alias}/${to}` as never);
  };

  return (
    <View className="flex-row border-t border-space-700 mt-2">
      {[
        { label: "Posts", value: profile.posts_count, onPress: undefined },
        { label: "Followers", value: profile.followers_count, onPress: () => navigate("followers") },
        { label: "Following", value: profile.following_count, onPress: () => navigate("following") },
      ].map((stat, i) => (
        <Pressable
          key={stat.label}
          onPress={stat.onPress}
          className={`flex-1 items-center py-3 cursor-pointer ${
            i < 2 ? "border-r border-space-700" : ""
          }`}
          disabled={!stat.onPress}
        >
          <Text className="text-star-100 font-bold text-lg">
            {formatCount(stat.value)}
          </Text>
          <Text className="text-nyx-text-muted text-xs">{stat.label}</Text>
        </Pressable>
      ))}
    </View>
  );
}

// ─── Post Grid Item ───────────────────────────────────────────────────────────

function PostGridItem({ post, size }: { post: Post; size: number }) {
  return (
    <Link href={`/(main)/post/${post.id}` as never} asChild>
      <Pressable style={{ width: size, height: size }} className="cursor-pointer">
        {post.media[0] ? (
          <Image
            source={{ uri: post.media[0].thumbnail_url ?? post.media[0].url }}
            style={{ width: size, height: size }}
            contentFit="cover"
            transition={200}
            placeholder={{ blurhash: "L6PZfSi_.AyE_3t7t7R**0o#DgR4" }}
          />
        ) : (
          <View
            style={{ width: size, height: size }}
            className="bg-space-700 items-center justify-center"
          >
            <Text className="text-nyx-text-muted text-xs">No image</Text>
          </View>
        )}
        {post.media.length > 1 && (
          <View className="absolute top-1.5 right-1.5 bg-space-900/70 rounded-sm px-1">
            <Text className="text-star-100 text-[9px] font-bold">+{post.media.length - 1}</Text>
          </View>
        )}
      </Pressable>
    </Link>
  );
}

// ─── Reel Grid Item ───────────────────────────────────────────────────────────

function ReelGridItem({ reel, size }: { reel: Reel; size: number }) {
  return (
    <Link href={`/(main)/reel/${reel.id}` as never} asChild>
      <Pressable style={{ width: size, height: size }} className="cursor-pointer">
        <Image
          source={{ uri: reel.thumbnail_url ?? reel.video_url }}
          style={{ width: size, height: size }}
          contentFit="cover"
          transition={200}
          placeholder={{ blurhash: "L6PZfSi_.AyE_3t7t7R**0o#DgR4" }}
        />
        <View
          className="absolute inset-0 items-center justify-center"
          style={{ background: "rgba(6,4,18,0.25)" } as never}
          pointerEvents="none"
        >
          <ReelsIcon size={20} color="rgba(240,235,248,0.8)" />
        </View>
      </Pressable>
    </Link>
  );
}

// ─── Posts Grid ───────────────────────────────────────────────────────────────

function PostsGrid({ posts, size }: { posts: Post[]; size: number }) {
  const gap = 1;
  const rows: Post[][] = [];
  for (let i = 0; i < posts.length; i += 3) rows.push(posts.slice(i, i + 3));

  return (
    <View>
      {rows.map((row, ri) => (
        <View key={ri} className="flex-row" style={{ gap }}>
          {row.map((p) => (
            <PostGridItem key={p.id} post={p} size={size} />
          ))}
          {row.length < 3 &&
            Array.from({ length: 3 - row.length }).map((_, i) => (
              <View key={`e${i}`} style={{ width: size, height: size }} />
            ))}
        </View>
      ))}
    </View>
  );
}

// ─── Reels Grid ───────────────────────────────────────────────────────────────

function ReelsGrid({ reels, size }: { reels: Reel[]; size: number }) {
  const gap = 1;
  const rows: Reel[][] = [];
  for (let i = 0; i < reels.length; i += 3) rows.push(reels.slice(i, i + 3));

  return (
    <View>
      {rows.map((row, ri) => (
        <View key={ri} className="flex-row" style={{ gap }}>
          {row.map((r) => (
            <ReelGridItem key={r.id} reel={r} size={size} />
          ))}
          {row.length < 3 &&
            Array.from({ length: 3 - row.length }).map((_, i) => (
              <View key={`e${i}`} style={{ width: size, height: size }} />
            ))}
        </View>
      ))}
    </View>
  );
}

// ─── Profile Header ───────────────────────────────────────────────────────────

interface ProfileHeaderProps {
  profile: UzumeProfile;
  isOwn: boolean;
  following: boolean;
  onFollowToggle: () => void;
  highlights: Highlight[];
  width: number;
}

function ProfileHeader({
  profile,
  isOwn,
  following,
  onFollowToggle,
  highlights,
  width,
}: ProfileHeaderProps) {
  const headerHeight = Math.min(width * 0.4, 180);

  return (
    <View>
      {/* Header image */}
      <View style={{ height: headerHeight }} className="bg-space-700 relative">
        {profile.header_url ? (
          <Image
            source={{ uri: profile.header_url }}
            style={{ width, height: headerHeight }}
            contentFit="cover"
            transition={300}
          />
        ) : (
          <View
            style={{ width, height: headerHeight }}
            style={{
              background:
                "linear-gradient(135deg, #1C1845 0%, #302A80 50%, #13103A 100%)",
            } as never}
          />
        )}
        {/* Subtle gradient fade to bg */}
        <View
          className="absolute bottom-0 left-0 right-0 h-16"
          style={{
            background:
              "linear-gradient(to bottom, transparent, #060412)",
          } as never}
          pointerEvents="none"
        />
      </View>

      {/* Avatar + action row */}
      <View className="px-4 flex-row items-end justify-between" style={{ marginTop: -36 }}>
        <View
          style={{
            padding: 3,
            borderRadius: 9999,
            background:
              "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 50%, #FFD93D 100%)",
          } as never}
        >
          <View className="w-[76px] h-[76px] rounded-full overflow-hidden bg-space-700 border-2 border-space-900">
            {profile.avatar_url ? (
              <Image
                source={{ uri: profile.avatar_url }}
                style={{ width: 76, height: 76 }}
                contentFit="cover"
                transition={200}
              />
            ) : (
              <View className="w-[76px] h-[76px] bg-space-600 items-center justify-center">
                <Text className="text-star-100 text-3xl font-bold">
                  {profile.display_name.charAt(0).toUpperCase()}
                </Text>
              </View>
            )}
          </View>
        </View>

        <View className="flex-row gap-2 pb-1">
          {isOwn ? (
            <Link href="/(main)/edit-profile" asChild>
              <Pressable className="h-9 px-5 rounded-xl bg-space-700 border border-space-500 flex-row items-center gap-2 cursor-pointer active:bg-space-600">
                <EditIcon size={15} color="#C4B5E8" />
                <Text className="text-star-200 text-sm font-medium">Edit Profile</Text>
              </Pressable>
            </Link>
          ) : (
            <Pressable
              onPress={onFollowToggle}
              className="h-9 px-5 rounded-xl items-center justify-center cursor-pointer"
              style={
                following
                  ? undefined
                  : {
                      background:
                        "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 100%)",
                    }
              }
              className={`h-9 px-5 rounded-xl items-center justify-center cursor-pointer ${
                following ? "bg-space-700 border border-space-500" : ""
              }`}
            >
              <Text
                className={`text-sm font-semibold ${
                  following ? "text-star-300" : "text-space-900"
                }`}
              >
                {following ? "Following" : "Follow"}
              </Text>
            </Pressable>
          )}
        </View>
      </View>

      {/* Bio section */}
      <View className="px-4 mt-3">
        <View className="flex-row items-center gap-1.5 mb-0.5">
          <Text className="text-star-100 font-bold text-lg">
            {profile.display_name}
          </Text>
          {profile.is_verified && <VerifiedIcon size={16} />}
        </View>
        <Text className="text-nyx-text-muted text-sm mb-1">@{profile.alias}</Text>

        {profile.bio && (
          <Text className="text-star-200 text-sm leading-5 mb-2">{profile.bio}</Text>
        )}

        <View className="flex-row flex-wrap gap-x-4 gap-y-1">
          {profile.location && (
            <Text className="text-nyx-text-muted text-xs">📍 {profile.location}</Text>
          )}
          {profile.website && (
            <Text className="text-dawn-400 text-xs font-medium">{profile.website}</Text>
          )}
        </View>
      </View>

      <StatsRow profile={profile} isOwn={isOwn} />
      <HighlightsRow highlights={highlights} />
    </View>
  );
}

// ─── Tab Bar ──────────────────────────────────────────────────────────────────

interface TabBarProps {
  active: TabId;
  onSelect: (t: TabId) => void;
  isOwn: boolean;
}

function ProfileTabBar({ active, onSelect, isOwn }: TabBarProps) {
  const tabs: { id: TabId; icon: React.ReactNode; activeIcon: React.ReactNode }[] = [
    {
      id: "posts",
      icon: <GridIcon size={22} color="#7C6FA0" />,
      activeIcon: <GridIcon size={22} color="#FF6B9D" />,
    },
    {
      id: "reels",
      icon: <ReelsIcon size={22} color="#7C6FA0" />,
      activeIcon: <ReelsIcon size={22} color="#FF6B9D" />,
    },
    ...(isOwn
      ? [
          {
            id: "saved" as TabId,
            icon: <BookmarkIcon size={22} color="#7C6FA0" />,
            activeIcon: <BookmarkFilledIcon size={22} color="#FF6B9D" />,
          },
        ]
      : []),
  ];

  return (
    <View className="flex-row border-b border-space-700 bg-space-900">
      {tabs.map((tab) => (
        <Pressable
          key={tab.id}
          onPress={() => onSelect(tab.id)}
          className={`flex-1 items-center py-3 cursor-pointer ${
            active === tab.id ? "border-b-2 border-dawn-400" : ""
          }`}
        >
          {active === tab.id ? tab.activeIcon : tab.icon}
        </Pressable>
      ))}
    </View>
  );
}

// ─── Screen ───────────────────────────────────────────────────────────────────

export default function ProfileScreen() {
  const { alias } = useLocalSearchParams<{ alias: string }>();
  const { profile: myProfile } = useAuth();
  const { width } = useWindowDimensions();

  const isOwn = alias === "me" || alias === myProfile?.alias;
  const resolvedAlias = isOwn ? (myProfile?.alias ?? "me") : alias;

  const [profile, setProfile] = useState<UzumeProfile | null>(null);
  const [highlights, setHighlights] = useState<Highlight[]>([]);
  const [posts, setPosts] = useState<Post[]>([]);
  const [profileReels, setProfileReels] = useState<Reel[]>([]);
  const [savedPosts, setSavedPosts] = useState<Post[]>([]);
  const [tab, setTab] = useState<TabId>("posts");
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [following, setFollowing] = useState(false);
  const [tabLoading, setTabLoading] = useState(false);

  const gridItemSize = Math.floor((Math.min(width, 600) - 2) / 3);

  const loadProfile = useCallback(async () => {
    try {
      const p = isOwn
        ? await profilesApi.getMyProfile()
        : await profilesApi.getProfile(resolvedAlias);
      setProfile(p);
      setFollowing(p.is_following ?? false);

      const [hl, feedRes] = await Promise.all([
        storiesApi.getHighlights(p.alias).catch(() => []),
        feedApi.getProfilePosts(p.alias),
      ]);
      setHighlights(hl);
      setPosts(feedRes.data ?? []);
    } catch {}
  }, [isOwn, resolvedAlias]);

  useEffect(() => {
    loadProfile().finally(() => setLoading(false));
  }, [loadProfile]);

  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadProfile();
    setRefreshing(false);
  }, [loadProfile]);

  const handleTabChange = useCallback(
    async (t: TabId) => {
      setTab(t);
      if (!profile) return;
      if (t === "reels" && profileReels.length === 0) {
        setTabLoading(true);
        try {
          const res = await reelsApi.getProfileReels(profile.alias);
          setProfileReels(res.data ?? []);
        } catch {}
        setTabLoading(false);
      }
      if (t === "saved" && savedPosts.length === 0 && isOwn) {
        setTabLoading(true);
        try {
          const res = await feedApi.getSavedPosts();
          setSavedPosts(res.data ?? []);
        } catch {}
        setTabLoading(false);
      }
    },
    [profile, profileReels.length, savedPosts.length, isOwn]
  );

  const handleFollowToggle = useCallback(async () => {
    if (!profile) return;
    const next = !following;
    setFollowing(next);
    try {
      if (next) await profilesApi.follow(profile.alias);
      else await profilesApi.unfollow(profile.alias);
    } catch {
      setFollowing(!next);
    }
  }, [following, profile]);

  if (loading || !profile) {
    return (
      <View className="flex-1 bg-space-900">
        <ProfileSkeleton />
      </View>
    );
  }

  const tabContent = () => {
    if (tabLoading) {
      return (
        <View className="py-16 items-center">
          <ActivityIndicator color="#FF6B9D" />
        </View>
      );
    }
    if (tab === "posts") {
      return posts.length > 0 ? (
        <PostsGrid posts={posts} size={gridItemSize} />
      ) : (
        <EmptyTab label="No posts yet" />
      );
    }
    if (tab === "reels") {
      return profileReels.length > 0 ? (
        <ReelsGrid reels={profileReels} size={gridItemSize} />
      ) : (
        <EmptyTab label="No reels yet" />
      );
    }
    if (tab === "saved") {
      return savedPosts.length > 0 ? (
        <PostsGrid posts={savedPosts} size={gridItemSize} />
      ) : (
        <EmptyTab label="No saved posts" />
      );
    }
    return null;
  };

  return (
    <View className="flex-1 bg-space-900">
      <ScrollView
        showsVerticalScrollIndicator={false}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={handleRefresh}
            tintColor="#FF6B9D"
            colors={["#FF6B9D"]}
          />
        }
      >
        <ProfileHeader
          profile={profile}
          isOwn={isOwn}
          following={following}
          onFollowToggle={handleFollowToggle}
          highlights={highlights}
          width={width}
        />
        <ProfileTabBar active={tab} onSelect={handleTabChange} isOwn={isOwn} />
        {tabContent()}
      </ScrollView>
    </View>
  );
}

function EmptyTab({ label }: { label: string }) {
  return (
    <View className="py-16 items-center justify-center">
      <Text className="text-nyx-text-muted text-sm">{label}</Text>
    </View>
  );
}
