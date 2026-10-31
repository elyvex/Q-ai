import { useEffect } from 'react'
import { BookOpen, Database, GitBranch, Network, Search, Terminal } from 'lucide-react'
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
  { name: 'Reader', href: '/', icon: BookOpen },
  { name: 'Research', href: '/research', icon: Search },
  { name: 'Knowledge Graph', href: '/graph', icon: Network },
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

  const runCommand = (command: () => unknown) => {
    onOpenChange(false)
    command()
  }

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <CommandInput placeholder="Type a command or search..." />
      <CommandList>
        <CommandEmpty>No results found.</CommandEmpty>
        <CommandGroup heading="Navigation">
          {navigation.map((item) => (
            <CommandItem
              key={item.href}
              onSelect={() => runCommand(() => (window.location.href = item.href))}
            >
              <item.icon className="mr-2 h-4 w-4" />
              <span>{item.name}</span>
            </CommandItem>
          ))}
        </CommandGroup>
        <CommandSeparator />
        <CommandGroup heading="Quick Actions">
          <CommandItem onSelect={() => runCommand(() => window.location.reload())}>
            <GitBranch className="mr-2 h-4 w-4" />
            <span>Reload Application</span>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  )
}
