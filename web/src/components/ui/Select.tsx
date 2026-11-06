import * as React from 'react'
import { ChevronDown } from 'lucide-react'
import { cn } from 'cn'

export interface SelectOption {
  value: string
  label: string
  disabled?: boolean
}

export interface SelectProps extends React.ComponentProps<'select'> {
  options?: SelectOption[]
  label?: string
  invalid?: boolean
  errorText?: string
}

export function Select({
  options,
  label,
  invalid = false,
  errorText,
  id,
  className,
  children,
  'aria-invalid': ariaInvalid,
  'aria-describedby': ariaDescribedBy,
  ...props
}: SelectProps) {
  const generatedId = React.useId()
  const selectId = id ?? generatedId
  const errorId = `${selectId}-error`
  const isInvalid = Boolean(invalid || ariaInvalid === true || ariaInvalid === 'true' || errorText)

  return (
    <div className="flex w-full flex-col gap-1.5" data-slot="select-wrapper">
      {label && (
        <label htmlFor={selectId} className="text-xs font-medium text-foreground">
          {label}
        </label>
      )}
      <div className="relative flex items-center">
        <select
          id={selectId}
          data-slot="select"
          aria-invalid={isInvalid || undefined}
          aria-describedby={errorText ? cn(ariaDescribedBy, errorId) : ariaDescribedBy}
          className={cn(
            'h-9 w-full appearance-none rounded-lg border border-input bg-background pr-8 pl-3 text-sm text-foreground transition-colors outline-none focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40 disabled:pointer-events-none disabled:cursor-not-allowed disabled:bg-muted/60 disabled:opacity-50 aria-invalid:border-[#B91C1C] aria-invalid:ring-2 aria-invalid:ring-[#B91C1C]/20',
            className,
          )}
          {...props}
        >
          {options
            ? options.map((opt) => (
                <option key={opt.value} value={opt.value} disabled={opt.disabled}>
                  {opt.label}
                </option>
              ))
            : children}
        </select>
        <ChevronDown
          className="pointer-events-none absolute right-2.5 size-4 text-muted-foreground"
          aria-hidden="true"
        />
      </div>
      {errorText && (
        <p id={errorId} role="alert" className="text-xs text-[#B91C1C]">
          {errorText}
        </p>
      )}
    </div>
  )
}

export default Select
