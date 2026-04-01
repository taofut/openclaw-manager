/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // B端专业配色 - 企业蓝/深蓝灰
        brand: {
          50: '#f0f5ff',
          100: '#e0eafc',
          200: '#c7d9f5',
          300: '#a3c1ed',
          400: '#7aa3e0',
          500: '#5b8add',  // 主色 - 专业蓝
          600: '#3d71c7',
          700: '#2f5aa8',
          800: '#284a8c',
          900: '#243d73',
          950: '#162849',
        },
        // 深色主题背景 - 蓝灰色调
        dark: {
          950: '#0c0e14',
          900: '#12141c',
          800: '#1a1d28',
          700: '#242834',
          600: '#2e3340',
          500: '#3a4050',
          400: '#4a5060',
        },
        // 状态色 - 更稳重
        status: {
          success: '#22c55e',
          warning: '#f59e0b',
          error: '#ef4444',
          info: '#3b82f6',
        },
        // 旧版兼容
        claw: {
          50: '#fef3f2',
          100: '#fee4e2',
          200: '#ffccc7',
          300: '#ffa8a0',
          400: '#ff7a6b',
          500: '#5b8add',
          600: '#3d71c7',
          700: '#2f5aa8',
          800: '#284a8c',
          900: '#243d73',
          950: '#162849',
        },
        accent: {
          cyan: '#06b6d4',
          purple: '#8b5cf6',
          green: '#22c55e',
          amber: '#f59e0b',
        }
      },
      fontFamily: {
        sans: [
          'SF Pro Display',
          '-apple-system',
          'BlinkMacSystemFont',
          'PingFang SC',
          'Hiragino Sans GB',
          'Microsoft YaHei',
          'sans-serif',
        ],
        mono: [
          'SF Mono',
          'JetBrains Mono',
          'Fira Code',
          'Menlo',
          'monospace',
        ],
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'fade-in': 'fadeIn 0.2s ease-out',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
      },
      boxShadow: {
        'card': '0 1px 3px rgba(0, 0, 0, 0.3), 0 1px 2px rgba(0, 0, 0, 0.2)',
        'card-hover': '0 4px 12px rgba(0, 0, 0, 0.4)',
        'inner-light': 'inset 0 1px 0 0 rgba(255, 255, 255, 0.03)',
      },
      backdropBlur: {
        xs: '2px',
      },
    },
  },
  plugins: [],
}
