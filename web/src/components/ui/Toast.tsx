import * as React from 'react'
import { AlertCircle, AlertTriangle, CheckCircle2, Info, X } from 'lucide-react'
import { cn } from 'cn'

export type ToastVariant = 'info' | 'success' | 'warning' | 'error' | 'danger'

export interface ToastProps {
  id?: string
  title: string
  description?: string
  variant?: ToastVariant
  onDismiss?: () => void
  className?: string
}

const VARIANT_STYLES: Record<ToastVariant, string> = {
  info: 'border-[#1D4ED8]/30 bg-[#EFF6FF] text-[#1D4ED8]',
  success: 'border-[#15803D]/30 bg-[#F0FDF4] text-[#15803D]',
  warning: 'border-[#B45309]/30 bg-[#FFFBEB] text-[#B45309]',
  error: 'border-[#B91C1C]/30 bg-[#FEF2F2] text-[#B91C1C]',
  danger: 'border-[#B91C1C]/30 bg-[#FEF2F2] text-[#B91C1C]',
}

function VariantIcon({ variant }: { variant: ToastVariant }) {
  switch (variant) {
    case 'success':
      return <CheckCircle2 className="mt-0.5 size-4 shrink-0" aria-hidden="true" />
    case 'warning':
      return <AlertTriangle className="mt-0.5 size-4 shrink-0" aria-hidden="true" />
    case 'error':
    case 'danger':
      return <AlertCircle className="mt-0.5 size-4 shrink-0" aria-hidden="true" />
    default:
      return <Info className="mt-0.5 size-4 shrink-0" aria-hidden="true" />
  }
}

export function Toast({
  title,
  description,
  variant = 'info',
  onDismiss,
  className,
}: ToastProps) {
  const isAssertive = variant === 'error' || variant === 'danger'
  return (
    <div
      role={isAssertive ? 'alert' : 'status'}
      aria-live={isAssertive ? 'assertive' : 'polite'}
      data-slot="toast"
      data-variant={variant}
      className={cn(
        'flex w-full max-w-sm items-start gap-3 rounded-lg border p-3.5 text-sm shadow-sm',
        VARIANT_STYLES[variant],
        className,
      )}
    >
      <VariantIcon variant={variant} />
      <div className="min-w-0 flex-1">
        <p className="font-medium">{title}</p>
        {description && <p className="mt-0.5 text-xs opacity-90">{description}</p>}
      </div>
      {onDismiss && (
        <button
          type="button"
          aria-label="Dismiss notification"
          onClick={onDismiss}
          className="inline-flex size-5 items-center justify-center rounded hover:bg-black/10 focus-visible:outline-2"
        >
          <X className="size-3.5" aria-hidden="true" />
        </button>
      )}
    </div>
  )
}

export function ToastContainer({
  children,
  className,
}: {
  children: React.ReactNode
  className?: string
}) {
  return (
    <div
      aria-label="Notifications"
      data-slot="toast-container"
      className={cn('fixed right-4 bottom-4 z-50 flex flex-col gap-2', className)}
    >
      {children}
    </div>
  )
}

export default Toast
