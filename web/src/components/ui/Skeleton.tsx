import * as React from 'react'
import { cn } from 'cn'

export interface SkeletonProps extends React.ComponentProps<'div'> {
  lines?: number
  label?: string
}

export function Skeleton({
  lines = 1,
  label = 'Loading content',
  className,
  ...props
}: SkeletonProps) {
  if (lines <= 1) {
    return (
      <div
        role="status"
        aria-busy="true"
        aria-label={label}
        data-slot="skeleton"
        className={cn('h-4 w-full animate-pulse rounded-md bg-muted', className)}
        {...props}
      />
    )
  }

  return (
    <div
      role="status"
      aria-busy="true"
      aria-label={label}
      data-slot="skeleton-group"
      className={cn('space-y-2', className)}
      {...props}
    >
      {Array.from({ length: lines }).map((_, idx) => (
        <div
          key={idx}
          data-slot="skeleton"
          className={cn(
            'h-4 animate-pulse rounded-md bg-muted',
            idx === lines - 1 ? 'w-3/4' : 'w-full',
          )}
        />
      ))}
    </div>
  )
}

export default Skeleton
