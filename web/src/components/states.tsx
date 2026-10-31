// Shared empty / error / loading states. They are deliberately typographic:
// the interface says what happened and what to do, never decorates absence.
import { AlertCircle, Loader2 } from 'lucide-react'

export function EmptyState({
  icon: Icon,
  title,
  children,
  className = '',
}: {
  icon?: React.ElementType
  title: React.ReactNode
  children?: React.ReactNode
  className?: string
}) {
  return (
    <div
      className={`flex flex-col items-center justify-center rounded-lg border bg-muted/30 px-6 py-12 text-center ${className}`}
    >
      {Icon !== undefined && <Icon className="mb-3 size-5 text-muted-foreground/60" aria-hidden="true" />}
      <p className="font-mono text-[0.625rem] uppercase tracking-[0.14em] text-muted-foreground">
        {title}
      </p>
      {children !== undefined && (
        <p className="mt-2 max-w-sm text-[0.8125rem] leading-relaxed text-foreground/80">{children}</p>
      )}
    </div>
  )
}

export function ErrorNote({ children }: { children: React.ReactNode }) {
  return (
    <p
      role="alert"
      className="mt-4 flex items-start gap-2 rounded-md border border-destructive/35 bg-destructive/[0.06] px-3.5 py-2.5 text-[0.8125rem] text-destructive"
    >
      <AlertCircle className="mt-0.5 size-3.5 shrink-0" aria-hidden="true" />
      <span>{children}</span>
    </p>
  )
}

export function LoadingLine({ label }: { label: string }) {
  return (
    <p role="status" className="flex items-center gap-2 text-[0.8125rem] text-muted-foreground">
      <Loader2 className="size-3.5 animate-spin" aria-hidden="true" />
      {label}
    </p>
  )
}
