import * as React from 'react'
import { cn } from 'cn'

export interface AnimatedBackgroundProps extends React.ComponentProps<'div'> {
  maxOpacity?: number
}

/**
 * Pure CSS animated gradient field positioned behind content (`z-index: -1`,
 * `pointer-events: none`). Opacity is clamped to <= 0.15 so foreground text
 * contrast meets WCAG 2.2 AA. Automatically pauses when the document is hidden
 * and disables motion completely when `prefers-reduced-motion: reduce` matches.
 */
export function AnimatedBackground({
  maxOpacity = 0.12,
  className,
  style,
  ...props
}: AnimatedBackgroundProps) {
  const clampedOpacity = Math.min(Math.max(maxOpacity, 0), 0.15)
  const [paused, setPaused] = React.useState<boolean>(() =>
    typeof document !== 'undefined' ? document.visibilityState === 'hidden' : false,
  )
  const [reducedMotion, setReducedMotion] = React.useState<boolean>(() => {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
      return false
    }
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches
  })

  React.useEffect(() => {
    function onVisibilityChange() {
      setPaused(document.visibilityState === 'hidden')
    }
    document.addEventListener('visibilitychange', onVisibilityChange)

    let mediaQuery: MediaQueryList | null = null
    function onMotionChange(event: MediaQueryListEvent) {
      setReducedMotion(event.matches)
    }
    if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
      mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
      setReducedMotion(mediaQuery.matches)
      if (typeof mediaQuery.addEventListener === 'function') {
        mediaQuery.addEventListener('change', onMotionChange)
      }
    }

    return () => {
      document.removeEventListener('visibilitychange', onVisibilityChange)
      if (mediaQuery && typeof mediaQuery.removeEventListener === 'function') {
        mediaQuery.removeEventListener('change', onMotionChange)
      }
    }
  }, [])

  const isAnimationActive = !paused && !reducedMotion

  return (
    <div
      aria-hidden="true"
      data-slot="animated-background"
      data-paused={paused ? 'true' : 'false'}
      data-reduced-motion={reducedMotion ? 'true' : 'false'}
      style={{
        zIndex: -1,
        pointerEvents: 'none',
        opacity: clampedOpacity,
        animationPlayState: isAnimationActive ? 'running' : 'paused',
        animationDuration: reducedMotion ? '0ms' : '24s',
        backgroundImage:
          'radial-gradient(circle at 20% 20%, rgb(15 118 110 / 0.14), transparent 55%), radial-gradient(circle at 80% 75%, rgb(120 113 108 / 0.12), transparent 60%)',
        ...style,
      }}
      className={cn('fixed inset-0 overflow-hidden select-none', className)}
      {...props}
    />
  )
}

export default AnimatedBackground
