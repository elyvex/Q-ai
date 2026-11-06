import * as React from 'react'
import { AlertCircle, RotateCcw } from 'lucide-react'
import { cn } from 'cn'
import { Button } from './Button'

export interface ErrorStateProps extends React.ComponentProps<'div'> {
  title?: string
  message: string
  code?: string
  remedy?: string
  retryLabel?: string
  onRetry?: () => void
  action?: React.ReactNode
}

export function ErrorState({
  title = 'Unable to complete request',
  message,
  code,
  remedy,
  retryLabel = 'Retry',
  onRetry,
  action,
  className,
  ...props
}: ErrorStateProps) {
  return (
    <div
      role="alert"
      data-slot="error-state"
      className={cn(
        'rounded-lg border border-[#B91C1C]/30 bg-[#FEF2F2] p-5 text-[#1C1917] dark:bg-[#B91C1C]/10 dark:text-[#F5F5F4]',
        className,
      )}
      {...props}
    >
      <div className="flex items-start gap-3">
        <AlertCircle className="mt-0.5 size-5 shrink-0 text-[#B91C1C]" aria-hidden="true" />
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="text-sm font-semibold">{title}</h3>
            {code && (
              <span className="rounded border border-[#B91C1C]/30 px-1.5 py-0.5 font-mono text-[0.7rem] text-[#B91C1C]">
                {code}
              </span>
            )}
          </div>
          <p className="mt-1 text-xs leading-relaxed opacity-90">{message}</p>
          {remedy && (
            <p className="mt-2 font-mono text-xs opacity-80">Remedy: {remedy}</p>
          )}
          {(action || onRetry) && (
            <div className="mt-3 flex items-center gap-2">
              {action ?? (
                <Button type="button" variant="outline" size="sm" onClick={onRetry}>
                  <RotateCcw className="mr-1 size-3.5" aria-hidden="true" />
                  {retryLabel}
                </Button>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default ErrorState
