import React from "react";
import { Pressable, Text, View, ActivityIndicator } from "react-native";

interface ButtonProps {
  label: string;
  onPress?: () => void;
  variant?: "primary" | "secondary" | "ghost" | "danger" | "dawn";
  size?: "sm" | "md" | "lg";
  loading?: boolean;
  disabled?: boolean;
  icon?: React.ReactNode;
  iconPosition?: "left" | "right";
  fullWidth?: boolean;
  className?: string;
}

const variantStyles = {
  primary: "bg-space-500 border border-nyx-border-glow active:opacity-80",
  secondary: "bg-space-700 border border-space-500 active:bg-space-600",
  ghost: "bg-transparent border border-space-500 active:bg-space-700",
  danger: "bg-red-900 border border-red-700 active:opacity-80",
  dawn: "border-0 active:opacity-80",
};

const textStyles = {
  primary: "text-star-100 font-semibold",
  secondary: "text-star-200 font-medium",
  ghost: "text-star-300 font-medium",
  danger: "text-red-300 font-semibold",
  dawn: "text-space-900 font-bold",
};

const sizeStyles = {
  sm: "h-8 px-3 rounded-lg",
  md: "h-10 px-4 rounded-xl",
  lg: "h-12 px-6 rounded-2xl",
};

const textSizeStyles = {
  sm: "text-sm",
  md: "text-base",
  lg: "text-base",
};

export function Button({
  label,
  onPress,
  variant = "primary",
  size = "md",
  loading = false,
  disabled = false,
  icon,
  iconPosition = "left",
  fullWidth = false,
  className,
}: ButtonProps) {
  const isDisabled = disabled || loading;

  return (
    <Pressable
      onPress={onPress}
      disabled={isDisabled}
      className={[
        "flex-row items-center justify-center cursor-pointer",
        sizeStyles[size],
        variantStyles[variant],
        fullWidth ? "w-full" : "self-start",
        isDisabled ? "opacity-50" : "",
        className ?? "",
      ]
        .filter(Boolean)
        .join(" ")}
      style={
        variant === "dawn"
          ? {
              background:
                "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 50%, #FFD93D 100%)",
            }
          : undefined
      }
    >
      {loading ? (
        <ActivityIndicator size="small" color="#F0EBF8" />
      ) : (
        <>
          {icon && iconPosition === "left" && (
            <View className="mr-2">{icon}</View>
          )}
          <Text className={`${textStyles[variant]} ${textSizeStyles[size]}`}>
            {label}
          </Text>
          {icon && iconPosition === "right" && (
            <View className="ml-2">{icon}</View>
          )}
        </>
      )}
    </Pressable>
  );
}

// Compact icon button
interface IconButtonProps {
  icon: React.ReactNode;
  onPress?: () => void;
  size?: "sm" | "md" | "lg";
  variant?: "ghost" | "filled" | "danger";
  label: string; // accessibility label
  disabled?: boolean;
  className?: string;
  badge?: number;
}

const iconBtnSize = {
  sm: "w-8 h-8",
  md: "w-10 h-10",
  lg: "w-12 h-12",
};

const iconBtnVariant = {
  ghost: "bg-transparent hover:bg-space-700 active:bg-space-600",
  filled: "bg-space-600 border border-space-500 hover:bg-space-500",
  danger: "bg-red-900/40 border border-red-800 hover:bg-red-800/60",
};

export function IconButton({
  icon,
  onPress,
  size = "md",
  variant = "ghost",
  label,
  disabled = false,
  className,
  badge,
}: IconButtonProps) {
  return (
    <Pressable
      onPress={onPress}
      disabled={disabled}
      accessibilityLabel={label}
      className={[
        "rounded-full items-center justify-center cursor-pointer relative",
        iconBtnSize[size],
        iconBtnVariant[variant],
        disabled ? "opacity-50" : "",
        className ?? "",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {icon}
      {badge !== undefined && badge > 0 && (
        <View className="absolute -top-0.5 -right-0.5 min-w-4 h-4 rounded-full bg-dawn-400 items-center justify-center px-1">
          <Text className="text-xs font-bold text-space-900">
            {badge > 99 ? "99+" : badge}
          </Text>
        </View>
      )}
    </Pressable>
  );
}
