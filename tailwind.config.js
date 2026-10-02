/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Theme variables mapping
        'app-bg': 'rgb(var(--bg-app-rgb) / <alpha-value>)',
        'panel-bg': 'rgb(var(--bg-panel-rgb) / <alpha-value>)',
        'surface-bg': 'rgb(var(--bg-surface-rgb) / <alpha-value>)',
        'border-main': 'rgb(var(--border-main-rgb) / <alpha-value>)',
        'border-main-light': 'rgb(var(--border-main-light-rgb) / <alpha-value>)',
        'hover-bg': 'rgb(var(--bg-hover-rgb) / <alpha-value>)',

        // Brand
        'brand': {
          50: '#EEF2FF',
          100: '#E0E7FF',
          200: '#C7D2FE',
          300: '#A5B4FC',
          400: '#818CF8',
          500: '#6366F1',
          600: '#4F46E5',
          700: '#4338CA',
          800: '#3730A3',
          900: '#312E81',
          DEFAULT: 'rgb(var(--brand-rgb) / <alpha-value>)',
        },

        // Semantic colors
        'success': {
          50: '#ECFDF5',
          100: '#D1FAE5',
          400: '#34D399',
          500: '#10B981',
          600: '#059669',
          DEFAULT: '#10B981',
        },
        'warning': {
          50: '#FFFBEB',
          100: '#FEF3C7',
          400: '#FBBF24',
          500: '#F59E0B',
          600: '#D97706',
          DEFAULT: '#F59E0B',
        },
        'danger': {
          50: '#FEF2F2',
          100: '#FEE2E2',
          400: '#F87171',
          500: '#EF4444',
          600: '#DC2626',
          DEFAULT: '#EF4444',
        },

        // Text hierarchy
        'text-primary': 'rgb(var(--text-primary-rgb) / <alpha-value>)',
        'text-secondary': 'rgb(var(--text-secondary-rgb) / <alpha-value>)',
        'text-muted': 'rgb(var(--text-muted-rgb) / <alpha-value>)',
      },
      fontFamily: {
        'inter': ['"Inter"', 'system-ui', '-apple-system', 'sans-serif'],
      },
      borderRadius: {
        '4xl': '2rem',
      },
      fontSize: {
        'hero': 'clamp(3rem, 10vw, 12rem)',
      },
      letterSpacing: {
        'hero': '-0.05em',
      },
      backdropBlur: {
        xs: '2px',
      },
      animation: {
        'fade-in': 'fadeIn 0.3s ease-out',
        'slide-up': 'slideUp 0.3s ease-out',
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
      },
    },
  },
  plugins: [],
}
