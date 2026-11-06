import * as React from 'react'
import { cn } from 'cn'
import { Button, type ButtonProps } from './Button'

export interface IconButtonProps extends Omit<ButtonProps, 'size'> {
  'aria-label': string
  tooltip?: string
  size?: 'sm' | 'md' | 'lg'
  icon?: React.ReactNode
}

const ICON_SIZE_MAP: Record<NonNullable<IconButtonProps['size']>, ButtonProps['size']> = {
  sm: 'icon-sm',
  md: 'icon',
  lg: 'icon-lg',
}

export function IconButton({
  'aria-label': ariaLabel,
  tooltip,
  size = 'md',
  variant = 'ghost',
  icon,
  children,
  className,
  ...props
}: IconButtonProps) {
  return (
    <Button
      type="button"
      variant={variant}
      size={ICON_SIZE_MAP[size]}
      aria-label={ariaLabel}
      title={tooltip ?? ariaLabel}
      data-tooltip={tooltip ?? ariaLabel}
      data-slot="icon-button"
      className={cn('p-0', className)}
      {...props}
    >
      {icon ?? children}
    </Button>
  )
}

export default IconButton
