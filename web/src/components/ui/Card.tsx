import * as React from 'react'
import { cn } from 'cn'

export interface CardProps extends React.ComponentProps<'div'> {
  interactive?: boolean
}

export function Card({ interactive = false, className, onClick, ...props }: CardProps) {
  const isClickable = Boolean(interactive || onClick)
  return (
    <div
      data-slot="card"
      data-interactive={isClickable ? 'true' : undefined}
      onClick={onClick}
      className={cn(
        'rounded-lg border border-border bg-card p-5 text-card-foreground shadow-xs transition-all duration-150',
        isClickable &&
          'cursor-pointer hover:-translate-y-0.5 hover:border-foreground/25 hover:shadow-sm',
        className,
      )}
      {...props}
    />
  )
}

export function CardHeader({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot="card-header"
      className={cn('mb-3 flex flex-col gap-1', className)}
      {...props}
    />
  )
}

export function CardTitle({ className, ...props }: React.ComponentProps<'h3'>) {
  return (
    <h3
      data-slot="card-title"
      className={cn('text-base leading-snug font-semibold text-foreground', className)}
      {...props}
    />
  )
}

export function CardDescription({ className, ...props }: React.ComponentProps<'p'>) {
  return (
    <p
      data-slot="card-description"
      className={cn('text-xs text-muted-foreground', className)}
      {...props}
    />
  )
}

export function CardContent({ className, ...props }: React.ComponentProps<'div'>) {
  return <div data-slot="card-content" className={cn('text-sm', className)} {...props} />
}

export function CardFooter({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot="card-footer"
      className={cn('mt-4 flex items-center justify-end gap-2 border-t pt-3', className)}
      {...props}
    />
  )
}

export default Card
