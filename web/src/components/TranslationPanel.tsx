// Translation panel: attributed translation text under the translation
// layer token. Renders as text — never injected as HTML (T-05-15).
export interface Translation {
  translator: string
  text: string
}

export default function TranslationPanel({ translator, text }: Translation) {
  return (
    <aside className="ayah-translation" data-translator={translator}>
      {text}
    </aside>
  )
}
