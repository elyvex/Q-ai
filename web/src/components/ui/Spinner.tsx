import * as React from 'react'
import { Loader2 } from 'lucide-react'
import { cn } from 'cn'

export interface SpinnerProps extends React.ComponentProps<'span'> {
  size?: 'sm' | 'md' | 'lg'
  label?: string
}

const SIZE_CLASSES: Record<NonNullable<SpinnerProps['size']>, string> = {
  sm: 'size-3.5',
  md: 'size-4',
  lg: 'size-6',
}

export function Spinner({
  size = 'md',
  label = 'Loading',
  className,
  ...props
}: SpinnerProps) {
  return (
    <span
      role="status"
      aria-label={label}
      data-slot="spinner"
      className={cn('inline-flex items-center justify-center', className)}
      {...props}
    >
      <Loader2 className={cn('animate-spin text-current', SIZE_CLASSES[size])} aria-hidden="true" />
      <span className="sr-only">{label}</span>
    </span>
  )
}

export default Spinner
