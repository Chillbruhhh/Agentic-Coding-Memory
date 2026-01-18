/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: "#ef4444", // Industrial Red
        "primary-glow": "#b91c1c",
        "background-light": "#f8fafc",
        "background-dark": "#09090b", // Obsidian
        "panel-dark": "#18181b", // Charcoal
        "border-dark": "#27272a", // Metallic Grey
        "rust-accent": "#7f1d1d",
        "code-bg": "#0c0a09",
      },
      fontFamily: {
        display: ["Inter", "sans-serif"],
        mono: ["JetBrains Mono", "monospace"],
      },
      borderRadius: {
        DEFAULT: "0.25rem", // Sharper corners for industrial look
        'sm': "0.125rem",
      },
      boxShadow: {
        'neon-red': '0 0 10px rgba(239, 68, 68, 0.3), 0 0 20px rgba(239, 68, 68, 0.1)',
        'inner-red': 'inset 0 0 20px rgba(239, 68, 68, 0.05)',
      }
    },
  },
  plugins: [],
}
