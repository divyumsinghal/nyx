/**
 * Composer entry — wire to media upload + caption when the post pipeline is ready.
 */
import React from "react";
import { View, Text, Pressable } from "react-native";
import { router } from "expo-router";

export default function NewPostScreen() {
  return (
    <View className="flex-1 bg-space-900 px-5 pt-6">
      <Text className="text-star-100 text-lg font-semibold mb-2">Create a post</Text>
      <Text className="text-star-400 text-sm leading-6 mb-6">
        The full composer (media, captions, visibility) will connect here. For now this route
        exists so navigation and typed routes stay consistent.
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
