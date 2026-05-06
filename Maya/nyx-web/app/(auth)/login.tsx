import { useState } from "react";
import {
  View,
  Text,
  TouchableOpacity,
  KeyboardAvoidingView,
  Platform,
} from "react-native";
import { useRouter } from "expo-router";
import { TextInput, Button } from "@nyx/ui";
import { useAuth, formatAuthError } from "../../src/context/AuthContext";

export default function LoginScreen() {
  const { login } = useAuth();
  const router = useRouter();

  const [identifier, setIdentifier] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleLogin() {
    const id = identifier.trim();
    if (!id || !password) {
      setError("Enter your email (or @username) and password.");
      return;
    }
    setError("");
    setLoading(true);
    try {
      await login(id, password);
      // AuthProvider session save → root nav redirects to "/"
    } catch (e) {
      setError(formatAuthError(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1"
    >
      <View className="flex-1 items-center justify-center p-6">
        {/* Logo */}
        <View className="items-center mb-10">
          <Text
            className="text-5xl font-bold mb-2"
            style={{ color: "#FFD93D", letterSpacing: 2 }}
          >
            nyx
          </Text>
        </View>

        {/* Form */}
        <View className="w-full max-w-xs">
          {error ? (
            <View className="bg-red-500/10 border border-red-500/20 rounded-xl p-3 mb-4">
              <Text className="text-red-400 text-sm text-center">{error}</Text>
            </View>
          ) : null}

          <TextInput
            placeholder="Email or @username"
            value={identifier}
            onChangeText={setIdentifier}
            autoCapitalize="none"
            autoComplete="email"
            keyboardType="email-address"
            returnKeyType="next"
            variant="minimal"
            className="mb-4"
          />

          <TextInput
            placeholder="Password"
            value={password}
            onChangeText={setPassword}
            secureTextEntry
            returnKeyType="done"
            onSubmitEditing={handleLogin}
            variant="minimal"
            className="mb-8"
          />

          <Button
            label="Log in"
            variant="dawn"
            size="lg"
            fullWidth
            loading={loading}
            onPress={handleLogin}
          />

          <View className="flex-row justify-center mt-4">
            <Text className="text-star-400 text-sm">Don't have an account? </Text>
            <TouchableOpacity onPress={() => router.replace("/register")}>
              <Text className="text-dawn-400 text-sm font-semibold">
                Sign up
              </Text>
            </TouchableOpacity>
          </View>
        </View>
      </View>
    </KeyboardAvoidingView>
  );
}
