import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  // Served same-origin at the route root (D-03): the binary serves
  // web/dist at `/…` with an SPA fallback, so asset URLs must be absolute —
  // base './' resolved assets against nested routes (/read/…) and blanked them.
  base: '/',
  build: {
    outDir: 'dist',
  },
})
