import { useEffect } from 'react'
import { BookOpen, Database, GitBranch, Search, Terminal } from 'lucide-react'
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from '@/components/ui/command'

const navigation = [
  { name: 'Reader — open the codex', href: '/', icon: BookOpen },
  { name: 'Research — lexical instruments', href: '/research', icon: Search },
  { name: 'Knowledge Graph — follow the tesserae', href: '/graph', icon: GitBranch },
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
      <CommandInput placeholder="Turn to a leaf…" />
      <CommandList>
        <CommandEmpty>Nothing bound under that name.</CommandEmpty>
        <CommandGroup heading="Vellum">
          {navigation.map((item) => (
            <CommandItem key={item.href} onSelect={() => runCommand(item.href)}>
              <item.icon className="mr-2 size-3.5 text-gold" />
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
