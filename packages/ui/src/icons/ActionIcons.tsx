import React from "react";
import { View } from "react-native";

interface IconProps {
  size?: number;
  color?: string;
  className?: string;
}

const D = 24;
const C = "#F0EBF8";

export function HeartIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path
          d="M20.84 4.61C20.3292 4.099 19.7228 3.69365 19.0554 3.41708C18.3879 3.14052 17.6725 2.99817 16.95 2.99817C16.2275 2.99817 15.5121 3.14052 14.8446 3.41708C14.1772 3.69365 13.5708 4.099 13.06 4.61L12 5.67L10.94 4.61C9.9083 3.5783 8.50903 2.99823 7.05 2.99823C5.59096 2.99823 4.19169 3.5783 3.16 4.61C2.1283 5.6417 1.54823 7.04097 1.54823 8.5C1.54823 9.95903 2.1283 11.3583 3.16 12.39L4.22 13.45L12 21.23L19.78 13.45L20.84 12.39C21.351 11.8792 21.7563 11.2728 22.0329 10.6054C22.3095 9.93789 22.4518 9.22248 22.4518 8.5C22.4518 7.77752 22.3095 7.0621 22.0329 6.39461C21.7563 5.72711 21.351 5.12075 20.84 4.61Z"
          stroke={color}
          strokeWidth="1.75"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    </View>
  );
}

export function HeartFilledIcon({ size = D, color = "#FF6B9D", className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path
          d="M20.84 4.61C20.3292 4.099 19.7228 3.69365 19.0554 3.41708C18.3879 3.14052 17.6725 2.99817 16.95 2.99817C16.2275 2.99817 15.5121 3.14052 14.8446 3.41708C14.1772 3.69365 13.5708 4.099 13.06 4.61L12 5.67L10.94 4.61C9.9083 3.5783 8.50903 2.99823 7.05 2.99823C5.59096 2.99823 4.19169 3.5783 3.16 4.61C2.1283 5.6417 1.54823 7.04097 1.54823 8.5C1.54823 9.95903 2.1283 11.3583 3.16 12.39L4.22 13.45L12 21.23L19.78 13.45L20.84 12.39C21.351 11.8792 21.7563 11.2728 22.0329 10.6054C22.3095 9.93789 22.4518 9.22248 22.4518 8.5C22.4518 7.77752 22.3095 7.0621 22.0329 6.39461C21.7563 5.72711 21.351 5.12075 20.84 4.61Z"
          fill={color}
          stroke={color}
          strokeWidth="1.75"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    </View>
  );
}

export function CommentIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path
          d="M21 15C21 15.5304 20.7893 16.0391 20.4142 16.4142C20.0391 16.7893 19.5304 17 19 17H7L3 21V5C3 4.46957 3.21071 3.96086 3.58579 3.58579C3.96086 3.21071 4.46957 3 5 3H19C19.5304 3 20.0391 3.21071 20.4142 3.58579C20.7893 3.96086 21 4.46957 21 5V15Z"
          stroke={color}
          strokeWidth="1.75"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    </View>
  );
}

export function ShareIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M4 12V20C4 20.5304 4.21071 21.0391 4.58579 21.4142C4.96086 21.7893 5.46957 22 6 22H18C18.5304 22 19.0391 21.7893 19.4142 21.4142C19.7893 21.0391 20 20.5304 20 20V12" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <path d="M16 6L12 2L8 6" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <path d="M12 2V15" stroke={color} strokeWidth="1.75" strokeLinecap="round" />
      </svg>
    </View>
  );
}

export function BookmarkIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M19 21L12 16L5 21V5C5 4.46957 5.21071 3.96086 5.58579 3.58579C5.96086 3.21071 6.46957 3 7 3H17C17.5304 3 18.0391 3.21071 18.4142 3.58579C18.7893 3.96086 19 4.46957 19 5V21Z" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </View>
  );
}

export function BookmarkFilledIcon({ size = D, color = "#A78BFA", className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M19 21L12 16L5 21V5C5 4.46957 5.21071 3.96086 5.58579 3.58579C5.96086 3.21071 6.46957 3 7 3H17C17.5304 3 18.0391 3.21071 18.4142 3.58579C18.7893 3.96086 19 4.46957 19 5V21Z" fill={color} stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </View>
  );
}

export function PlusIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M12 5V19M5 12H19" stroke={color} strokeWidth="2" strokeLinecap="round" />
      </svg>
    </View>
  );
}

export function MoreHorizIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <circle cx="5" cy="12" r="1.5" fill={color} />
        <circle cx="12" cy="12" r="1.5" fill={color} />
        <circle cx="19" cy="12" r="1.5" fill={color} />
      </svg>
    </View>
  );
}

export function CloseIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M18 6L6 18M6 6L18 18" stroke={color} strokeWidth="2" strokeLinecap="round" />
      </svg>
    </View>
  );
}

export function SendIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M22 2L11 13M22 2L15 22L11 13L2 9L22 2Z" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </View>
  );
}

export function EditIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M11 4H4C3.46957 4 2.96086 4.21071 2.58579 4.58579C2.21071 4.96086 2 5.46957 2 6V20C2 20.5304 2.21071 21.0391 2.58579 21.4142C2.96086 21.7893 3.46957 22 4 22H18C18.5304 22 19.0391 21.7893 19.4142 21.4142C19.7893 21.0391 20 20.5304 20 20V13" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <path d="M18.5 2.49998C18.8978 2.10216 19.4374 1.87866 20 1.87866C20.5626 1.87866 21.1022 2.10216 21.5 2.49998C21.8978 2.89781 22.1213 3.43737 22.1213 3.99998C22.1213 4.56259 21.8978 5.10216 21.5 5.49998L12 15L8 16L9 12L18.5 2.49998Z" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </View>
  );
}

export function TrashIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M3 6H5H21" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <path d="M8 6V4C8 3.46957 8.21071 2.96086 8.58579 2.58579C8.96086 2.21071 9.46957 2 10 2H14C14.5304 2 15.0391 2.21071 15.4142 2.58579C15.7893 2.96086 16 3.46957 16 4V6M19 6V20C19 20.5304 18.7893 21.0391 18.4142 21.4142C18.0391 21.7893 17.5304 22 17 22H7C6.46957 22 5.96086 21.7893 5.58579 21.4142C5.21071 21.0391 5 20.5304 5 20V6H19Z" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </View>
  );
}

export function FlagIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <line x1="4" y1="22" x2="4" y2="15" stroke={color} strokeWidth="1.75" strokeLinecap="round" />
      </svg>
    </View>
  );
}
