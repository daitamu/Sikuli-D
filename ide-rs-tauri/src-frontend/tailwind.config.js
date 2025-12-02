/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // SikuliX brand colors
        'sikuli': {
          50: '#f0f9ff',
          100: '#e0f2fe',
          200: '#bae6fd',
          300: '#7dd3fc',
          400: '#38bdf8',
          500: '#0ea5e9',
          600: '#0284c7',
          700: '#0369a1',
          800: '#075985',
          900: '#0c4a6e',
          950: '#082f49',
        },
        // Dark theme colors (Modern Zinc Palette)
        'dark': {
          'bg': '#18181b',      // zinc-900
          'surface': '#27272a', // zinc-800
          'sidebar': '#202022', // Slightly lighter than bg
          'border': '#3f3f46',  // zinc-700
          'text': '#e4e4e7',    // zinc-200
          'text-muted': '#a1a1aa', // zinc-400
          'hover': '#3f3f46',   // zinc-700
          'active': '#52525b',  // zinc-600
        }
      },
      fontFamily: {
        'mono': ['Consolas', 'Monaco', 'Courier New', 'monospace'],
      },
      screens: {
        // Custom breakpoints for better responsive control
        'xs': '480px',
        'sm': '640px',
        'md': '768px',
        'lg': '1024px',
        'xl': '1280px',
        '2xl': '1536px',
      },
    },
  },
  plugins: [],
}
