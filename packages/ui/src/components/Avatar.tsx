import React from "react";
import { View, Text, Pressable } from "react-native";
import { Image } from "expo-image";

interface AvatarProps {
  uri?: string;
  alias?: string;
  size?: "xs" | "sm" | "md" | "lg" | "xl" | "2xl";
  hasStory?: boolean;
  hasUnseenStory?: boolean;
  onPress?: () => void;
  className?: string;
}

const sizeMap = {
  xs: "w-6 h-6",
  sm: "w-8 h-8",
  md: "w-10 h-10",
  lg: "w-12 h-12",
  xl: "w-16 h-16",
  "2xl": "w-24 h-24",
};

const textSizeMap = {
  xs: "text-xs",
  sm: "text-sm",
  md: "text-base",
  lg: "text-lg",
  xl: "text-2xl",
  "2xl": "text-3xl",
};

const storyRingMap = {
  xs: "p-0.5",
  sm: "p-0.5",
  md: "p-[2px]",
  lg: "p-[3px]",
  xl: "p-1",
  "2xl": "p-1",
};

function getInitials(alias?: string): string {
  if (!alias) return "?";
  return alias.charAt(0).toUpperCase();
}

export function Avatar({
  uri,
  alias,
  size = "md",
  hasStory = false,
  hasUnseenStory = false,
  onPress,
  className,
}: AvatarProps) {
  const content = (
    <View
      className={[
        hasStory ? storyRingMap[size] : "",
        hasStory
          ? hasUnseenStory
            ? "rounded-full bg-dawn-gradient"
            : "rounded-full bg-space-500"
          : "",
      ]
        .filter(Boolean)
        .join(" ")}
      style={
        hasStory && hasUnseenStory
          ? ({
              background:
                "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 50%, #FFD93D 100%)",
              padding: size === "2xl" ? 3 : size === "xl" ? 3 : 2,
              borderRadius: 9999,
            } as never)
          : undefined
      }
    >
      <View
        className={`${sizeMap[size]} rounded-full bg-space-700 border border-space-500 overflow-hidden items-center justify-center`}
      >
        {uri ? (
          <Image
            source={{ uri }}
            style={{ width: "100%", height: "100%" }}
            contentFit="cover"
            transition={200}
          />
        ) : (
          <Text
            className={`${textSizeMap[size]} font-semibold text-star-300`}
          >
            {getInitials(alias)}
          </Text>
        )}
      </View>
    </View>
  );

  if (onPress) {
    return (
      <Pressable onPress={onPress} className={`cursor-pointer ${className ?? ""}`}>
        {content}
      </Pressable>
    );
  }

  return <View className={className}>{content}</View>;
}
