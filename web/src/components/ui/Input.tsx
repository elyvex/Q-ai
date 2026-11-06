import * as React from 'react'
import { Input as InputPrimitive } from '@base-ui/react/input'
import { cn } from 'cn'

export interface InputProps extends React.ComponentProps<'input'> {
  invalid?: boolean
  errorText?: string
  label?: string
}

function Input({
  className,
  type = 'text',
  invalid = false,
  errorText,
  label,
  id,
  'aria-invalid': ariaInvalid,
  'aria-describedby': ariaDescribedBy,
  ...props
}: InputProps) {
  const generatedId = React.useId()
  const inputId = id ?? generatedId
  const errorId = `${inputId}-error`
  const isInvalid = Boolean(invalid || ariaInvalid === true || ariaInvalid === 'true' || errorText)

  const inputElement = (
    <InputPrimitive
      id={inputId}
      type={type}
      data-slot="input"
      aria-invalid={isInvalid || undefined}
      aria-describedby={errorText ? cn(ariaDescribedBy, errorId) : ariaDescribedBy}
      className={cn(
        'h-9 w-full min-w-0 rounded-lg border border-input bg-transparent px-3 py-1.5 text-sm transition-colors outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40 disabled:pointer-events-none disabled:cursor-not-allowed disabled:bg-muted/60 disabled:opacity-50 aria-invalid:border-[#B91C1C] aria-invalid:ring-2 aria-invalid:ring-[#B91C1C]/20',
        className,
      )}
      {...props}
    />
  )

  if (!label && !errorText) {
    return inputElement
  }

  return (
    <div className="flex w-full flex-col gap-1.5" data-slot="input-wrapper">
      {label && (
        <label htmlFor={inputId} className="text-xs font-medium text-foreground">
          {label}
        </label>
      )}
      {inputElement}
      {errorText && (
        <p id={errorId} role="alert" className="text-xs text-[#B91C1C]">
          {errorText}
        </p>
      )}
    </div>
  )
}

export interface TextAreaProps extends React.ComponentProps<'textarea'> {
  invalid?: boolean
  errorText?: string
  label?: string
}

function TextArea({
  className,
  invalid = false,
  errorText,
  label,
  id,
  'aria-invalid': ariaInvalid,
  'aria-describedby': ariaDescribedBy,
  ...props
}: TextAreaProps) {
  const generatedId = React.useId()
  const textareaId = id ?? generatedId
  const errorId = `${textareaId}-error`
  const isInvalid = Boolean(invalid || ariaInvalid === true || ariaInvalid === 'true' || errorText)

  return (
    <div className="flex w-full flex-col gap-1.5" data-slot="textarea-wrapper">
      {label && (
        <label htmlFor={textareaId} className="text-xs font-medium text-foreground">
          {label}
        </label>
      )}
      <textarea
        id={textareaId}
        data-slot="textarea"
        aria-invalid={isInvalid || undefined}
        aria-describedby={errorText ? cn(ariaDescribedBy, errorId) : ariaDescribedBy}
        className={cn(
          'min-h-20 w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm transition-colors outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40 disabled:pointer-events-none disabled:cursor-not-allowed disabled:bg-muted/60 disabled:opacity-50 aria-invalid:border-[#B91C1C] aria-invalid:ring-2 aria-invalid:ring-[#B91C1C]/20',
          className,
        )}
        {...props}
      />
      {errorText && (
        <p id={errorId} role="alert" className="text-xs text-[#B91C1C]">
          {errorText}
        </p>
      )}
    </div>
  )
}

export { Input, TextArea }
export default Input
