import * as React from 'react'
import {
  BookOpen,
  Database,
  FlaskConical,
  Network,
  Search,
  Terminal,
} from 'lucide-react'

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from '@/components/ui/sidebar'

const GROUPS: {
  label: string
  items: { path: string; label: string; icon: React.ElementType }[]
}[] = [
  {
    label: 'Quran',
    items: [
      { path: '/', label: 'Reader', icon: BookOpen },
      { path: '/research', label: 'Research', icon: Search },
      { path: '/graph', label: 'Knowledge Graph', icon: Network },
    ],
  },
  {
    label: 'Data & Tools',
    items: [
      { path: '/rag-debug', label: 'RAG Debug', icon: Database },
      { path: '/explorer', label: 'API Explorer', icon: Terminal },
    ],
  },
]

function NavItem({
  path,
  label,
  icon: Icon,
  currentPath,
}: {
  path: string
  label: string
  icon: React.ElementType
  currentPath: string
}) {
  const active = path === '/' ? currentPath.startsWith('/read/') : currentPath === path
  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        isActive={active}
        tooltip={label}
        size="default"
        render={
          <a href={path} aria-current={active ? 'page' : undefined} className="group" />
        }
      >
        <Icon className="h-4 w-4" />
        <span>{label}</span>
      </SidebarMenuButton>
    </SidebarMenuItem>
  )
}

export function AppSidebar({
  currentPath,
  ...props
}: React.ComponentProps<typeof Sidebar> & { currentPath: string }) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader className="border-b border-sidebar-border">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              size="lg"
              render={<a href="/" className="group" />}
              className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            >
              <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80 text-primary-foreground shadow-sm">
                <FlaskConical className="size-4" />
              </div>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-semibold">Q-ai</span>
                <span className="truncate text-xs text-muted-foreground">
                  Research Workspace
                </span>
              </div>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent className="gap-0">
        {GROUPS.map((group) => (
          <SidebarGroup key={group.label} className="px-2 py-2">
            <SidebarGroupLabel className="px-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
              {group.label}
            </SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {group.items.map((item) => (
                  <NavItem key={item.path} currentPath={currentPath} {...item} />
                ))}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        ))}
      </SidebarContent>
      <SidebarFooter className="border-t border-sidebar-border">
        <div className="rounded-lg bg-sidebar-accent/50 p-2.5 text-[11px] leading-snug text-muted-foreground group-data-[collapsible=icon]:hidden">
          <div className="mb-1 flex items-center gap-1.5 font-medium text-foreground">
            <span className="relative flex h-1.5 w-1.5">
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500 opacity-75"></span>
              <span className="relative inline-flex h-1.5 w-1.5 rounded-full bg-emerald-500"></span>
            </span>
            Local-first
          </div>
          <div className="text-[10px]">Provenance-first • Canonical text preserved</div>
        </div>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
