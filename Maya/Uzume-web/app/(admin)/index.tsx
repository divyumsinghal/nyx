import { View, Text, ScrollView, TouchableOpacity } from "react-native";
import { useRouter } from "expo-router";
import { Card } from "@nyx/ui";
import { ShieldIcon, FlagIcon } from "@nyx/ui";

export default function AdminDashboard() {
  const router = useRouter();

  return (
    <View className="flex-1 bg-[#060412]">
      <View className="px-6 pt-16 pb-4 border-b border-[#2A2460]/60 glass-card">
        <Text className="text-2xl font-bold text-dawn-gradient">Admin Console</Text>
        <Text className="text-gray-400">Moderation and platform health</Text>
      </View>

      <ScrollView className="flex-1 p-6" showsVerticalScrollIndicator={false}>
        <View className="flex-row gap-4 mb-6">
          <Card className="flex-1 p-4 bg-[#13103A] border-[#2A2460]">
            <FlagIcon size={24} color="#FF6B9D" />
            <Text className="text-2xl font-bold text-white mt-2">12</Text>
            <Text className="text-gray-400 text-sm">Open Reports</Text>
          </Card>
          <Card className="flex-1 p-4 bg-[#13103A] border-[#2A2460]">
             <ShieldIcon size={24} color="#A78BFA" />
             <Text className="text-2xl font-bold text-white mt-2">100%</Text>
             <Text className="text-gray-400 text-sm">System Health</Text>
          </Card>
        </View>

        <Text className="text-lg font-bold text-white mb-4">Quick Actions</Text>
        
        <TouchableOpacity onPress={() => router.push("/(admin)/reports")}>
          <Card className="p-4 mb-4 bg-[#13103A] border-[#2A2460] flex-row items-center justify-between">
            <View className="flex-row items-center">
              <View className="p-3 bg-[#FF6B9D]/10 rounded-full mr-4">
                <FlagIcon size={20} color="#FF6B9D" />
              </View>
              <View>
                <Text className="text-white font-bold text-lg">Review Reports</Text>
                <Text className="text-gray-400">Handle user reports and content</Text>
              </View>
            </View>
          </Card>
        </TouchableOpacity>

      </ScrollView>
    </View>
  );
}
