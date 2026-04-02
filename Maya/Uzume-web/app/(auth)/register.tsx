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

export default function RegisterScreen() {
  const { register } = useAuth();
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleRegister() {
    if (!displayName.trim() || !email.trim() || !password) return;
    setError("");
    setLoading(true);
    try {
      await register(email.trim(), password, displayName.trim());
      router.replace("/(main)");
    } catch (e) {
      if (e instanceof ApiError) {
        setError(e.message);
      } else {
        setError("Something went wrong. Please try again.");
      }
    } finally {
      setLoading(false);
    }
  }

  return (
    <View className="flex-1 bg-space-900 star-field">
      <View
        className="absolute inset-0 pointer-events-none"
        style={{
          background:
            "radial-gradient(ellipse 60% 40% at 80% 100%, rgba(167,139,250,0.1) 0%, transparent 70%)",
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
            <View className="items-center mb-8">
              <View className="mb-4">
                <UzumeLogoIcon size={48} />
              </View>
              <Text
                className="text-3xl font-extrabold"
                style={{
                  background:
                    "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 50%, #FFD93D 100%)",
                  WebkitBackgroundClip: "text",
                  WebkitTextFillColor: "transparent",
                  backgroundClip: "text",
                } as never}
              >
                Join Uzume
              </Text>
              <Text className="text-nyx-text-muted text-sm mt-1">
                Own your digital social life
              </Text>
            </View>

            <View className="w-full max-w-sm glass-card rounded-3xl p-8">
              <Text className="text-star-100 text-2xl font-bold mb-2">
                Create account
              </Text>
              <Text className="text-nyx-text-muted text-sm mb-6">
                Privacy-first. No ads. No tracking.
              </Text>

              <View className="gap-4">
                <TextInput
                  label="Display name"
                  placeholder="Your name"
                  value={displayName}
                  onChangeText={setDisplayName}
                  autoComplete="name"
                />
                <TextInput
                  label="Email"
                  placeholder="you@example.com"
                  value={email}
                  onChangeText={setEmail}
                  autoCapitalize="none"
                  keyboardType="email-address"
                  autoComplete="email"
                />
                <TextInput
                  label="Password"
                  placeholder="At least 8 characters"
                  value={password}
                  onChangeText={setPassword}
                  secureTextEntry
                  autoComplete="new-password"
                  hint="Minimum 8 characters"
                />

                {error !== "" && (
                  <View className="bg-red-950 border border-red-800 rounded-xl px-4 py-3">
                    <Text className="text-red-400 text-sm">{error}</Text>
                  </View>
                )}

                <Pressable
                  onPress={handleRegister}
                  disabled={
                    loading || !displayName || !email || password.length < 8
                  }
                  className="h-12 rounded-2xl items-center justify-center cursor-pointer mt-2 active:opacity-80 disabled:opacity-50"
                  style={{
                    background:
                      "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 60%, #FFD93D 100%)",
                  } as never}
                >
                  <Text className="text-space-900 font-bold text-base">
                    {loading ? "Creating account…" : "Create account"}
                  </Text>
                </Pressable>

                <Text className="text-nyx-text-disabled text-xs text-center">
                  By signing up you agree to our Terms of Service and Privacy
                  Policy.
                </Text>
              </View>
            </View>

            <View className="flex-row items-center mt-8 gap-1">
              <Text className="text-nyx-text-muted text-sm">
                Already have an account?
              </Text>
              <Link href="/(auth)/login" asChild>
                <Pressable className="cursor-pointer">
                  <Text className="text-dawn-400 text-sm font-semibold">
                    Sign in
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
