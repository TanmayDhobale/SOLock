import type { Config } from "tailwindcss";

export default {
    darkMode: "class",
    content: [
        "./pages/**/*.{js,ts,jsx,tsx,mdx}",
        "./components/**/*.{js,ts,jsx,tsx,mdx}",
        "./app/**/*.{js,ts,jsx,tsx,mdx}",
    ],
    theme: {
        extend: {
            colors: {
                background: "hsl(240 10% 3.9%)",
                foreground: "hsl(0 0% 98%)",
                card: {
                    DEFAULT: "hsl(240 10% 3.9%)",
                    foreground: "hsl(0 0% 98%)",
                },
                popover: {
                    DEFAULT: "hsl(240 10% 3.9%)",
                    foreground: "hsl(0 0% 98%)",
                },
                primary: {
                    DEFAULT: "hsl(142 76% 36%)",
                    foreground: "hsl(0 0% 9%)",
                },
                secondary: {
                    DEFAULT: "hsl(240 3.7% 15.9%)",
                    foreground: "hsl(0 0% 98%)",
                },
                muted: {
                    DEFAULT: "hsl(240 3.7% 15.9%)",
                    foreground: "hsl(240 5% 64.9%)",
                },
                accent: {
                    DEFAULT: "hsl(269 70% 65%)",
                    foreground: "hsl(0 0% 98%)",
                },
                destructive: {
                    DEFAULT: "hsl(0 62.8% 30.6%)",
                    foreground: "hsl(0 0% 98%)",
                },
                border: "hsl(240 3.7% 15.9%)",
                input: "hsl(240 3.7% 15.9%)",
                ring: "hsl(142 76% 36%)",
            },
            borderRadius: {
                lg: "var(--radius)",
                md: "calc(var(--radius) - 2px)",
                sm: "calc(var(--radius) - 4px)",
            },
            fontFamily: {
                sans: ["var(--font-geist-sans)"],
                mono: ["var(--font-geist-mono)"],
            },
            animation: {
                "fade-in": "fadeIn 0.5s ease-in-out",
                "slide-up": "slideUp 0.5s ease-out",
                "pulse-glow": "pulseGlow 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
            },
            keyframes: {
                fadeIn: {
                    "0%": { opacity: "0" },
                    "100%": { opacity: "1" },
                },
                slideUp: {
                    "0%": { transform: "translateY(20px)", opacity: "0" },
                    "100%": { transform: "translateY(0)", opacity: "1" },
                },
                pulseGlow: {
                    "0%, 100%": {
                        opacity: "1",
                        boxShadow: "0 0 20px rgba(20, 241, 149, 0.3)",
                    },
                    "50%": {
                        opacity: "0.8",
                        boxShadow: "0 0 40px rgba(20, 241, 149, 0.6)",
                    },
                },
            },
            backgroundImage: {
                "gradient-radial": "radial-gradient(var(--tw-gradient-stops))",
                "gradient-conic":
                    "conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))",
            },
        },
    },
    plugins: [],
} satisfies Config;
