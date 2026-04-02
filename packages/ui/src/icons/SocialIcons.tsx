import React from "react";
import { View } from "react-native";

interface IconProps { size?: number; color?: string; className?: string; }
const D = 24; const C = "#F0EBF8";

export function VerifiedIcon({ size = D, color = "#818CF8", className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M12 2L14.09 8.26L21 9.27L16.5 14.14L17.18 21.02L12 18.77L6.82 21.02L7.5 14.14L3 9.27L9.91 8.26L12 2Z" fill={color} />
        <path d="M9 12L11 14L15 10" stroke="white" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </View>
  );
}

export function FollowIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M16 21V19C16 17.9391 15.5786 16.9217 14.8284 16.1716C14.0783 15.4214 13.0609 15 12 15H5C3.93913 15 2.92172 15.4214 2.17157 16.1716C1.42143 16.9217 1 17.9391 1 19V21" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <circle cx="8.5" cy="7" r="4" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <line x1="20" y1="8" x2="20" y2="14" stroke={color} strokeWidth="2" strokeLinecap="round" />
        <line x1="23" y1="11" x2="17" y2="11" stroke={color} strokeWidth="2" strokeLinecap="round" />
      </svg>
    </View>
  );
}

export function UnfollowIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M16 21V19C16 17.9391 15.5786 16.9217 14.8284 16.1716C14.0783 15.4214 13.0609 15 12 15H5C3.93913 15 2.92172 15.4214 2.17157 16.1716C1.42143 16.9217 1 17.9391 1 19V21" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <circle cx="8.5" cy="7" r="4" stroke={color} strokeWidth="1.75" />
        <line x1="23" y1="11" x2="17" y2="11" stroke={color} strokeWidth="2" strokeLinecap="round" />
      </svg>
    </View>
  );
}

export function StarIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2Z" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </View>
  );
}

export function NyxLogoIcon({ size = D, color = "#FF6B9D", className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        {/* Stylized N as crescent + star */}
        <path
          d="M6 20V4L12 14L18 4V20"
          stroke={color}
          strokeWidth="2.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <circle cx="19" cy="5" r="2" fill={color} opacity="0.6" />
      </svg>
    </View>
  );
}

export function UzumeLogoIcon({ size = D, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        {/* Stylized U with star accent */}
        <path
          d="M5 4V14C5 16.7614 8.13401 19 12 19C15.866 19 19 16.7614 19 14V4"
          stroke="url(#uzumeGrad)"
          strokeWidth="2.5"
          strokeLinecap="round"
        />
        <circle cx="12" cy="5" r="1.5" fill="#FFD93D" />
        <defs>
          <linearGradient id="uzumeGrad" x1="5" y1="4" x2="19" y2="19" gradientUnits="userSpaceOnUse">
            <stop stopColor="#FF6B9D" />
            <stop offset="0.5" stopColor="#FF8C61" />
            <stop offset="1" stopColor="#FFD93D" />
          </linearGradient>
        </defs>
      </svg>
    </View>
  );
}
