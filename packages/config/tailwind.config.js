/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        // ── Nyx Starry Night palette ──────────────────────────────────────
        space: {
          950: "#030209",
          900: "#060412",
          850: "#0A0718",
          800: "#0D0A1E",
          750: "#100D28",
          700: "#13103A",
          650: "#181450",
          600: "#1C1845",
          550: "#21205A",
          500: "#26226B",
          450: "#302A80",
          400: "#3A3494",
        },
        // ── Dawn gradient palette ─────────────────────────────────────────
        dawn: {
          50: "#FFF5F8",
          100: "#FFE4EF",
          200: "#FFC8E0",
          300: "#FFA0C8",
          400: "#FF6B9D",
          500: "#FF4080",
          600: "#FF1A63",
          700: "#E8004A",
          800: "#C4003E",
          900: "#A00035",
        },
        // ── Coral / warm accent ───────────────────────────────────────────
        coral: {
          300: "#FFAE88",
          400: "#FF8C61",
          500: "#FF6A3A",
          600: "#E84E1E",
        },
        // ── Amber / golden dawn ───────────────────────────────────────────
        amber: {
          300: "#FFE97A",
          400: "#FFD93D",
          500: "#FFC800",
          600: "#E8B200",
        },
        // ── Star / text palette ───────────────────────────────────────────
        star: {
          50: "#FFFFFF",
          100: "#F0EBF8",
          200: "#E2D9F3",
          300: "#C4B5E8",
          400: "#A78BFA",
          500: "#818CF8",
          600: "#6B63CC",
          700: "#524A99",
          800: "#3A3466",
          900: "#251E44",
        },
        // ── Semantic aliases ──────────────────────────────────────────────
        nyx: {
          bg: "#060412",
          surface: "#0D0A1E",
          card: "#13103A",
          raised: "#1C1845",
          border: "#2A2460",
          "border-glow": "#4A3F9A",
          accent: "#FF6B9D",
          "accent-secondary": "#A78BFA",
          "text-primary": "#F0EBF8",
          "text-secondary": "#C4B5E8",
          "text-muted": "#7C6FA0",
          "text-disabled": "#4A3F6B",
        },
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        display: ["Inter", "system-ui", "sans-serif"],
      },
      backgroundImage: {
        "dawn-gradient":
          "linear-gradient(135deg, #FF6B9D 0%, #FF8C61 50%, #FFD93D 100%)",
        "dawn-radial":
          "radial-gradient(ellipse at top, #FF6B9D22 0%, transparent 70%)",
        "nebula-gradient":
          "linear-gradient(180deg, #060412 0%, #0D0A1E 50%, #1C1845 100%)",
        "card-gradient":
          "linear-gradient(145deg, #13103A 0%, #0D0A1E 100%)",
        "star-shimmer":
          "radial-gradient(ellipse at 20% 50%, #A78BFA18 0%, transparent 50%), radial-gradient(ellipse at 80% 20%, #FF6B9D12 0%, transparent 50%)",
      },
      boxShadow: {
        "dawn-glow": "0 0 20px rgba(255, 107, 157, 0.3)",
        "purple-glow": "0 0 20px rgba(167, 139, 250, 0.3)",
        "card-glow": "0 4px 24px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(42, 36, 96, 0.5)",
        "raised": "0 8px 32px rgba(0, 0, 0, 0.8)",
      },
      animation: {
        "star-twinkle": "twinkle 3s ease-in-out infinite",
        "dawn-pulse": "dawnPulse 4s ease-in-out infinite",
        "fade-in": "fadeIn 0.2s ease-out",
        "slide-up": "slideUp 0.3s ease-out",
        "slide-in-right": "slideInRight 0.3s ease-out",
      },
      keyframes: {
        twinkle: {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.3" },
        },
        dawnPulse: {
          "0%, 100%": { opacity: "0.6" },
          "50%": { opacity: "1" },
        },
        fadeIn: {
          from: { opacity: "0" },
          to: { opacity: "1" },
        },
        slideUp: {
          from: { transform: "translateY(8px)", opacity: "0" },
          to: { transform: "translateY(0)", opacity: "1" },
        },
        slideInRight: {
          from: { transform: "translateX(16px)", opacity: "0" },
          to: { transform: "translateX(0)", opacity: "1" },
        },
      },
      borderRadius: {
        "4xl": "2rem",
        "5xl": "2.5rem",
      },
    },
  },
  plugins: [],
};
