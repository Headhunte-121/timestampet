/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
    "./src-tauri/src/**/*.rs"
  ],
  safelist: [
    'bg-[#FF6B00]',
    'bg-emerald-500',
    'bg-gray-600',
    'text-[#FF6B00]',
    'text-emerald-400',
    'scale-105',
    'opacity-100',
    'opacity-0',
    'translate-y-0',
    'translate-y-4',
    'border-[#FF6B00]',
    'ring-[#FF6B00]'
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'Segoe UI', 'Roboto', 'Helvetica', 'Arial', 'sans-serif'],
      },
      colors: {
        'brand-orange': '#FF6B00',
        'brand-light': '#FF944D',
        'cinema-black': '#0D0F14',
        'surface-gray': '#1F222A',
        'muted': '#8E929C',
      },
    },
  },
  plugins: [require("tailwind-scrollbar-hide")],
}