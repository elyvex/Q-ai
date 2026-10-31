import { useEffect } from 'react'
import { TooltipProvider } from '@/components/ui/tooltip'
import { Toaster } from 'sonner'
import { ThemeProvider } from 'next-themes'
import Router from './router'
import './index.css'

// Q-ai Research Workspace
// Provenance-first research platform for Quranic and Islamic scholarship

export default function App() {
  const path =
    typeof window === 'undefined' ? '/' : window.location.pathname + window.location.search

  useEffect(() => {
    document.title = 'Q-ai | Research Workspace'
  }, [])

  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <TooltipProvider>
        <div className="min-h-screen bg-background font-sans antialiased">
          <Router path={path} />
          <Toaster position="bottom-right" richColors />
        </div>
      </TooltipProvider>
    </ThemeProvider>
  )
}
