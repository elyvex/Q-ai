// Translation panel: attributed translation text under the translation
// layer token. Renders as text — never injected as HTML (T-05-15).
export interface Translation {
  translator: string
  text: string
}

export default function TranslationPanel({ translator, text }: Translation) {
  return (
    <aside className="ayah-translation border-l-2 border-trans/40 pl-3" data-translator={translator}>
      <span className="mb-0.5 block font-mono text-[0.6rem] uppercase tracking-[0.14em] text-trans">
        {translator}
      </span>
      <span className="text-[0.9375rem] leading-relaxed">{text}</span>
    </aside>
  )
}
