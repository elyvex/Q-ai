import * as React from 'react'
import { X } from 'lucide-react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from 'cn'

const badgeVariants = cva(
  'inline-flex items-center gap-1.5 rounded-md border px-2 py-0.5 text-xs font-medium whitespace-nowrap transition-colors',
  {
    variants: {
      variant: {
        default: 'border-border bg-muted text-foreground',
        neutral: 'border-[#E7E5E4] bg-[#F5F5F4] text-[#44403C]',
        primary: 'border-[#0F766E]/30 bg-[#F0FDFA] text-[#0F766E]',
        success: 'border-[#15803D]/30 bg-[#F0FDF4] text-[#15803D]',
        warning: 'border-[#B45309]/30 bg-[#FFFBEB] text-[#B45309]',
        danger: 'border-[#B91C1C]/30 bg-[#FEF2F2] text-[#B91C1C]',
        info: 'border-[#1D4ED8]/30 bg-[#EFF6FF] text-[#1D4ED8]',
        canonical: 'layer-canonical border-current bg-transparent font-mono uppercase tracking-wider',
        translation:
          'layer-translation border-dashed border-current bg-transparent font-mono uppercase tracking-wider',
        annotation:
          'layer-annotation border-dotted border-current bg-transparent font-mono uppercase tracking-wider',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
)

export interface BadgeProps
  extends React.ComponentProps<'span'>,
    VariantProps<typeof badgeVariants> {
  dot?: boolean
}

export function Badge({
  variant = 'default',
  dot = false,
  className,
  children,
  ...props
}: BadgeProps) {
  return (
    <span
      data-slot="badge"
      data-variant={variant}
      className={cn(badgeVariants({ variant, className }))}
      {...props}
    >
      {dot && <span className="size-1.5 rounded-full bg-current" aria-hidden="true" />}
      {children}
    </span>
  )
}

export interface ChipProps extends BadgeProps {
  onDismiss?: () => void
  dismissLabel?: string
}

export function Chip({
  onDismiss,
  dismissLabel = 'Remove',
  children,
  className,
  ...props
}: ChipProps) {
  return (
    <Badge data-slot="chip" className={cn('rounded-full px-2.5 py-0.5', className)} {...props}>
      <span>{children}</span>
      {onDismiss && (
        <button
          type="button"
          aria-label={dismissLabel}
          onClick={onDismiss}
          className="ml-0.5 inline-flex size-3.5 items-center justify-center rounded-full hover:bg-black/10 focus-visible:outline-2"
        >
          <X className="size-2.5" aria-hidden="true" />
        </button>
      )}
    </Badge>
  )
}

export { badgeVariants }
export default Badge
