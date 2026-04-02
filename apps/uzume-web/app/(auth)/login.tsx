import React, { useState } from "react";
import {
  View,
  Text,
  Pressable,
  KeyboardAvoidingView,
  ScrollView,
  Platform,
} from "react-native";
import { Link, router } from "expo-router";
import { useAuth } from "../../src/context/AuthContext";
import { TextInput } from "@nyx/ui";
import { UzumeLogoIcon } from "@nyx/ui";
import { ApiError } from "@nyx/api";

export default function LoginScreen() {
  const { login } = useAuth();
  const [identifier, setIdentifier] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleLogin() {
    if (!identifier.trim() || !password) return;
    setError("");
    setLoading(true);
    try {
      await login(identifier.trim(), password);
      router.replace("/(main)");
    } catch (e) {
      if (e instanceof ApiError) {
        setError(
          e.status === 401
            ? "Invalid email or password."
            : e.message,
        );
      } else {
        setError("Something went wrong. Please try again.");
      }
    } finally {
      setLoading(false);
    }
  }

  return (
    <View className="flex-1 bg-space-900 star-field">
      {/* Ambient dawn glow */}
      <View
        className="absolute inset-0 pointer-events-none"
        style={{
          background:
            "radial-gradient(ellipse 60% 40% at 50% 0%, rgba(255,107,157,0.12) 0%, transparent 70%)",
        } as never}
      />

      <KeyboardAvoidingView
        behavior={Platform.OS === "ios" ? "padding" : "height"}
        className="flex-1"
      >
        <ScrollView
          contentContainerStyle={{ flexGrow: 1 }}
          keyboardShouldPersistTaps="handled"
        >
          <View className="flex-1 justify-center items-center px-6 py-12">
            {/* Logo */}
            <View className="items-center mb-10">
              <View className="mb-4">
                <UzumeLogoIcon size={56} />
              </View>
              <Text
                className="text-4xl font-extrabold text-dawn-gradient"
                style={{
                  background:
                    "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 50%, #FFD93D 100%)",
                  WebkitBackgroundClip: "text",
                  WebkitTextFillColor: "transparent",
                  backgroundClip: "text",
                } as never}
              >
                Uzume
              </Text>
              <Text className="text-star-600 text-sm mt-1">
                Own your social experience
              </Text>
            </View>

            {/* Card */}
            <View className="w-full max-w-sm glass-card rounded-3xl p-8">
              <Text className="text-star-100 text-2xl font-bold mb-2">
                Welcome back
              </Text>
              <Text className="text-nyx-text-muted text-sm mb-8">
                Sign in to continue to Uzume
              </Text>

              <View className="gap-4">
                <TextInput
                  label="Email or username"
                  placeholder="you@example.com"
                  value={identifier}
                  onChangeText={setIdentifier}
                  autoCapitalize="none"
                  keyboardType="email-address"
                  autoComplete="email"
                />
                <TextInput
                  label="Password"
                  placeholder="Enter your password"
                  value={password}
                  onChangeText={setPassword}
                  secureTextEntry
                  autoComplete="current-password"
                />

                {error !== "" && (
                  <View className="bg-red-950 border border-red-800 rounded-xl px-4 py-3">
                    <Text className="text-red-400 text-sm">{error}</Text>
                  </View>
                )}

                <Pressable
                  onPress={handleLogin}
                  disabled={loading || !identifier || !password}
                  className="h-12 rounded-2xl items-center justify-center cursor-pointer mt-2 active:opacity-80 disabled:opacity-50"
                  style={{
                    background:
                      "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 60%, #FFD93D 100%)",
                  } as never}
                >
                  <Text className="text-space-900 font-bold text-base">
                    {loading ? "Signing in…" : "Sign in"}
                  </Text>
                </Pressable>

                <Pressable className="items-center cursor-pointer">
                  <Text className="text-star-500 text-sm">
                    Forgot password?
                  </Text>
                </Pressable>
              </View>
            </View>

            {/* Sign up link */}
            <View className="flex-row items-center mt-8 gap-1">
              <Text className="text-nyx-text-muted text-sm">
                Don't have an account?
              </Text>
              <Link href="/(auth)/register" asChild>
                <Pressable className="cursor-pointer">
                  <Text className="text-dawn-400 text-sm font-semibold">
                    Sign up
                  </Text>
                </Pressable>
              </Link>
            </View>
          </View>
        </ScrollView>
      </KeyboardAvoidingView>
    </View>
  );
}
