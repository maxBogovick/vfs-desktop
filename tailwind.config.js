/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Tahoma', 'Arial', 'sans-serif'],
        mono: ['Consolas', 'Monaco', 'Courier New', 'monospace'],
      },
      colors: {
        // Theme colors are defined via CSS variables in themes.css
        // This allows dynamic theme switching
        'vf': {
          'bg-primary': 'var(--vf-bg-primary)',
          'bg-secondary': 'var(--vf-bg-secondary)',
          'bg-tertiary': 'var(--vf-bg-tertiary)',
          'surface-default': 'var(--vf-surface-default)',
          'surface-hover': 'var(--vf-surface-hover)',
          'surface-selected': 'var(--vf-surface-selected)',
          'surface-active': 'var(--vf-surface-active)',
          'text-primary': 'var(--vf-text-primary)',
          'text-secondary': 'var(--vf-text-secondary)',
          'text-tertiary': 'var(--vf-text-tertiary)',
          'text-disabled': 'var(--vf-text-disabled)',
          'accent-primary': 'var(--vf-accent-primary)',
          'accent-hover': 'var(--vf-accent-hover)',
          'accent-active': 'var(--vf-accent-active)',
          'border-default': 'var(--vf-border-default)',
          'border-subtle': 'var(--vf-border-subtle)',
          'border-accent': 'var(--vf-border-accent)',
        },
      },
      boxShadow: {
        'vf-sm': '0 1px 2px var(--vf-shadow-sm)',
        'vf-md': '0 4px 6px var(--vf-shadow-md)',
        'vf-lg': '0 10px 15px var(--vf-shadow-lg)',
      },
    },
  },
  plugins: [],
}
