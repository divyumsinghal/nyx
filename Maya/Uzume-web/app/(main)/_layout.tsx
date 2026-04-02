import React, { useState } from "react";
import { View, Text, Pressable, useWindowDimensions } from "react-native";
import { Drawer } from "expo-router/drawer";
import { Redirect, usePathname, Link } from "expo-router";
import { useAuth } from "../../src/context/AuthContext";
import { Avatar } from "@nyx/ui";
import {
  HomeIcon,
  HomeFilledIcon,
  ExploreIcon,
  ReelsIcon,
  MessagesIcon,
  BellIcon,
  ProfileIcon,
  SettingsIcon,
  UzumeLogoIcon,
  PlusIcon,
} from "@nyx/ui";

interface NavItem {
  href: string;
  label: string;
  icon: React.ReactNode;
  activeIcon: React.ReactNode;
}

function NavItems(): NavItem[] {
  return [
    {
      href: "/(main)",
      label: "Home",
      icon: <HomeIcon size={22} />,
      activeIcon: <HomeFilledIcon size={22} color="#FF6B9D" />,
    },
    {
      href: "/(main)/explore",
      label: "Explore",
      icon: <ExploreIcon size={22} />,
      activeIcon: <ExploreIcon size={22} color="#FF6B9D" />,
    },
    {
      href: "/(main)/reels",
      label: "Reels",
      icon: <ReelsIcon size={22} />,
      activeIcon: <ReelsIcon size={22} color="#FF6B9D" />,
    },
    {
      href: "/(main)/messages",
      label: "Messages",
      icon: <MessagesIcon size={22} />,
      activeIcon: <MessagesIcon size={22} color="#FF6B9D" />,
    },
    {
      href: "/(main)/notifications",
      label: "Notifications",
      icon: <BellIcon size={22} />,
      activeIcon: <BellIcon size={22} color="#FF6B9D" />,
    },
    {
      href: "/(main)/profile/me",
      label: "Profile",
      icon: <ProfileIcon size={22} />,
      activeIcon: <ProfileIcon size={22} color="#FF6B9D" />,
    },
    {
      href: "/(main)/settings",
      label: "Settings",
      icon: <SettingsIcon size={22} />,
      activeIcon: <SettingsIcon size={22} color="#FF6B9D" />,
    },
  ];
}

function Sidebar() {
  const pathname = usePathname();
  const { profile, logout } = useAuth();
  const items = NavItems();

  const isActive = (href: string) => {
    if (href === "/(main)") return pathname === "/" || pathname === "/(main)" || pathname === "";
    return pathname.startsWith(href.replace("/(main)", ""));
  };

  return (
    <View className="h-full bg-space-900 border-r border-space-700 w-64 flex-col py-6">
      {/* Logo */}
      <View className="px-6 mb-8 flex-row items-center gap-3">
        <UzumeLogoIcon size={32} />
        <Text
          className="text-2xl font-bold"
          style={{
            background: "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 100%)",
            WebkitBackgroundClip: "text",
            WebkitTextFillColor: "transparent",
            backgroundClip: "text",
          } as never}
        >
          Uzume
        </Text>
      </View>

      {/* Nav items */}
      <View className="flex-1 px-3 gap-1">
        {items.map((item) => {
          const active = isActive(item.href);
          return (
            <Link key={item.href} href={item.href as never} asChild>
              <Pressable
                className={[
                  "flex-row items-center gap-3 px-3 h-12 rounded-xl cursor-pointer transition-colors duration-150",
                  active
                    ? "bg-space-700 border border-space-500"
                    : "hover:bg-space-800",
                ]
                  .filter(Boolean)
                  .join(" ")}
              >
                {active ? item.activeIcon : item.icon}
                <Text
                  className={`text-base font-medium ${
                    active ? "text-dawn-400" : "text-star-300"
                  }`}
                >
                  {item.label}
                </Text>
              </Pressable>
            </Link>
          );
        })}
      </View>

      {/* New Post CTA */}
      <View className="px-4 mb-4">
        <Link href="/(main)/new-post" asChild>
          <Pressable
            className="h-11 rounded-2xl flex-row items-center justify-center gap-2 cursor-pointer active:opacity-80"
            style={{
              background:
                "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 60%, #FFD93D 100%)",
            } as never}
          >
            <PlusIcon size={18} color="#030209" />
            <Text className="text-space-900 font-bold text-sm">New Post</Text>
          </Pressable>
        </Link>
      </View>

      {/* User card */}
      <View className="px-4 pt-4 border-t border-space-700">
        <Pressable className="flex-row items-center gap-3 rounded-xl p-2 cursor-pointer hover:bg-space-800">
          <Avatar
            uri={profile?.avatar_url}
            alias={profile?.alias ?? profile?.display_name}
            size="sm"
          />
          <View className="flex-1">
            <Text className="text-star-200 text-sm font-medium" numberOfLines={1}>
              {profile?.display_name ?? "Loading…"}
            </Text>
            <Text className="text-nyx-text-muted text-xs" numberOfLines={1}>
              @{profile?.alias ?? "…"}
            </Text>
          </View>
        </Pressable>
      </View>
    </View>
  );
}

function BottomNav() {
  const pathname = usePathname();
  const items = NavItems().slice(0, 5); // Only main 5 for mobile

  const isActive = (href: string) => {
    if (href === "/(main)") return pathname === "/" || pathname === "/(main)";
    return pathname.startsWith(href.replace("/(main)", ""));
  };

  return (
    <View className="h-16 bg-space-900 border-t border-space-700 flex-row items-center px-2">
      {items.map((item) => {
        const active = isActive(item.href);
        return (
          <Link key={item.href} href={item.href as never} asChild>
            <Pressable className="flex-1 items-center justify-center h-full cursor-pointer">
              {active ? item.activeIcon : item.icon}
            </Pressable>
          </Link>
        );
      })}
    </View>
  );
}

export default function MainLayout() {
  const { isAuthenticated, isLoading } = useAuth();
  const { width } = useWindowDimensions();
  const isDesktop = width >= 1024;

  if (!isLoading && !isAuthenticated) {
    return <Redirect href="/(auth)/login" />;
  }

  if (isDesktop) {
    return (
      <View className="flex-1 flex-row bg-space-900">
        <Sidebar />
        <View className="flex-1">
          <Drawer
            screenOptions={{ headerShown: false, drawerType: "permanent" }}
          />
        </View>
      </View>
    );
  }

  return (
    <View className="flex-1 bg-space-900">
      <View className="flex-1">
        <Drawer screenOptions={{ headerShown: false }} />
      </View>
      <BottomNav />
    </View>
  );
}
