/**
 * Notifications — grouped by recency, with type-specific rendering.
 */
import React, { useState, useCallback } from "react";
import {
  View,
  Text,
  Pressable,
  SectionList,
  ActivityIndicator,
} from "react-native";
import { Image } from "expo-image";
import { router } from "expo-router";
import { Avatar } from "@nyx/ui";
import {
  HeartFilledIcon,
  CommentIcon,
  FollowIcon,
  BellIcon,
  VerifiedIcon,
  CheckIcon,
} from "@nyx/ui";

// ─── Types ────────────────────────────────────────────────────────────────────

type NotifType = "like" | "comment" | "follow" | "mention" | "reel_like" | "story_react";

interface Notification {
  id: string;
  type: NotifType;
  actor: {
    alias: string;
    display_name: string;
    avatar_url?: string;
    is_verified?: boolean;
  };
  post_thumbnail?: string;
  post_id?: string;
  comment_preview?: string;
  read: boolean;
  created_at: string;
}

// ─── Placeholder data ─────────────────────────────────────────────────────────
// Real notifications would come via WebSocket or long-poll; these represent
// the shape while the realtime infra is connected.

const PLACEHOLDER_NOTIFICATIONS: Notification[] = [
  {
    id: "n1",
    type: "like",
    actor: { alias: "aurora.v", display_name: "Aurora Voss", is_verified: true },
    post_thumbnail: "https://picsum.photos/seed/post1/80/80",
    post_id: "post-1",
    read: false,
    created_at: new Date(Date.now() - 5 * 60_000).toISOString(),
  },
  {
    id: "n2",
    type: "comment",
    actor: { alias: "nightstar", display_name: "Night Star" },
    comment_preview: "This is absolutely stunning 🌙",
    post_thumbnail: "https://picsum.photos/seed/post2/80/80",
    post_id: "post-2",
    read: false,
    created_at: new Date(Date.now() - 23 * 60_000).toISOString(),
  },
  {
    id: "n3",
    type: "follow",
    actor: { alias: "cosmicdancer", display_name: "Cosmic Dancer", is_verified: false },
    read: false,
    created_at: new Date(Date.now() - 50 * 60_000).toISOString(),
  },
  {
    id: "n4",
    type: "mention",
    actor: { alias: "solarnyx", display_name: "Solar Nyx" },
    comment_preview: "Have you seen what @you made?",
    post_id: "post-3",
    read: false,
    created_at: new Date(Date.now() - 3 * 3600_000).toISOString(),
  },
  {
    id: "n5",
    type: "like",
    actor: { alias: "stellarwind", display_name: "Stellar Wind" },
    post_thumbnail: "https://picsum.photos/seed/post5/80/80",
    post_id: "post-5",
    read: true,
    created_at: new Date(Date.now() - 1.2 * 86400_000).toISOString(),
  },
  {
    id: "n6",
    type: "follow",
    actor: { alias: "luminary.k", display_name: "Luminary K", is_verified: true },
    read: true,
    created_at: new Date(Date.now() - 2 * 86400_000).toISOString(),
  },
  {
    id: "n7",
    type: "reel_like",
    actor: { alias: "dawnbreak", display_name: "Dawnbreak" },
    post_thumbnail: "https://picsum.photos/seed/reel1/80/80",
    post_id: "reel-1",
    read: true,
    created_at: new Date(Date.now() - 5 * 86400_000).toISOString(),
  },
  {
    id: "n8",
    type: "story_react",
    actor: { alias: "nebulaxo", display_name: "Nebula XO" },
    read: true,
    created_at: new Date(Date.now() - 9 * 86400_000).toISOString(),
  },
  {
    id: "n9",
    type: "comment",
    actor: { alias: "midnighthue", display_name: "Midnight Hue" },
    comment_preview: "Love this so much 💫",
    post_id: "post-9",
    post_thumbnail: "https://picsum.photos/seed/post9/80/80",
    read: true,
    created_at: new Date(Date.now() - 14 * 86400_000).toISOString(),
  },
];

