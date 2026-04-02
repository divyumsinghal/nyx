import React from "react";
import { View } from "react-native";

interface IconProps { size?: number; color?: string; className?: string; }
const D = 24; const C = "#F0EBF8";

export function PlayIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <polygon points="5 3 19 12 5 21 5 3" fill={color} />
      </svg>
    </View>
  );
}

export function PauseIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <rect x="6" y="4" width="4" height="16" rx="1" fill={color} />
        <rect x="14" y="4" width="4" height="16" rx="1" fill={color} />
      </svg>
    </View>
  );
}

export function MuteIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M11 5L6 9H2V15H6L11 19V5Z" fill={color} stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
        <line x1="23" y1="9" x2="17" y2="15" stroke={color} strokeWidth="2" strokeLinecap="round" />
        <line x1="17" y1="9" x2="23" y2="15" stroke={color} strokeWidth="2" strokeLinecap="round" />
      </svg>
    </View>
  );
}

export function VolumeIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" fill={color} />
        <path d="M15.54 8.46C16.4773 9.39764 17.004 10.6692 17.004 11.995C17.004 13.3208 16.4773 14.5924 15.54 15.53" stroke={color} strokeWidth="1.75" strokeLinecap="round" />
        <path d="M19.07 4.93C20.9447 6.80528 21.9979 9.34836 21.9979 12C21.9979 14.6516 20.9447 17.1947 19.07 19.07" stroke={color} strokeWidth="1.75" strokeLinecap="round" />
      </svg>
    </View>
  );
}

export function CameraIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M23 19C23 19.5304 22.7893 20.0391 22.4142 20.4142C22.0391 20.7893 21.5304 21 21 21H3C2.46957 21 1.96086 20.7893 1.58579 20.4142C1.21071 20.0391 1 19.5304 1 19V8C1 7.46957 1.21071 6.96086 1.58579 6.58579C1.96086 6.21071 2.46957 6 3 6H7L9 3H15L17 6H21C21.5304 6 22.0391 6.21071 22.4142 6.58579C22.7893 6.96086 23 7.46957 23 8V19Z" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <circle cx="12" cy="13" r="4" stroke={color} strokeWidth="1.75" />
      </svg>
    </View>
  );
}

export function ImageIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <rect x="3" y="3" width="18" height="18" rx="2" stroke={color} strokeWidth="1.75" />
        <circle cx="8.5" cy="8.5" r="1.5" fill={color} />
        <path d="M21 15L16 10L5 21" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </View>
  );
}

export function VideoIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <polygon points="23 7 16 12 23 17 23 7" fill={color} />
        <rect x="1" y="5" width="15" height="14" rx="2" stroke={color} strokeWidth="1.75" />
      </svg>
    </View>
  );
}

export function GridIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <rect x="3" y="3" width="7" height="7" rx="1" stroke={color} strokeWidth="1.75" />
        <rect x="14" y="3" width="7" height="7" rx="1" stroke={color} strokeWidth="1.75" />
        <rect x="3" y="14" width="7" height="7" rx="1" stroke={color} strokeWidth="1.75" />
        <rect x="14" y="14" width="7" height="7" rx="1" stroke={color} strokeWidth="1.75" />
      </svg>
    </View>
  );
}

export function MusicIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M9 18V5L21 3V16" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <circle cx="6" cy="18" r="3" stroke={color} strokeWidth="1.75" />
        <circle cx="18" cy="16" r="3" stroke={color} strokeWidth="1.75" />
      </svg>
    </View>
  );
}
