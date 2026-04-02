/**
 * Explore / Discover — search, trending tags, grid of posts & people.
 */
import React, { useState, useCallback, useEffect, useRef } from "react";
import {
  View,
  Text,
  Pressable,
  TextInput,
  ScrollView,
  ActivityIndicator,
  useWindowDimensions,
} from "react-native";
import { Image } from "expo-image";
import { FlashList } from "@shopify/flash-list";
import { Link, router } from "expo-router";
import {
  discoverApi,
  type TrendingTag,
  type UzumeProfile,
  type Post,
} from "@nyx/api";
import { Avatar, Skeleton } from "@nyx/ui";
import {
  ExploreIcon,
  CloseIcon,
  VerifiedIcon,
  FollowIcon,
} from "@nyx/ui";

// ─── Types ────────────────────────────────────────────────────────────────────

type TabId = "posts" | "people" | "tags";

// ─── Utilities ────────────────────────────────────────────────────────────────

function formatCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

// ─── Search Bar ───────────────────────────────────────────────────────────────

interface SearchBarProps {
  value: string;
  onChange: (v: string) => void;
  onClear: () => void;
}

function SearchBar({ value, onChange, onClear }: SearchBarProps) {
  return (
    <View className="flex-row items-center bg-space-700 border border-space-500 rounded-2xl px-4 h-11 gap-3 mx-4 my-3">
      <ExploreIcon size={18} color="#7C6FA0" />
      <TextInput
        value={value}
        onChangeText={onChange}
        placeholder="Search posts, people, tags…"
        placeholderTextColor="#7C6FA0"
        className="flex-1 text-star-100 text-sm bg-transparent outline-none"
        style={{ outline: "none" } as never}
        autoCapitalize="none"
        autoCorrect={false}
        returnKeyType="search"
      />
      {value.length > 0 && (
        <Pressable onPress={onClear} className="cursor-pointer">
          <CloseIcon size={16} color="#7C6FA0" />
        </Pressable>
      )}
    </View>
  );
}

// ─── Trending Tags ────────────────────────────────────────────────────────────

function TrendingTagsRow({ tags }: { tags: TrendingTag[] }) {
  if (tags.length === 0) return null;
  return (
    <View className="mb-4">
      <Text className="text-star-300 text-xs font-semibold uppercase tracking-wider px-4 mb-2">
        Trending
      </Text>
      <ScrollView
        horizontal
        showsHorizontalScrollIndicator={false}
        contentContainerStyle={{ paddingHorizontal: 16, gap: 8 }}
      >
        {tags.slice(0, 15).map((tag) => (
          <Pressable
            key={tag.tag}
            onPress={() => router.push(`/(main)/tag/${tag.tag}` as never)}
            className="cursor-pointer"
          >
            <View
              className="px-4 py-2 rounded-full border border-space-500 bg-space-700"
            >
              <Text className="text-dawn-400 font-semibold text-sm">
                #{tag.tag}
              </Text>
              <Text className="text-nyx-text-muted text-xs text-center">
                {formatCount(tag.posts_count)}
              </Text>
            </View>
          </Pressable>
        ))}
      </ScrollView>
    </View>
  );
}

// ─── Tab Bar ──────────────────────────────────────────────────────────────────

interface TabBarProps {
  active: TabId;
  onSelect: (id: TabId) => void;
}

const TABS: { id: TabId; label: string }[] = [
  { id: "posts", label: "Posts" },
  { id: "people", label: "People" },
  { id: "tags", label: "Tags" },
];

function TabBar({ active, onSelect }: TabBarProps) {
  return (
    <View className="flex-row border-b border-space-700 mx-4 mb-3">
      {TABS.map((tab) => (
        <Pressable
          key={tab.id}
          onPress={() => onSelect(tab.id)}
          className={`flex-1 items-center pb-2.5 pt-1 cursor-pointer ${
            active === tab.id ? "border-b-2 border-dawn-400" : ""
          }`}
        >
          <Text
            className={`text-sm font-medium ${
              active === tab.id ? "text-dawn-400" : "text-nyx-text-muted"
            }`}
          >
            {tab.label}
          </Text>
        </Pressable>
      ))}
    </View>
  );
}

// ─── Post Grid Item ───────────────────────────────────────────────────────────

