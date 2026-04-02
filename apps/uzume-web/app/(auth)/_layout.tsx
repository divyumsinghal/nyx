import { Stack } from "expo-router";
import { useAuth } from "../../src/context/AuthContext";
import { Redirect } from "expo-router";

export default function AuthLayout() {
  const { isAuthenticated } = useAuth();
  if (isAuthenticated) return <Redirect href="/(main)" />;

  return (
    <Stack screenOptions={{ headerShown: false, animation: "slide_from_right" }} />
  );
}