// ─── Grouping ─────────────────────────────────────────────────────────────────

function groupNotifications(notifications: Notification[]) {
  const now = Date.now();
  const today: Notification[] = [];
  const thisWeek: Notification[] = [];
  const earlier: Notification[] = [];

  for (const n of notifications) {
    const age = now - new Date(n.created_at).getTime();
    if (age < 86400_000) today.push(n);
    else if (age < 7 * 86400_000) thisWeek.push(n);
    else earlier.push(n);
  }

  return [
    ...(today.length ? [{ title: "Today", data: today }] : []),
    ...(thisWeek.length ? [{ title: "This Week", data: thisWeek }] : []),
    ...(earlier.length ? [{ title: "Earlier", data: earlier }] : []),
  ];
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

function timeAgo(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60_000);
  if (mins < 1) return "now";
  if (mins < 60) return `${mins}m`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h`;
  const days = Math.floor(hrs / 24);
  if (days < 7) return `${days}d`;
  return `${Math.floor(days / 7)}w`;
}

function notifIcon(type: NotifType) {
  switch (type) {
    case "like":
    case "reel_like":
      return <HeartFilledIcon size={16} color="#FF6B9D" />;
    case "comment":
    case "mention":
      return <CommentIcon size={16} color="#A78BFA" />;
    case "follow":
      return <FollowIcon size={16} color="#FF8C61" />;
    case "story_react":
      return <HeartFilledIcon size={16} color="#FFD93D" />;
    default:
      return <BellIcon size={16} color="#7C6FA0" />;
  }
}

function notifIconBg(type: NotifType): string {
  switch (type) {
    case "like":
    case "reel_like":
      return "#FF6B9D22";
    case "comment":
    case "mention":
      return "#A78BFA22";
    case "follow":
      return "#FF8C6122";
    case "story_react":
      return "#FFD93D22";
    default:
      return "#7C6FA022";
  }
}

function notifActionText(n: Notification): React.ReactNode {
  const bold = (s: string) => (
    <Text className="text-star-100 font-semibold">{s}</Text>
  );

  switch (n.type) {
    case "like":
      return (
        <Text className="text-star-200 text-sm leading-5">
          {bold(n.actor.display_name)} liked your post.
        </Text>
      );
    case "reel_like":
      return (
        <Text className="text-star-200 text-sm leading-5">
          {bold(n.actor.display_name)} liked your reel.
        </Text>
      );
    case "comment":
      return (
        <Text className="text-star-200 text-sm leading-5">
          {bold(n.actor.display_name)} commented:{" "}
          <Text className="text-star-300">"{n.comment_preview}"</Text>
        </Text>
      );
    case "mention":
      return (
        <Text className="text-star-200 text-sm leading-5">
          {bold(n.actor.display_name)} mentioned you:{" "}
          <Text className="text-star-300">"{n.comment_preview}"</Text>
        </Text>
      );
    case "follow":
      return (
        <Text className="text-star-200 text-sm leading-5">
          {bold(n.actor.display_name)} started following you.
        </Text>
      );
    case "story_react":
      return (
        <Text className="text-star-200 text-sm leading-5">
          {bold(n.actor.display_name)} reacted to your story.
        </Text>
      );
  }
}

// ─── Notification Row ─────────────────────────────────────────────────────────

function NotifRow({ notif }: { notif: Notification }) {
  const handlePress = () => {
    if (notif.post_id) {
      if (notif.type === "reel_like") {
        router.push(`/(main)/reel/${notif.post_id}` as never);
      } else {
        router.push(`/(main)/post/${notif.post_id}` as never);
      }
    } else if (notif.type === "follow") {
      router.push(`/(main)/profile/${notif.actor.alias}` as never);
    }
  };

  return (
    <Pressable
      onPress={handlePress}
      className={`flex-row items-center px-4 py-3 gap-3 cursor-pointer active:bg-space-800 ${
        !notif.read ? "bg-space-800/60" : ""
      }`}
    >
      {/* Unread dot */}
      {!notif.read && (
        <View className="absolute left-2 top-1/2 w-1.5 h-1.5 rounded-full bg-dawn-400" />
      )}

      {/* Avatar with icon badge */}
      <View className="relative">
        <Avatar uri={notif.actor.avatar_url} alias={notif.actor.alias} size="md" />
        <View
          className="absolute -bottom-0.5 -right-0.5 w-5 h-5 rounded-full items-center justify-center border-2 border-space-900"
          style={{ backgroundColor: notifIconBg(notif.type) } as never}
        >
          {notifIcon(notif.type)}
        </View>
      </View>

      {/* Text */}
      <View className="flex-1 gap-0.5">
        {notifActionText(notif)}
        <Text className="text-nyx-text-muted text-xs">{timeAgo(notif.created_at)}</Text>
      </View>

      {/* Post thumbnail */}
      {notif.post_thumbnail && (
        <View className="w-11 h-11 rounded-lg overflow-hidden border border-space-600">
          <Image
            source={{ uri: notif.post_thumbnail }}
            style={{ width: 44, height: 44 }}
            contentFit="cover"
            transition={200}
          />
        </View>
      )}
    </Pressable>
  );
}

// ─── Section Header ───────────────────────────────────────────────────────────

function SectionHeader({ title }: { title: string }) {
  return (
    <View className="bg-space-900 px-4 py-2 border-b border-space-700">
      <Text className="text-nyx-text-muted text-xs font-semibold uppercase tracking-wider">
        {title}
      </Text>
    </View>
  );
}

// ─── Screen ───────────────────────────────────────────────────────────────────

export default function NotificationsScreen() {
  const [notifications, setNotifications] = useState<Notification[]>(
    PLACEHOLDER_NOTIFICATIONS
  );
  const [allRead, setAllRead] = useState(false);

  const sections = groupNotifications(notifications);
  const unreadCount = notifications.filter((n) => !n.read).length;

  const markAllRead = useCallback(() => {
    setNotifications((prev) => prev.map((n) => ({ ...n, read: true })));
    setAllRead(true);
  }, []);

  return (
    <View className="flex-1 bg-space-900">
      {/* Header */}
      <View className="flex-row items-center justify-between px-4 py-3 border-b border-space-700 bg-space-800">
        <Text className="text-star-100 text-lg font-bold">Notifications</Text>
        {unreadCount > 0 && (
          <Pressable
            onPress={markAllRead}
            className="flex-row items-center gap-1.5 cursor-pointer"
          >
            <CheckIcon size={16} color="#FF6B9D" />
            <Text className="text-dawn-400 text-sm font-medium">Mark all read</Text>
          </Pressable>
        )}
      </View>

      {sections.length === 0 ? (
        <View className="flex-1 items-center justify-center py-16 px-8">
          <View
            className="w-20 h-20 rounded-full items-center justify-center mb-5"
            style={{
              background: "linear-gradient(135deg, #FF6B9D22 0%, #A78BFA22 100%)",
            } as never}
          >
            <BellIcon size={36} color="#7C6FA0" />
          </View>
          <Text className="text-star-100 text-xl font-bold mb-2 text-center">
            All quiet here
          </Text>
          <Text className="text-nyx-text-muted text-sm text-center">
            When people like, comment, or follow you, you'll see it here.
          </Text>
        </View>
      ) : (
        <SectionList
          sections={sections}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => <NotifRow notif={item} />}
          renderSectionHeader={({ section }) => (
            <SectionHeader title={section.title} />
          )}
          showsVerticalScrollIndicator={false}
          stickySectionHeadersEnabled
        />
      )}
    </View>
  );
}
