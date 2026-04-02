/**
 * Nyx design tokens — starry night + dawn palette.
 * Use NativeWind className for all styling. These are JavaScript-accessible constants.
 */

export const Colors = {
  // ── Space backgrounds ─────────────────────────────────────────────────────
  space950: "#030209",
  space900: "#060412",
  space800: "#0D0A1E",
  space700: "#13103A",
  space600: "#1C1845",
  space500: "#26226B",

  // ── Dawn accent ───────────────────────────────────────────────────────────
  dawn400: "#FF6B9D",
  dawn500: "#FF4080",
  coral400: "#FF8C61",
  amber400: "#FFD93D",

  // ── Star / text ───────────────────────────────────────────────────────────
  star50: "#FFFFFF",
  star100: "#F0EBF8",
  star200: "#E2D9F3",
  star300: "#C4B5E8",
  star400: "#A78BFA",
  star500: "#818CF8",
  star700: "#524A99",
  star800: "#3A3466",

  // ── Semantic ──────────────────────────────────────────────────────────────
  bg: "#060412",
  surface: "#0D0A1E",
  card: "#13103A",
  raised: "#1C1845",
  border: "#2A2460",
  borderGlow: "#4A3F9A",
  accent: "#FF6B9D",
  accentSecondary: "#A78BFA",
  textPrimary: "#F0EBF8",
  textSecondary: "#C4B5E8",
  textMuted: "#7C6FA0",
  textDisabled: "#4A3F6B",
  success: "#34D399",
  error: "#F87171",
  warning: "#FBBF24",
} as const;

export const Typography = {
  fontSizeXs: 11,
  fontSizeSm: 13,
  fontSizeBase: 15,
  fontSizeMd: 16,
  fontSizeLg: 18,
  fontSizeXl: 20,
  fontSize2xl: 24,
  fontSize3xl: 30,
  fontSize4xl: 36,

  fontWeightRegular: "400" as const,
  fontWeightMedium: "500" as const,
  fontWeightSemibold: "600" as const,
  fontWeightBold: "700" as const,
  fontWeightExtrabold: "800" as const,

  lineHeightTight: 1.2,
  lineHeightSnug: 1.4,
  lineHeightNormal: 1.5,
  lineHeightRelaxed: 1.625,
} as const;

export const Spacing = {
  xs: 4,
  sm: 8,
  md: 12,
  base: 16,
  lg: 20,
  xl: 24,
  "2xl": 32,
  "3xl": 40,
  "4xl": 48,
  "5xl": 64,
} as const;

export const BorderRadius = {
  sm: 4,
  md: 8,
  lg: 12,
  xl: 16,
  "2xl": 20,
  "3xl": 24,
  full: 9999,
} as const;

export const Shadow = {
  card: {
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.6,
    shadowRadius: 24,
    elevation: 8,
  },
  dawnGlow: {
    shadowColor: "#FF6B9D",
    shadowOffset: { width: 0, height: 0 },
    shadowOpacity: 0.4,
    shadowRadius: 16,
    elevation: 4,
  },
} as const;
