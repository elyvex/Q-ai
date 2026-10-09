import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  // Relative asset URLs: the built SPA is embedded into the `qai` binary
  // and served same-origin (D-03), so absolute `/assets/…` paths would break.
  base: './',
  build: {
    outDir: 'dist',
  },
})
