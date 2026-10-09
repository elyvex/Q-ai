import react from '@vitejs/plugin-react'
import { defineConfig } from 'vitest/config'

// Component tests: jsdom + the existing Vite/TS setup (05-04 toolchain).
export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.{ts,tsx}'],
  },
})
