/**
 * Design Token Exports (Phase 13-01)
 * Mirrors CSS custom properties in web/src/tokens/*.css for typed access
 * across components, views, and verification tests.
 */

export const COLORS = {
  canvas: {
    light: '#FFFFFF',
    dark: '#0B1020',
  },
  neutral: {
    100: '#F5F5F4',
    200: '#E7E5E4',
    300: '#D6D3D1',
    400: '#A8A29E',
    500: '#78716C',
    600: '#57534E',
    700: '#44403C',
    800: '#292524',
    900: '#1C1917',
  },
  accent: {
    default: '#0F766E',
    hover: '#115E59',
    active: '#134E4A',
    subtle: '#F0FDFA',
  },
  status: {
    success: '#15803D',
    warning: '#B45309',
    danger: '#B91C1C',
    info: '#1D4ED8',
  },
} as const

export const TYPOGRAPHY = {
  fonts: {
    arabic: "'Amiri', 'Scheherazade New', 'Noto Naskh Arabic', 'Geeza Pro', serif",
    sans: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
    mono: "'JetBrains Mono', 'Fira Code', monospace",
    serif: "'Newsreader Variable', 'Newsreader', Georgia, serif",
  },
  scale: {
    xs: 'clamp(0.72rem, 0.7rem + 0.1vw, 0.75rem)',
    sm: 'clamp(0.8125rem, 0.8rem + 0.12vw, 0.875rem)',
    base: 'clamp(0.9375rem, 0.9rem + 0.15vw, 1rem)',
    lg: 'clamp(1.0625rem, 1rem + 0.25vw, 1.125rem)',
    xl: 'clamp(1.25rem, 1.15rem + 0.4vw, 1.375rem)',
    '2xl': 'clamp(1.5rem, 1.35rem + 0.6vw, 1.75rem)',
    '3xl': 'clamp(1.875rem, 1.65rem + 0.9vw, 2.25rem)',
    arabicSm: '1.75rem',
    arabicMd: '2rem',
    arabicLg: '2.25rem',
  },
  lineHeight: {
    tight: 1.25,
    snug: 1.375,
    normal: 1.5,
    relaxed: 1.65,
    arabic: 2.15,
    arabicLg: 2.2,
  },
} as const

export const ELEVATION = {
  radii: {
    sm: '6px',
    md: '8px',
    lg: '10px',
    full: '9999px',
  },
  borders: {
    width: '1px',
    neutral: '1px solid #E7E5E4',
    strong: '1px solid #D6D3D1',
  },
  shadows: {
    none: 'none',
    card: '0 1px 2px 0 rgb(28 25 23 / 0.05)',
    cardHover: '0 2px 4px -1px rgb(28 25 23 / 0.08), 0 1px 2px -1px rgb(28 25 23 / 0.04)',
    popover: '0 4px 8px -2px rgb(28 25 23 / 0.08), 0 2px 4px -2px rgb(28 25 23 / 0.04)',
    modal: '0 10px 20px -5px rgb(28 25 23 / 0.12), 0 4px 6px -4px rgb(28 25 23 / 0.06)',
  },
} as const

export const MOTION = {
  durations: {
    fast: '100ms',
    normal: '150ms',
    slow: '250ms',
  },
  easings: {
    easeOut: 'cubic-bezier(0.16, 1, 0.3, 1)',
    easeInOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
    standard: 'cubic-bezier(0.2, 0, 0, 1)',
  },
} as const

export const LAYERS = {
  canonical: {
    cssPrefix: '--layer-canonical-',
    className: 'ayah-canonical',
    badgeClassName: 'layer-canonical',
    ruleStyle: 'solid',
    badgeStyle: 'solid',
    weight: 400,
  },
  translation: {
    cssPrefix: '--layer-translation-',
    className: 'ayah-translation',
    badgeClassName: 'layer-translation',
    ruleStyle: 'dashed',
    badgeStyle: 'outlined',
    weight: 400,
  },
  annotation: {
    cssPrefix: '--layer-annotation-',
    className: 'ayah-annotation',
    badgeClassName: 'layer-annotation',
    ruleStyle: 'double',
    badgeStyle: 'pill-dotted',
    weight: 500,
  },
} as const

export const TOKENS = {
  colors: COLORS,
  typography: TYPOGRAPHY,
  elevation: ELEVATION,
  motion: MOTION,
  layers: LAYERS,
} as const

export default TOKENS
