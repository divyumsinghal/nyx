import React from "react";
import { View } from "react-native";

interface IconProps { size?: number; color?: string; className?: string; }
const D = 24; const C = "#F0EBF8";

export function CheckIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M20 6L9 17L4 12" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </View>
  );
}

export function InfoIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" stroke={color} strokeWidth="1.75" />
        <path d="M12 16V12M12 8H12.01" stroke={color} strokeWidth="1.75" strokeLinecap="round" />
      </svg>
    </View>
  );
}

export function AlertIcon({ size = D, color = "#FBBF24", className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M10.29 3.86L1.82 18C1.64 18.32 1.55 18.68 1.55 19.04C1.55 20.14 2.44 21.04 3.54 21.04H20.47C21.57 21.04 22.46 20.15 22.46 19.05C22.46 18.69 22.37 18.33 22.19 18.01L13.72 3.87C13.36 3.27 12.71 2.9 12.01 2.9C11.31 2.9 10.65 3.26 10.29 3.86Z" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <path d="M12 9V13M12 17H12.01" stroke={color} strokeWidth="1.75" strokeLinecap="round" />
      </svg>
    </View>
  );
}

export function LockIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <rect x="3" y="11" width="18" height="11" rx="2" stroke={color} strokeWidth="1.75" />
        <path d="M7 11V7C7 5.67392 7.52678 4.40215 8.46447 3.46447C9.40215 2.52678 10.6739 2 12 2C13.3261 2 14.5979 2.52678 15.5355 3.46447C16.4732 4.40215 17 5.67392 17 7V11" stroke={color} strokeWidth="1.75" strokeLinecap="round" />
      </svg>
    </View>
  );
}

export function EyeIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M1 12C1 12 5 4 12 4C19 4 23 12 23 12C23 12 19 20 12 20C5 20 1 12 1 12Z" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <circle cx="12" cy="12" r="3" stroke={color} strokeWidth="1.75" />
      </svg>
    </View>
  );
}

export function EyeOffIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M17.94 17.94C16.2306 19.243 14.1491 19.9649 12 20C5 20 1 12 1 12C2.24389 9.68192 3.96914 7.65663 6.06 6.06M9.9 4.24C10.5883 4.0789 11.2931 3.99836 12 4C19 4 23 12 23 12C22.393 13.1356 21.6691 14.2048 20.84 15.19M14.12 14.12C13.8454 14.4148 13.5141 14.6512 13.1462 14.8151C12.7782 14.9791 12.3809 15.0673 11.9781 15.0744C11.5753 15.0815 11.1752 15.0074 10.8016 14.8565C10.4281 14.7056 10.0887 14.4811 9.80385 14.1962C9.51897 13.9113 9.29439 13.5719 9.14351 13.1984C8.99262 12.8248 8.91853 12.4247 8.92563 12.0219C8.93274 11.6191 9.02091 11.2218 9.18488 10.8538C9.34884 10.4859 9.58525 10.1546 9.88 9.88" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <path d="M1 1L23 23" stroke={color} strokeWidth="1.75" strokeLinecap="round" />
      </svg>
    </View>
  );
}

export function ShieldIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M12 22C12 22 20 18 20 12V5L12 2L4 5V12C4 18 12 22 12 22Z" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </View>
  );
}

export function LogoutIcon({ size = D, color = C, className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M9 21H5C4.46957 21 3.96086 20.7893 3.58579 20.4142C3.21071 20.0391 3 19.5304 3 19V5C3 4.46957 3.21071 3.96086 3.58579 3.58579C3.96086 3.21071 4.46957 3 5 3H9" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <polyline points="16 17 21 12 16 7" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
        <line x1="21" y1="12" x2="9" y2="12" stroke={color} strokeWidth="1.75" strokeLinecap="round" />
      </svg>
    </View>
  );
}

export function LoadingSpinnerIcon({ size = D, color = "#FF6B9D", className }: IconProps) {
  return (
    <View className={className} style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none" style={{ animation: "spin 1s linear infinite" }}>
        <path d="M12 2V6M12 18V22M4.93 4.93L7.76 7.76M16.24 16.24L19.07 19.07M2 12H6M18 12H22M4.93 19.07L7.76 16.24M16.24 7.76L19.07 4.93" stroke={color} strokeWidth="2" strokeLinecap="round" />
      </svg>
    </View>
  );
}
