import { useState } from "react";
import { View, Text, TouchableOpacity, ActivityIndicator } from "react-native";
import { useRouter } from "expo-router";
import { TextInput, Button, Card } from "@nyx/ui";
import { useAuth } from "../../src/context/AuthContext";

export default function LoginScreen() {
  const [identifier, setIdentifier] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { login } = useAuth();
  const router = useRouter();

  async function handleLogin() {
    if (!identifier || !password) {
      setError("Please fill in all fields");
      return;
    }
    setError("");
    setIsSubmitting(true);
    try {
      await login(identifier, password);
      router.replace("/");
    } catch (e: any) {
      setError(e.message || "Failed to login");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <View className="flex-1 items-center justify-center p-6 star-field">
      <Card className="w-full max-w-sm p-8 glass-card">
        <View className="items-center mb-8">
          <Text className="text-4xl font-bold text-dawn-gradient mb-2">NYX</Text>
          <Text className="text-gray-400 text-center">Account Portal</Text>
        </View>

        {error ? (
          <View className="bg-red-500/10 p-3 rounded-lg mb-4 border border-red-500/20">
            <Text className="text-red-400 text-sm text-center">{error}</Text>
          </View>
        ) : null}

        <View className="gap-4 mb-6">
          <TextInput
            placeholder="Email or Username"
            value={identifier}
            onChangeText={setIdentifier}
            autoCapitalize="none"
          />
          <TextInput
            placeholder="Password"
            value={password}
            onChangeText={setPassword}
            secureTextEntry
          />
        </View>

        <Button 
          onPress={handleLogin} 
          disabled={isSubmitting}
          className="w-full bg-[#FF6B9D] mb-4"
        >
          {isSubmitting ? <ActivityIndicator color="white" /> : <Text className="text-white font-bold">Sign In</Text>}
        </Button>

        <TouchableOpacity onPress={() => router.push("/register")} className="items-center mt-4">
          <Text className="text-gray-400">
            Don't have an account? <Text className="text-[#FF6B9D]">Register</Text>
          </Text>
        </TouchableOpacity>
      </Card>
    </View>
  );
}
