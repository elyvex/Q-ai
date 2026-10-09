import { useState } from 'react'
import Router from './router'
import './tokens/layers.css'
import './tokens/themes.css'

// Q-ai reading UI shell: mounts the client-side router. Themes select via
// `data-theme` on the root (see tokens/themes.css).
const THEMES = ['light', 'dark', 'sepia', 'contrast'] as const

export default function App() {
  const [theme, setTheme] = useState<(typeof THEMES)[number]>('light')
  const path = typeof window === 'undefined' ? '/' : window.location.pathname
  return (
    <div data-theme={theme === 'light' ? undefined : theme}>
      <label>
        theme{' '}
        <select value={theme} onChange={(e) => setTheme(e.target.value as typeof theme)}>
          {THEMES.map((t) => (
            <option key={t} value={t}>
              {t}
            </option>
          ))}
        </select>
      </label>
      <Router path={path} />
    </div>
  )
}
