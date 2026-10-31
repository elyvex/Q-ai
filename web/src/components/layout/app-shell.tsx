// Full-width application frame: a single top rail, one content measure,
// a quiet colophon. Navigation is horizontal because the product has five
// destinations — a sidebar would spend a third of a laptop screen on them.
import { useState } from 'react'
import { SiteHeader } from '@/components/layout/site-header'
import { CommandMenu } from '@/components/command/command-menu'

export function AppShell({
  currentPath,
  children,
}: {
  currentPath: string
  children: React.ReactNode
}) {
  const [open, setOpen] = useState(false)

  return (
    <div className="flex min-h-screen flex-col">
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:fixed focus:left-3 focus:top-3 focus:z-50 focus:rounded-md focus:border focus:bg-card focus:px-3 focus:py-2 focus:text-sm"
      >
        Skip to content
      </a>
      <SiteHeader currentPath={currentPath} onSearchClick={() => setOpen(true)} />
      <main id="main" className="flex-1">
        <div className="mx-auto w-full max-w-[86rem] px-4 py-8 sm:px-6 sm:py-10 lg:px-8 lg:py-12">
          {children}
        </div>
      </main>
      <footer className="border-t">
        <div className="mx-auto flex w-full max-w-[86rem] flex-wrap items-center gap-x-4 gap-y-1 px-4 py-4 font-mono text-[0.625rem] uppercase tracking-[0.14em] text-muted-foreground sm:px-6 lg:px-8">
          <span>Q-ai · Research workspace</span>
          <span className="ml-auto inline-flex items-center gap-1.5">
            <span className="prodot bg-canon" aria-hidden="true" />
            Local-first — canonical text is never altered
          </span>
        </div>
      </footer>
      <CommandMenu open={open} onOpenChange={setOpen} />
    </div>
  )
}
