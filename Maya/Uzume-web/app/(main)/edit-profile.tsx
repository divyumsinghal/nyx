/**
 * Profile editor — extend with full form + avatar/header upload when wired to API.
 */
import React from "react";
import { View, Text, Pressable } from "react-native";
import { router } from "expo-router";

export default function EditProfileScreen() {
  return (
    <View className="flex-1 bg-space-900 px-5 pt-6">
      <Text className="text-star-100 text-lg font-semibold mb-2">Edit profile</Text>
      <Text className="text-star-400 text-sm leading-6 mb-6">
        Profile fields and uploads will live here. This screen exists so drawer and profile links
        resolve correctly.
      </Text>
      <Pressable
        onPress={() => router.back()}
        className="self-start px-4 py-2 rounded-xl bg-space-700 border border-space-600 cursor-pointer active:opacity-80"
      >
        <Text className="text-star-200 text-sm font-medium">Go back</Text>
      </Pressable>
    </View>
  );
}
