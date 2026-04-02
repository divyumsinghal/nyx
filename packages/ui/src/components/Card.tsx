import React from "react";
import { View, Pressable } from "react-native";

interface CardProps {
  children: React.ReactNode;
  variant?: "default" | "raised" | "glass" | "flat";
  onPress?: () => void;
  className?: string;
  padding?: boolean;
}

const variantStyles = {
  default: "bg-space-700 border border-space-500 rounded-2xl",
  raised: "bg-space-600 border border-nyx-border-glow rounded-2xl",
  glass: "rounded-2xl border border-space-500",
  flat: "bg-space-800 rounded-2xl",
};

export function Card({
  children,
  variant = "default",
  onPress,
  className,
  padding = true,
}: CardProps) {
  const style =
    variant === "glass"
      ? ({
          background: "rgba(19, 16, 58, 0.6)",
          backdropFilter: "blur(12px)",
        } as never)
      : undefined;

  const base = [
    variantStyles[variant],
    padding ? "p-4" : "",
    className ?? "",
  ]
    .filter(Boolean)
    .join(" ");

  if (onPress) {
    return (
      <Pressable
        onPress={onPress}
        className={`${base} cursor-pointer active:opacity-80`}
        style={style}
      >
        {children}
      </Pressable>
    );
  }

  return (
    <View className={base} style={style}>
      {children}
    </View>
  );
}
