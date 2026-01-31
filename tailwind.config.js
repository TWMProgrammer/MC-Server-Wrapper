/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: [
    "./index.html",
    "./ui/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        background: 'var(--background)',
        surface: 'var(--surface)',
        primary: {
          DEFAULT: 'hsl(var(--primary))',
          hover: 'hsl(var(--primary-hover))',
          active: 'hsl(var(--primary-active))',
        },
        accent: {
          emerald: '#10b981',
          amber: '#f59e0b',
          rose: '#f43f5e',
          indigo: '#6366f1',
        },
        gray: {
          950: '#0a0a0c',
          900: '#16161a',
          800: '#212126',
          700: '#2d2d33',
          600: '#3e3e47',
          500: '#52525c',
          400: '#8c8c99',
          300: '#b0b0ba',
          200: '#d1d1d6',
          100: '#f0f0f5',
        }
      },
      borderRadius: {
        'xl': '1rem',
        '2xl': '1.5rem',
        '3xl': '2rem',
      },
      boxShadow: {
        'glow-primary': '0 0 15px -3px hsl(var(--primary) / 0.5)',
        'glow-emerald': '0 0 15px -3px rgba(16, 185, 129, 0.5)',
        'inner-light': 'inset 0 1px 0 0 rgba(255, 255, 255, 0.05)',
      }
    },
  },
  plugins: [],
}
