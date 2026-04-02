import React from "react";
import { View } from "react-native";

interface SkeletonProps {
  width?: string | number;
  height?: string | number;
  rounded?: "sm" | "md" | "lg" | "xl" | "full";
  className?: string;
}

const roundedMap = {
  sm: "rounded",
  md: "rounded-lg",
  lg: "rounded-xl",
  xl: "rounded-2xl",
  full: "rounded-full",
};

export function Skeleton({ width, height, rounded = "md", className }: SkeletonProps) {
  return (
    <View
      className={`bg-space-600 overflow-hidden ${roundedMap[rounded]} ${className ?? ""}`}
      style={{
        width,
        height,
        backgroundImage:
          "linear-gradient(90deg, transparent 0%, rgba(167,139,250,0.06) 50%, transparent 100%)",
        backgroundSize: "200% 100%",
        animation: "shimmer 1.5s infinite",
      } as never}
    />
  );
}

export function PostSkeleton() {
  return (
    <View className="bg-space-800 border-b border-space-700 p-4">
      {/* Header */}
      <View className="flex-row items-center gap-3 mb-3">
        <Skeleton width={40} height={40} rounded="full" />
        <View className="flex-1 gap-2">
          <Skeleton width={120} height={14} />
          <Skeleton width={80} height={12} />
        </View>
      </View>
      {/* Image placeholder */}
      <Skeleton width="100%" height={300} rounded="xl" className="mb-3" />
      {/* Actions */}
      <View className="flex-row gap-4">
        <Skeleton width={60} height={24} />
        <Skeleton width={60} height={24} />
        <Skeleton width={60} height={24} />
      </View>
    </View>
  );
}

export function ProfileSkeleton() {
  return (
    <View className="p-4">
      <View className="items-center mb-6">
        <Skeleton width={96} height={96} rounded="full" className="mb-3" />
        <Skeleton width={140} height={20} className="mb-2" />
        <Skeleton width={100} height={14} />
      </View>
      <View className="flex-row justify-around mb-6">
        {[0, 1, 2].map((i) => (
          <View key={i} className="items-center gap-1">
            <Skeleton width={50} height={20} />
            <Skeleton width={60} height={12} />
          </View>
        ))}
      </View>
      <View className="grid grid-cols-3 gap-0.5">
        {Array.from({ length: 9 }).map((_, i) => (
          <Skeleton key={i} height={120} rounded="none" />
        ))}
      </View>
    </View>
  );
}
