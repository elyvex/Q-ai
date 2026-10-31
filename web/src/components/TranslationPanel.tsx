// Translation panel: attributed translation text under the translation
// layer token. Renders as text — never injected as HTML (T-05-15).
export interface Translation {
  translator: string
  text: string
}

export default function TranslationPanel({ translator, text }: Translation) {
  return (
    <aside className="ayah-translation mt-3 rounded-lg px-3 py-2 text-sm/relaxed" data-translator={translator}>
      <span className="mb-0.5 block text-[0.65rem] font-semibold uppercase tracking-wider opacity-70">
        {translator}
      </span>
      {text}
    </aside>
  )
}
