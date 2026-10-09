// Layer badge: the visible layer marker. The class always matches the
// layer, so a translation can never wear the canonical token (D-09).
export type Layer = 'canonical' | 'translation' | 'annotation'

const CLASS: Record<Layer, string> = {
  canonical: 'layer-canonical',
  translation: 'layer-translation',
  annotation: 'layer-annotation',
}

export default function LayerBadge({ layer, label }: { layer: Layer; label: string }) {
  return <span className={CLASS[layer]} data-layer={layer}>{label}</span>
}
