import { useState } from 'react'
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar'
import { AppSidebar } from '@/components/navigation/app-sidebar'
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
    <SidebarProvider defaultOpen={true}>
      <AppSidebar currentPath={currentPath} />
      <SidebarInset>
        <SiteHeader currentPath={currentPath} onSearchClick={() => setOpen(true)} />
        <main className="flex flex-1 flex-col">
          <div className="mx-auto w-full max-w-7xl flex-1 px-4 py-6 sm:px-6 lg:px-8">
            {children}
          </div>
        </main>
      </SidebarInset>
      <CommandMenu open={open} onOpenChange={setOpen} />
    </SidebarProvider>
  )
}
