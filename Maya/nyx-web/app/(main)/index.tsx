import { View, Text, TouchableOpacity, Linking, Platform } from "react-native";
import { useAuth } from "../../src/context/AuthContext";
import { Button, Card } from "@nyx/ui";
import { LogoutIcon, ProfileIcon, ShieldIcon } from "@nyx/ui";

export default function AccountPortal() {
  const { user, logout } = useAuth();
  
  return (
    <View className="flex-1 star-field bg-[#060412]">
      <View className="px-6 pt-16 pb-4 border-b border-[#2A2460]/60 flex-row justify-between items-center glass-card rounded-b-3xl absolute top-0 w-full z-10">
        <Text className="text-2xl font-bold text-dawn-gradient">Nyx Portal</Text>
        <TouchableOpacity onPress={logout} className="p-2 bg-red-500/10 rounded-full">
          <LogoutIcon size={20} color="#FF6B9D" />
        </TouchableOpacity>
      </View>

      <View className="flex-1 px-6 pt-32 pb-8">
        <View className="items-center mb-8">
          <View className="p-1 rounded-full border-2 border-[#FFD93D] mb-4">
            <View className="w-24 h-24 rounded-full bg-[#13103A] items-center justify-center border border-[#2A2460]">
               <ProfileIcon size={40} color="#FFD93D" />
            </View>
          </View>
          <Text className="text-2xl font-bold text-white mb-1">{user?.display_name || "User"}</Text>
          <Text className="text-gray-400">{user?.email || "No email"}</Text>
          <View className="mt-2 bg-[#2A2460] px-3 py-1 rounded-full border border-[#4A3F9A]">
             <Text className="text-xs text-[#A78BFA] font-bold">ACTIVE PORTAL</Text>
          </View>
        </View>

        <Text className="text-lg font-bold text-white mb-4">Account Settings</Text>
        
        <View className="gap-4">
          <Card className="glass-card p-4 flex-row items-center border-[#2A2460]/80">
            <View className="w-10 h-10 rounded-full bg-[#FF6B9D]/10 items-center justify-center mr-4">
              <ProfileIcon size={20} color="#FF6B9D" />
            </View>
            <View className="flex-1">
              <Text className="text-white font-bold">Personal Information</Text>
              <Text className="text-gray-400 text-sm">Update your identity and display name</Text>
            </View>
          </Card>

          <Card className="glass-card p-4 flex-row items-center border-[#2A2460]/80">
            <View className="w-10 h-10 rounded-full bg-[#A78BFA]/10 items-center justify-center mr-4">
              <ShieldIcon size={20} color="#A78BFA" />
            </View>
            <View className="flex-1">
              <Text className="text-white font-bold">Security</Text>
              <Text className="text-gray-400 text-sm">Password, 2FA, and active sessions</Text>
            </View>
          </Card>
        </View>
        
        <View className="flex-1" />
        
        <Button className="w-full bg-[#13103A] border border-[#2A2460]" onPress={() => {
           if (Platform.OS === 'web') {
             window.location.href = 'http://localhost:8081'; // uzume-web is probably on 8081
           } else {
             Linking.openURL('uzume://');
           }
        }}>
           <Text className="text-white font-bold text-center w-full">Access Uzume Platform</Text>
        </Button>
      </View>
    </View>
  );
}
