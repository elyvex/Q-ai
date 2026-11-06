import * as React from 'react'
import { Button as ButtonPrimitive } from '@base-ui/react/button'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from 'cn'
import { Spinner } from './Spinner'

const buttonVariants = cva(
  "group/button inline-flex shrink-0 items-center justify-center rounded-lg border border-transparent bg-clip-padding text-sm font-medium whitespace-nowrap transition-all duration-150 outline-none select-none hover:-translate-y-px active:scale-[0.98] focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-2 aria-invalid:ring-destructive/20 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90',
        primary: 'bg-[#0F766E] text-white border-[#0F766E] hover:bg-[#115E59]',
        secondary:
          'bg-secondary text-secondary-foreground border-border hover:bg-muted',
        outline:
          'border-border bg-background text-foreground hover:bg-muted hover:text-foreground',
        ghost:
          'bg-transparent text-foreground hover:bg-muted hover:text-foreground',
        danger:
          'bg-[#B91C1C] text-white border-[#B91C1C] hover:bg-[#991B1B] focus-visible:ring-[#B91C1C]',
        destructive:
          'bg-destructive/10 text-destructive hover:bg-destructive/20 focus-visible:border-destructive/40 focus-visible:ring-destructive/20',
        link: 'text-primary underline-offset-4 hover:underline hover:translate-y-0 active:scale-100',
      },
      size: {
        default: 'h-9 gap-1.5 px-3.5 text-sm',
        xs: 'h-6 gap-1 rounded-md px-2 text-xs',
        sm: 'h-8 gap-1.5 rounded-md px-2.5 text-xs',
        md: 'h-9 gap-2 rounded-lg px-3.5 text-sm',
        lg: 'h-10 gap-2 rounded-lg px-4 text-base',
        xl: 'h-12 gap-2.5 rounded-xl px-6 text-base',
        icon: 'size-8',
        'icon-xs': 'size-6 rounded-md',
        'icon-sm': 'size-7 rounded-md',
        'icon-lg': 'size-9 rounded-lg',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)

export type ButtonProps = ButtonPrimitive.Props &
  VariantProps<typeof buttonVariants> & {
    loading?: boolean
    loadingLabel?: string
  }

function Button({
  className,
  variant = 'default',
  size = 'default',
  loading = false,
  loadingLabel = 'Loading',
  disabled,
  children,
  ...props
}: ButtonProps) {
  const isDisabled = Boolean(disabled || loading)
  return (
    <ButtonPrimitive
      data-slot="button"
      data-variant={variant}
      data-size={size}
      data-loading={loading ? 'true' : undefined}
      aria-busy={loading || undefined}
      disabled={isDisabled}
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    >
      {loading && <Spinner size="sm" label={loadingLabel} className="mr-1.5" />}
      {children}
    </ButtonPrimitive>
  )
}

export { Button, buttonVariants }
export default Button
