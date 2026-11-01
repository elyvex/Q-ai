import { useEffect, useState } from 'react'
import { Command, Menu, Moon, Sun, X } from 'lucide-react'
import { useTheme } from 'next-themes'

export const NAV = [
  { path: '/', label: 'Reader' },
  { path: '/search', label: 'Search' },
  { path: '/research', label: 'Research' },
  { path: '/hadith', label: 'Hadith' },
  { path: '/tafsir', label: 'Tafsir' },
  { path: '/scripture', label: 'Scripture' },
  { path: '/graph', label: 'Graph' },
  { path: '/rag-debug', label: 'RAG' },
  { path: '/explorer', label: 'Explorer' },
] as const

export function isActive(path: string, currentPath: string): boolean {
  if (path === '/') return currentPath === '/' || currentPath.startsWith('/read/')
  return currentPath.split('?', 1)[0] === path
}

export function SiteHeader({
  currentPath,
  onSearchClick,
}: {
  currentPath: string
  onSearchClick: () => void
}) {
  const { resolvedTheme, setTheme } = useTheme()
  const [mounted, setMounted] = useState(false)
  const [menuOpen, setMenuOpen] = useState(false)
  useEffect(() => setMounted(true), [])

  return (
    <header className="sticky top-0 z-30 border-b bg-background/85 backdrop-blur-md">
      <div className="mx-auto flex h-14 w-full max-w-[86rem] items-center gap-3 px-4 sm:px-6 lg:px-8">
        <a href="/" className="flex shrink-0 items-center gap-2.5" aria-label="Q-ai home">
          <span
            className="grid size-7 place-items-center rounded-[5px] bg-primary font-arabic text-[0.95rem] leading-none text-primary-foreground"
            aria-hidden="true"
          >
            ق
          </span>
          <span className="font-serif text-[1.0625rem] font-medium tracking-[-0.01em]">Q-ai</span>
          <span className="hidden font-mono text-[0.625rem] uppercase tracking-[0.16em] text-muted-foreground lg:inline">
            Concordance
          </span>
        </a>

        <nav aria-label="Primary" className="ml-3 hidden h-14 items-stretch md:flex">
          {NAV.map((item) => {
            const active = isActive(item.path, currentPath)
            return (
              <a
                key={item.path}
                href={item.path}
                aria-current={active ? 'page' : undefined}
                className={`relative flex items-center px-3 text-[0.8125rem] font-medium transition-colors ${
                  active
                    ? 'text-foreground'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                {item.label}
                {active && (
                  <span
                    className="absolute inset-x-2 bottom-0 h-[2px] rounded-full bg-primary"
                    aria-hidden="true"
                  />
                )}
              </a>
            )
          })}
        </nav>

        <div className="ml-auto flex items-center gap-1">
          <button
            type="button"
            onClick={onSearchClick}
            className="hidden h-8 items-center gap-2 rounded-md border bg-card px-2.5 text-xs text-muted-foreground transition-colors hover:text-foreground sm:inline-flex"
            aria-label="Open command menu"
          >
            <Command className="size-3" aria-hidden="true" />
            <span>Search</span>
            <kbd className="ml-1 rounded border bg-muted px-1 font-mono text-[0.6rem] leading-4 text-muted-foreground">
              ⌘K
            </kbd>
          </button>
          <button
            type="button"
            onClick={onSearchClick}
            className="btn btn-quiet size-8 px-0 sm:hidden"
            aria-label="Open command menu"
          >
            <Command className="size-4" aria-hidden="true" />
          </button>
          <button
            type="button"
            className="btn btn-quiet size-8 px-0"
            aria-label={
              mounted && resolvedTheme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'
            }
            onClick={() => setTheme(resolvedTheme === 'dark' ? 'light' : 'dark')}
          >
            {mounted && resolvedTheme === 'dark' ? (
              <Sun className="size-4" aria-hidden="true" />
            ) : (
              <Moon className="size-4" aria-hidden="true" />
            )}
          </button>
          <button
            type="button"
            className="btn btn-quiet size-8 px-0 md:hidden"
            aria-expanded={menuOpen}
            aria-controls="mobile-nav"
            aria-label={menuOpen ? 'Close navigation' : 'Open navigation'}
            onClick={() => setMenuOpen((o) => !o)}
          >
            {menuOpen ? (
              <X className="size-4" aria-hidden="true" />
            ) : (
              <Menu className="size-4" aria-hidden="true" />
            )}
          </button>
        </div>
      </div>

      {menuOpen && (
        <nav
          id="mobile-nav"
          aria-label="Primary"
          className="border-t bg-background md:hidden"
        >
          <div className="mx-auto w-full max-w-[86rem] px-2 py-2">
            {NAV.map((item) => {
              const active = isActive(item.path, currentPath)
              return (
                <a
                  key={item.path}
                  href={item.path}
                  aria-current={active ? 'page' : undefined}
                  onClick={() => setMenuOpen(false)}
                  className={`flex items-center justify-between rounded-md px-3 py-2.5 text-sm ${
                    active
                      ? 'bg-accent font-medium text-accent-foreground'
                      : 'text-muted-foreground hover:bg-muted hover:text-foreground'
                  }`}
                >
                  {item.label}
                  {active && <span className="prodot bg-primary" aria-hidden="true" />}
                </a>
              )
            })}
          </div>
        </nav>
      )}
    </header>
  )
}
