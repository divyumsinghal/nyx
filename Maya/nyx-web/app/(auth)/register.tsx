import { useState } from "react";
import { View, Text, TouchableOpacity, ActivityIndicator } from "react-native";
import { useRouter } from "expo-router";
import { TextInput, Button, Card } from "@nyx/ui";
import { useAuth } from "../../src/context/AuthContext";

export default function RegisterScreen() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [error, setError] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { register } = useAuth();
  const router = useRouter();

  async function handleRegister() {
    if (!email || !password || !displayName) {
      setError("Please fill in all fields");
      return;
    }
    setError("");
    setIsSubmitting(true);
    try {
      await register(email, password, displayName);
      router.replace("/");
    } catch (e: any) {
      setError(e.message || "Failed to register");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <View className="flex-1 items-center justify-center p-6 star-field">
      <Card className="w-full max-w-sm p-8 glass-card">
        <View className="items-center mb-8">
          <Text className="text-4xl font-bold text-dawn-gradient mb-2">NYX</Text>
          <Text className="text-gray-400 text-center">Create Account</Text>
        </View>

        {error ? (
          <View className="bg-red-500/10 p-3 rounded-lg mb-4 border border-red-500/20">
            <Text className="text-red-400 text-sm text-center">{error}</Text>
          </View>
        ) : null}

        <View className="gap-4 mb-6">
          <TextInput
            placeholder="Email Address"
            value={email}
            onChangeText={setEmail}
            autoCapitalize="none"
            keyboardType="email-address"
          />
          <TextInput
            placeholder="Display Name"
            value={displayName}
            onChangeText={setDisplayName}
          />
          <TextInput
            placeholder="Password"
            value={password}
            onChangeText={setPassword}
            secureTextEntry
          />
        </View>

        <Button 
          onPress={handleRegister} 
          disabled={isSubmitting}
          className="w-full bg-[#FFD93D] mb-4"
        >
          {isSubmitting ? <ActivityIndicator color="black" /> : <Text className="font-bold text-black">Create Account</Text>}
        </Button>

        <TouchableOpacity onPress={() => router.push("/login")} className="items-center mt-4">
          <Text className="text-gray-400">
            Already have an account? <Text className="text-[#FFD93D]">Log in</Text>
          </Text>
        </TouchableOpacity>
      </Card>
    </View>
  );
}
