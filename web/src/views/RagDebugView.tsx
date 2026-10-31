// RAG debug view: typed unavailability (D-11). There is deliberately no
// retrieval code path here — rendering results would present fiction as
// evidence. Retrieval lands in Phase 8.
import { FileLock2 } from 'lucide-react'
import { PageHeader } from '../components/layout/page-header'

export default function RagDebugView() {
  return (
    <section>
      <PageHeader
        eyebrow="Instruments · Observation"
        title="RAG Debug"
        lede="A sealed vault, not a placeholder: the interface stays empty precisely because no retrieval has run. Fiction is never dressed as evidence."
        actions={<span className="chip">sealed</span>}
      />

      <div className="mt-8 grid gap-x-10 gap-y-8 lg:grid-cols-[minmax(0,1fr)_16rem]">
        <div
          role="status"
          className="flex min-h-[18rem] flex-col items-center justify-center rounded-lg border border-dashed px-6 py-12 text-center"
        >
          <FileLock2 className="mb-4 size-6 text-muted-foreground/60" aria-hidden="true" />
          <p className="rag-state display text-[1.35rem] text-foreground">
            RAG not configured / unavailable
          </p>
          <p className="mt-2 max-w-sm text-[0.8125rem] leading-relaxed text-muted-foreground">
            No results are shown because none were retrieved. This panel never fabricates exhibits.
          </p>
          <code className="mt-5 rounded border bg-muted px-2 py-1 font-mono text-[0.65rem] text-muted-foreground">
            rag: unbound
          </code>
        </div>

        <aside className="lg:sticky lg:top-20 lg:h-fit">
          <p className="kicker mb-2.5">Why this is empty</p>
          <ul className="margin-note space-y-2.5 border-t pt-4">
            <li>Retrieval debugging lands in Phase 8; no retrieval path is compiled in yet.</li>
            <li>A typed unavailable state is surfaced instead of a mock result set.</li>
            <li>The runtime fails closed — absence is stated, never disguised.</li>
          </ul>
        </aside>
      </div>
    </section>
  )
}
