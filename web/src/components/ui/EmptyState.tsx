import * as React from 'react'
import { Inbox } from 'lucide-react'
import { cn } from 'cn'
import { Button } from './Button'

export interface EmptyStateProps extends React.ComponentProps<'div'> {
  title: string
  description?: string
  icon?: React.ReactNode
  actionLabel?: string
  onAction?: () => void
  action?: React.ReactNode
}

export function EmptyState({
  title,
  description,
  icon,
  actionLabel,
  onAction,
  action,
  children,
  className,
  ...props
}: EmptyStateProps) {
  return (
    <div
      data-slot="empty-state"
      className={cn(
        'flex flex-col items-start justify-center rounded-lg border border-dashed border-border bg-card p-6 text-left',
        className,
      )}
      {...props}
    >
      <div className="mb-3 inline-flex size-8 items-center justify-center rounded-md border bg-muted text-muted-foreground">
        {icon ?? <Inbox className="size-4" aria-hidden="true" />}
      </div>
      <h3 className="text-sm font-medium text-foreground">{title}</h3>
      {(description || children) && (
        <div className="mt-1 max-w-prose text-xs leading-relaxed text-muted-foreground">
          {description ?? children}
        </div>
      )}
      {(action || (actionLabel && onAction)) && (
        <div className="mt-4">
          {action ?? (
            <Button type="button" variant="outline" size="sm" onClick={onAction}>
              {actionLabel}
            </Button>
          )}
        </div>
      )}
    </div>
  )
}

export default EmptyState
