// Canonical ayah slot (ADR-0112 / D-09): accepts ONLY canonical text +
// reference. There is deliberately no translation/annotation prop — the
// type system, not discipline, enforces layer separation.
export interface CanonicalProps {
  arabicText: string
  reference: string
}

export default function AyahColumn({ arabicText, reference }: CanonicalProps) {
  return (
    <p className="ayah-canonical" dir="rtl" lang="ar" data-reference={reference}>
      {arabicText}
    </p>
  )
}
