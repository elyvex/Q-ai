// Canonical ayah slot (ADR-0112 / D-09): accepts ONLY canonical text +
// reference. There is deliberately no translation/annotation prop — the
// type system, not discipline, enforces layer separation. The optional
// size prop is presentational only (13-13): three explicit pairs inside the
// UI-SPEC §4 band (1.75→2.25rem) and the §15 Amiri band (2.15→2.2).
export interface CanonicalProps {
  arabicText: string
  reference: string
  size?: 'sm' | 'md' | 'lg'
}

const SIZE_CLASSES = {
  sm: 'text-[1.75rem] leading-[2.15]',
  md: 'text-[2rem] leading-[2.15]',
  lg: 'text-[2.25rem] leading-[2.2]',
} as const

export default function AyahColumn({ arabicText, reference, size = 'md' }: CanonicalProps) {
  return (
    <p
      className={`ayah-canonical my-2 ${SIZE_CLASSES[size]}`}
      dir="rtl"
      lang="ar"
      data-reference={reference}
      data-ayah-size={size}
    >
      {arabicText}
    </p>
  )
}
