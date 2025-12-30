/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        "./crates/frontend-web/src/**/*.rs",
        "./crates/frontend-web/index.html"
    ],
    darkMode: 'class',
    theme: {
        extend: {
            colors: {
                'void': '#030407',
                'surface': '#141419',
                'accent-glow': 'rgba(139, 92, 246, 0.8)',
            },
            backdropBlur: {
                'glass': '40px',
            },
            animation: {
                'spring-up': 'springUp 0.6s var(--ease-spring, cubic-bezier(0.175, 0.885, 0.32, 1.275)) forwards',
                'slide-in-left': 'slideInLeft 0.8s var(--ease-spring, cubic-bezier(0.175, 0.885, 0.32, 1.275)) forwards',
                'scale-in': 'scaleIn 0.4s var(--ease-spring, cubic-bezier(0.175, 0.885, 0.32, 1.275)) forwards',
                'fade-in': 'fadeIn 0.3s ease-out forwards',
                'pulse-glow': 'pulseGlow 2s ease-in-out infinite',
                'float': 'float 3s ease-in-out infinite',
            },
            keyframes: {
                springUp: {
                    '0%': { opacity: '0', transform: 'translateY(20px) scale(0.95)' },
                    '100%': { opacity: '1', transform: 'translateY(0) scale(1)' },
                },
                slideInLeft: {
                    '0%': { opacity: '0', transform: 'translateX(-30px)' },
                    '100%': { opacity: '1', transform: 'translateX(0)' },
                },
                scaleIn: {
                    '0%': { opacity: '0', transform: 'scale(0.9)' },
                    '100%': { opacity: '1', transform: 'scale(1)' },
                },
                fadeIn: {
                    '0%': { opacity: '0' },
                    '100%': { opacity: '1' },
                },
                pulseGlow: {
                    '0%, 100%': { boxShadow: '0 0 10px rgba(139, 92, 246, 0.8)', opacity: '1' },
                    '50%': { boxShadow: '0 0 20px rgba(139, 92, 246, 0.8)', opacity: '0.7' },
                },
                float: {
                    '0%, 100%': { transform: 'translateY(0)' },
                    '50%': { transform: 'translateY(-10px)' },
                }
            }
        }
    },
    plugins: []
}
