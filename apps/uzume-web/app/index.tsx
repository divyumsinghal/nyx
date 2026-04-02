import { Redirect } from "expo-router";
import { useAuth } from "../src/context/AuthContext";
import { View, ActivityIndicator } from "react-native";

export default function Index() {
  const { isLoading, isAuthenticated } = useAuth();

  if (isLoading) {
    return (
      <View className="flex-1 bg-space-900 items-center justify-center">
        <ActivityIndicator size="large" color="#FF6B9D" />
      </View>
    );
  }

  return <Redirect href={isAuthenticated ? "/(main)" : "/(auth)/login"} />;
}
