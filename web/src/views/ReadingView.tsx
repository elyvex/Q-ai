// RTL reading view with provenance-first design
// Canonical text renders byte-exact in document order
import { useState } from 'react'
import { BookOpen, Layers } from 'lucide-react'
import AyahColumn from '../components/AyahColumn'
import TranslationPanel from '../components/TranslationPanel'
import WordInspector, { type InspectorMode } from '../components/WordInspector'
import DisplayModeControl, { type TranslationMode } from '../display/DisplayModeControl'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent } from '@/components/ui/card'
import type { AyahView } from '../api/client'

export default function ReadingView({
  ayahs,
  onResolve,
}: {
  ayahs: AyahView[]
  onResolve: (reference: string) => void
}) {
  const [translationMode, setTranslationMode] = useState<TranslationMode>('inline')
  const [inspectorMode, setInspectorMode] = useState<InspectorMode>('popover')
  void onResolve

  const parts = ayahs[0]?.canonical.reference.split(':') ?? []
  const label = parts.length >= 4 ? `Āyah ${parts[2]}:${parts[3]}` : undefined
  const surahRef = parts.length >= 3 ? `Surah ${parts[2]}` : undefined

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="space-y-1">
          <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
            <BookOpen className="h-3.5 w-3.5" />
            <span>Quran</span>
            <span>/</span>
            <span>Reader</span>
          </div>
          <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">
            {surahRef ? `${surahRef}, ${label?.replace('Āyah ', '')}` : label ?? 'Read'}
          </h1>
          <p className="text-sm text-muted-foreground">
            Canonical Arabic text rendered byte-exact • Translations and annotations as separate layers
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Badge variant="outline" className="gap-1.5 text-xs">
            <Layers className="h-3 w-3" />
            Provenance Tracked
          </Badge>
          <Card className="p-1">
            <DisplayModeControl
              translationMode={translationMode}
              inspectorMode={inspectorMode}
              onTranslationMode={setTranslationMode}
              onInspectorMode={setInspectorMode}
            />
          </Card>
        </div>
      </div>

      <div className="space-y-4" dir="rtl" lang="ar">
        {ayahs.map((ayah) => (
          <Card key={ayah.canonical.reference} className="overflow-hidden shadow-sm">
            <CardContent className="p-6 sm:p-8">
              <div
                className="mb-4 flex items-center justify-between gap-3 border-b pb-3"
                dir="ltr"
              >
                <Badge variant="secondary" className="font-mono text-xs">
                  {ayah.canonical.reference}
                </Badge>
                <Badge variant="outline" className="text-xs text-provenance-canon">
                  Canonical
                </Badge>
              </div>
              <AyahColumn arabicText={ayah.canonical.arabicText} reference={ayah.canonical.reference} />
              {ayah.translations.length === 0 ? (
                <div
                  className="mt-4 rounded-lg border border-dashed p-3 text-sm text-muted-foreground italic"
                  data-empty-translation="true"
                  dir="ltr"
                >
                  No translation available for this edition
                </div>
              ) : translationMode === 'inline' ? (
                <div className="mt-6 space-y-3" dir="ltr">
                  {ayah.translations.map((t) => (
                    <TranslationPanel key={t.translator} translator={t.translator} text={t.text} />
                  ))}
                </div>
              ) : translationMode === 'side-by-side' ? (
                <div className="mt-6 grid gap-4 md:grid-cols-2" dir="ltr">
                  {ayah.translations.map((t) => (
                    <Card key={t.translator} className="bg-muted/30">
                      <CardContent className="p-4">
                        <TranslationPanel translator={t.translator} text={t.text} />
                      </CardContent>
                    </Card>
                  ))}
                </div>
              ) : (
                <details
                  className="mt-6 rounded-lg border bg-muted/30"
                  dir="ltr"
                >
                  <summary className="cursor-pointer px-4 py-2.5 text-sm font-medium hover:bg-muted/50">
                    Translations ({ayah.translations.length})
                  </summary>
                  <div className="space-y-2 border-t p-4">
                    {ayah.translations.map((t) => (
                      <TranslationPanel key={t.translator} translator={t.translator} text={t.text} />
                    ))}
                  </div>
                </details>
              )}
              <div className="mt-4 text-right" dir="rtl">
                <WordInspector word={{ surface: '•' }} mode={inspectorMode} />
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}
