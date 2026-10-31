import { Search } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { SidebarTrigger } from '@/components/ui/sidebar'
import { cn } from '@/lib/utils'

export function SiteHeader({
  currentPath,
  onSearchClick,
}: {
  currentPath: string
  onSearchClick?: () => void
}) {
  const getBreadcrumb = () => {
    if (currentPath.startsWith('/read/')) return ['Quran', 'Reader']
    if (currentPath === '/research') return ['Research', 'Workspace']
    if (currentPath === '/graph') return ['Research', 'Knowledge Graph']
    if (currentPath === '/rag-debug') return ['System', 'RAG Debug']
    if (currentPath === '/explorer') return ['System', 'API Explorer']
    return ['Quran', 'Reader']
  }

  const crumbs = getBreadcrumb()

  return (
    <header className="sticky top-0 z-10 flex h-14 items-center gap-4 border-b bg-background/95 px-4 backdrop-blur supports-[backdrop-filter]:bg-background/60 sm:px-6">
      <div className="flex items-center gap-2">
        <SidebarTrigger className="-ml-1" />
      </div>
      
      <nav className="flex flex-1 items-center gap-1 text-sm">
        {crumbs.map((crumb, i) => (
          <div key={i} className="flex items-center gap-1">
            {i > 0 && <span className="text-muted-foreground">/</span>}
            <span className={cn(
              "px-1.5 py-0.5",
              i === crumbs.length - 1 ? "font-medium text-foreground" : "text-muted-foreground"
            )}>
              {crumb}
            </span>
          </div>
        ))}
      </nav>

      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          className="hidden h-8 w-56 items-center justify-between text-muted-foreground sm:flex"
          onClick={onSearchClick}
        >
          <div className="flex items-center gap-2">
            <Search className="h-3.5 w-3.5" />
            <span className="text-xs">Search...</span>
          </div>
          <kbd className="pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground">
            <span className="text-xs">⌘</span>K
          </kbd>
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 sm:hidden"
          onClick={onSearchClick}
        >
          <Search className="h-4 w-4" />
        </Button>
      </div>
    </header>
  )
}