function PostGridItem({ post, size }: { post: Post; size: number }) {
  return (
    <Link href={`/(main)/post/${post.id}` as never} asChild>
      <Pressable
        style={{ width: size, height: size }}
        className="cursor-pointer"
      >
        <Image
          source={{
            uri: post.media[0]?.url ?? post.media[0]?.thumbnail_url,
          }}
          style={{ width: size, height: size }}
          contentFit="cover"
          transition={200}
          placeholder={{ blurhash: "L6PZfSi_.AyE_3t7t7R**0o#DgR4" }}
        />
        {post.media.length > 1 && (
          <View className="absolute top-1.5 right-1.5 bg-space-900/70 rounded-full w-4 h-4 items-center justify-center">
            <Text className="text-star-100 text-[9px] font-bold">+</Text>
          </View>
        )}
      </Pressable>
    </Link>
  );
}

// ─── People Card ──────────────────────────────────────────────────────────────

function PeopleCard({ profile }: { profile: UzumeProfile }) {
  const [following, setFollowing] = useState(profile.is_following ?? false);

  const handleFollow = async () => {
    const next = !following;
    setFollowing(next);
    try {
      if (next) await import("@nyx/api").then((m) => m.profilesApi.follow(profile.alias));
      else await import("@nyx/api").then((m) => m.profilesApi.unfollow(profile.alias));
    } catch {
      setFollowing(!next);
    }
  };

  return (
    <View className="bg-space-700 border border-space-600 rounded-2xl p-4 mx-4 mb-3 flex-row items-center gap-3">
      <Link href={`/(main)/profile/${profile.alias}` as never} asChild>
        <Pressable className="cursor-pointer">
          <Avatar uri={profile.avatar_url} alias={profile.alias} size="lg" />
        </Pressable>
      </Link>
      <View className="flex-1">
        <View className="flex-row items-center gap-1 mb-0.5">
          <Link href={`/(main)/profile/${profile.alias}` as never} asChild>
            <Pressable className="cursor-pointer">
              <Text className="text-star-100 font-semibold text-sm">
                {profile.display_name}
              </Text>
            </Pressable>
          </Link>
          {profile.is_verified && <VerifiedIcon size={13} />}
        </View>
        <Text className="text-nyx-text-muted text-xs mb-1">
          @{profile.alias}
        </Text>
        {profile.bio && (
          <Text className="text-star-300 text-xs" numberOfLines={1}>
            {profile.bio}
          </Text>
        )}
        <Text className="text-nyx-text-muted text-xs mt-0.5">
          {formatCount(profile.followers_count)} followers
        </Text>
      </View>
      <Pressable
        onPress={handleFollow}
        className={`h-8 px-4 rounded-xl items-center justify-center cursor-pointer ${
          following
            ? "bg-space-600 border border-space-500"
            : ""
        }`}
        style={
          !following
            ? ({
                background:
                  "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 100%)",
              } as never)
            : undefined
        }
      >
        <Text
          className={`text-xs font-semibold ${
            following ? "text-star-300" : "text-space-900"
          }`}
        >
          {following ? "Following" : "Follow"}
        </Text>
      </Pressable>
    </View>
  );
}

// ─── Tag Search Result ────────────────────────────────────────────────────────

function TagRow({ tag }: { tag: TrendingTag }) {
  return (
    <Link href={`/(main)/tag/${tag.tag}` as never} asChild>
      <Pressable className="flex-row items-center gap-4 px-4 py-3 border-b border-space-700 cursor-pointer active:bg-space-800">
        <View
          className="w-11 h-11 rounded-full items-center justify-center"
          style={{
            background:
              "linear-gradient(135deg, #FF6B9D22 0%, #A78BFA22 100%)",
          } as never}
        >
          <Text className="text-dawn-400 font-bold text-base">#</Text>
        </View>
        <View className="flex-1">
          <Text className="text-star-100 font-semibold text-sm">#{tag.tag}</Text>
          <Text className="text-nyx-text-muted text-xs">
            {formatCount(tag.posts_count)} posts
          </Text>
        </View>
      </Pressable>
    </Link>
  );
}

// ─── Grid Posts ───────────────────────────────────────────────────────────────

function PostsGrid({ posts }: { posts: Post[] }) {
  const { width } = useWindowDimensions();
  const cols = 3;
  const gap = 1;
  const size = Math.floor((Math.min(width, 600) - gap * (cols - 1)) / cols);

  const rows: Post[][] = [];
  for (let i = 0; i < posts.length; i += cols) {
    rows.push(posts.slice(i, i + cols));
  }

  return (
    <View>
      {rows.map((row, ri) => (
        <View key={ri} className="flex-row" style={{ gap }}>
          {row.map((post) => (
            <PostGridItem key={post.id} post={post} size={size} />
          ))}
          {row.length < cols &&
            Array.from({ length: cols - row.length }).map((_, i) => (
              <View key={`empty-${i}`} style={{ width: size, height: size }} />
            ))}
        </View>
      ))}
    </View>
  );
}

