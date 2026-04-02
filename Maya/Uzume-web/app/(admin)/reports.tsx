import { View, Text, ScrollView, TouchableOpacity } from "react-native";
import { useRouter } from "expo-router";
import { Card, Button } from "@nyx/ui";
import { ChevronLeftIcon, FlagIcon, CheckIcon, TrashIcon } from "@nyx/ui";

export default function AdminReports() {
  const router = useRouter();
  
  return (
    <View className="flex-1 bg-[#060412]">
      <View className="px-4 pt-16 pb-4 border-b border-[#2A2460]/60 glass-card flex-row items-center">
        <TouchableOpacity onPress={() => router.back()} className="p-2 mr-2">
          <ChevronLeftIcon size={24} color="white" />
        </TouchableOpacity>
        <Text className="text-xl font-bold text-white">Active Reports</Text>
      </View>

      <ScrollView className="flex-1 p-4" showsVerticalScrollIndicator={false}>
        <Card className="p-4 bg-[#13103A] border border-[#FF6B9D]/40 mb-4">
          <View className="flex-row justify-between items-start mb-3">
             <Text className="text-white font-bold">Harassment Report</Text>
             <View className="bg-[#FF6B9D]/20 px-2 py-1 rounded">
               <Text className="text-[#FF6B9D] text-xs font-bold">HIGH PRIORITY</Text>
             </View>
          </View>
          <Text className="text-gray-300 mb-4 selection:bg-[#FF6B9D]/30">User @toxic_dude posted abusive behavior in the comments of a reel.</Text>
          <View className="flex-row gap-2 mt-2 pt-3 border-t border-[#2A2460]">
             <Button className="flex-1 bg-[#13103A] border border-red-500/50" onPress={() => {}}>
               <TrashIcon size={16} color="#ef4444" />
               <Text className="text-red-400 font-bold ml-2">Delete Content</Text>
             </Button>
             <Button className="flex-1 bg-[#4A3F9A]" onPress={() => {}}>
               <CheckIcon size={16} color="white" />
               <Text className="text-white font-bold ml-2">Dismiss</Text>
             </Button>
          </View>
        </Card>
      </ScrollView>
    </View>
  );
}
