import React, { useState } from "react";
import {
  View,
  Text,
  TextInput as RNTextInput,
  Pressable,
  type TextInputProps as RNTextInputProps,
} from "react-native";
import { EyeIcon, EyeOffIcon } from "../icons/UIIcons";

interface TextInputProps extends Omit<RNTextInputProps, "style"> {
  label?: string;
  error?: string;
  hint?: string;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  variant?: "default" | "search";
  className?: string;
}

export function TextInput({
  label,
  error,
  hint,
  leftIcon,
  rightIcon,
  variant = "default",
  secureTextEntry,
  className,
  ...props
}: TextInputProps) {
  const [showPassword, setShowPassword] = useState(false);
  const isPassword = secureTextEntry;

  return (
    <View className={`w-full ${className ?? ""}`}>
      {label && (
        <Text className="text-star-300 text-sm font-medium mb-1.5">
          {label}
        </Text>
      )}
      <View
        className={[
          "flex-row items-center rounded-xl border",
          variant === "search"
            ? "bg-space-700 border-space-500 h-10"
            : "bg-space-700 border-space-500 h-12",
          error ? "border-red-500" : "border-space-500 focus-within:border-dawn-400",
          "transition-colors duration-200",
        ]
          .filter(Boolean)
          .join(" ")}
      >
        {leftIcon && (
          <View className="pl-3 pr-2">{leftIcon}</View>
        )}
        <RNTextInput
          className="flex-1 text-star-100 text-base px-3 h-full"
          placeholderTextColor="#4A3F6B"
          secureTextEntry={isPassword && !showPassword}
          style={{ outline: "none" } as never}
          {...props}
        />
        {isPassword ? (
          <Pressable
            onPress={() => setShowPassword((v: boolean) => !v)}
            className="pr-3 cursor-pointer"
            accessibilityLabel={showPassword ? "Hide password" : "Show password"}
          >
            {showPassword ? (
              <EyeOffIcon size={18} color="#7C6FA0" />
            ) : (
              <EyeIcon size={18} color="#7C6FA0" />
            )}
          </Pressable>
        ) : rightIcon ? (
          <View className="pr-3">{rightIcon}</View>
        ) : null}
      </View>
      {error && (
        <Text className="text-red-400 text-xs mt-1">{error}</Text>
      )}
      {hint && !error && (
        <Text className="text-nyx-text-muted text-xs mt-1">{hint}</Text>
      )}
    </View>
  );
}
