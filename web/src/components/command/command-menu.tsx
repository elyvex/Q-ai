import { useEffect } from 'react'
import { BookOpen, BookMarked, Database, GitBranch, Search, ShieldCheck, Terminal } from 'lucide-react'
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from '@/components/ui/command'

export const NAVIGATION = [
  { name: 'Reader', href: '/', icon: BookOpen },
  { name: 'Search', href: '/search', icon: Search },
  { name: 'Research', href: '/research', icon: Search },
  { name: 'Words & Roots', href: '/words', icon: Search },
  { name: 'Research Tools', href: '/tools', icon: Terminal },
  { name: 'Citations', href: '/citations', icon: ShieldCheck },
  { name: 'Hadith', href: '/hadith', icon: BookMarked },
  { name: 'Tafsir', href: '/tafsir', icon: BookOpen },
  { name: 'Scripture', href: '/scripture', icon: BookOpen },
  { name: 'Knowledge Graph', href: '/graph', icon: GitBranch },
  { name: 'RAG Debug', href: '/rag-debug', icon: Database },
  { name: 'API Explorer', href: '/explorer', icon: Terminal },
]

export function CommandMenu({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        onOpenChange(!open)
      }
    }
    document.addEventListener('keydown', down)
    return () => document.removeEventListener('keydown', down)
  }, [open, onOpenChange])

  const runCommand = (href: string) => {
    onOpenChange(false)
    window.location.href = href
  }

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <CommandInput placeholder="Go to a page or reference…" />
      <CommandList>
        <CommandEmpty>Nothing matches that name.</CommandEmpty>
        <CommandGroup heading="Pages">
          {NAVIGATION.map((item) => (
            <CommandItem key={item.href} onSelect={() => runCommand(item.href)}>
              <item.icon className="mr-2 size-3.5 text-muted-foreground" />
              <span>{item.name}</span>
            </CommandItem>
          ))}
        </CommandGroup>
        <CommandSeparator />
        <CommandGroup heading="Reading deep links">
          <CommandItem onSelect={() => runCommand('/read/test@0.1.0/1:1')}>
            <span className="font-mono text-xs">/read/test@0.1.0/1:1</span>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  )
}