// ─── Empty State ──────────────────────────────────────────────────────────────

function EmptyState({ query }: { query: string }) {
  return (
    <View className="items-center justify-center py-16 px-8">
      <View
        className="w-20 h-20 rounded-full items-center justify-center mb-5"
        style={{
          background: "linear-gradient(135deg, #FF6B9D22 0%, #A78BFA22 100%)",
        } as never}
      >
        <ExploreIcon size={36} color="#7C6FA0" />
      </View>
      <Text className="text-star-100 text-lg font-bold mb-2 text-center">
        {query ? `No results for "${query}"` : "Nothing here yet"}
      </Text>
      <Text className="text-nyx-text-muted text-sm text-center">
        {query
          ? "Try a different search term or explore trending tags."
          : "Check back later for new content."}
      </Text>
    </View>
  );
}

// ─── Screen ───────────────────────────────────────────────────────────────────

export default function ExploreScreen() {
  const [query, setQuery] = useState("");
  const [tab, setTab] = useState<TabId>("posts");
  const [loading, setLoading] = useState(true);
  const [searching, setSearching] = useState(false);

  const [explorePosts, setExplorePosts] = useState<Post[]>([]);
  const [trendingTags, setTrendingTags] = useState<TrendingTag[]>([]);
  const [suggestedProfiles, setSuggestedProfiles] = useState<UzumeProfile[]>([]);

  const [searchPosts, setSearchPosts] = useState<Post[]>([]);
  const [searchPeople, setSearchPeople] = useState<UzumeProfile[]>([]);
  const [searchTags, setSearchTags] = useState<TrendingTag[]>([]);

  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const load = async () => {
      try {
        const [explore, trending, suggested] = await Promise.all([
          discoverApi.getExplore(),
          discoverApi.getTrending(),
          discoverApi.getSuggestedProfiles(),
        ]);
        setExplorePosts(explore.data?.posts ?? []);
        setTrendingTags(trending);
        setSuggestedProfiles(suggested);
      } catch {}
      setLoading(false);
    };
    load();
  }, []);

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    if (!query.trim()) {
      setSearchPosts([]);
      setSearchPeople([]);
      setSearchTags([]);
      return;
    }
    debounceRef.current = setTimeout(async () => {
      setSearching(true);
      try {
        const results = await discoverApi.search(query.trim());
        setSearchPosts(results.posts ?? []);
        setSearchPeople(results.profiles ?? []);
        setSearchTags(results.tags ?? []);
      } catch {}
      setSearching(false);
    }, 350);
  }, [query]);

  const inSearch = query.trim().length > 0;

  const postsToShow = inSearch ? searchPosts : explorePosts;
  const peopleToShow = inSearch ? searchPeople : suggestedProfiles;
  const tagsToShow = inSearch ? searchTags : trendingTags;

  return (
    <View className="flex-1 bg-space-900">
      <SearchBar
        value={query}
        onChange={setQuery}
        onClear={() => setQuery("")}
      />

      {!inSearch && (
        <TrendingTagsRow tags={trendingTags} />
      )}

      {inSearch && (
        <TabBar active={tab} onSelect={setTab} />
      )}

      {loading ? (
        <View className="flex-row flex-wrap gap-0.5 px-0">
          {Array.from({ length: 9 }).map((_, i) => (
            <Skeleton key={i} height={130} width="32.5%" rounded="none" />
          ))}
        </View>
      ) : searching ? (
        <View className="py-12 items-center">
          <ActivityIndicator color="#FF6B9D" />
        </View>
      ) : inSearch ? (
        <ScrollView showsVerticalScrollIndicator={false}>
          {tab === "posts" && (
            postsToShow.length > 0 ? (
              <PostsGrid posts={postsToShow} />
            ) : (
              <EmptyState query={query} />
            )
          )}
          {tab === "people" && (
            peopleToShow.length > 0 ? (
              <View className="pt-2">
                {peopleToShow.map((p) => (
                  <PeopleCard key={p.id} profile={p} />
                ))}
              </View>
            ) : (
              <EmptyState query={query} />
            )
          )}
          {tab === "tags" && (
            tagsToShow.length > 0 ? (
              <View>
                {tagsToShow.map((t) => (
                  <TagRow key={t.tag} tag={t} />
                ))}
              </View>
            ) : (
              <EmptyState query={query} />
            )
          )}
        </ScrollView>
      ) : (
        <ScrollView showsVerticalScrollIndicator={false}>
          {postsToShow.length > 0 ? (
            <PostsGrid posts={postsToShow} />
          ) : (
            <EmptyState query="" />
          )}
        </ScrollView>
      )}
    </View>
  );
}
