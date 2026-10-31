// RAG debug view: typed unavailability (D-11). There is deliberately no
// retrieval code path here — rendering results would present fiction as
// evidence. Retrieval lands in Phase 8.
import { FileLock2 } from 'lucide-react'

export default function RagDebugView() {
  return (
    <section>
      <div className="mb-6 flex flex-wrap items-end justify-between gap-x-6 gap-y-4 border-b pb-6">
        <div>
          <p className="kicker mb-2">Instruments · Observation</p>
          <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">RAG Debug</h1>
          <p className="mt-1 max-w-xl text-[0.8125rem] leading-relaxed text-muted-foreground">
            A sealed vault, not a placeholder: the interface stays empty precisely because no
            retrieval has run. Fiction is never dressed as evidence.
          </p>
        </div>
        <FileLock2 className="size-5 text-gold/60" aria-hidden="true" />
      </div>

      <div
        className="relative overflow-hidden rounded-xl border border-annot/35 bg-annot-soft px-6 py-10 text-center sm:px-10"
        role="status"
      >
        <span
          className="pointer-events-none absolute inset-y-0 left-8 hidden w-px border-l border-dashed border-annot/25 sm:block"
          aria-hidden="true"
        />
        <span
          className="pointer-events-none absolute inset-y-0 right-8 hidden w-px border-r border-dashed border-annot/25 sm:block"
          aria-hidden="true"
        />
        <span
          className="rag-state pointer-events-none absolute left-1/2 top-4 hidden -translate-x-1/2 rotate-[-4deg] rounded border border-annot/50 bg-background px-3 py-1 text-[0.62rem] font-semibold uppercase tracking-[0.2em] text-annot sm:block"
          aria-hidden="true"
        >
          sealed
        </span>
        <FileLock2 className="mx-auto mb-3 size-8 text-annot/70" aria-hidden="true" />
        <p className="rag-state text-base font-medium text-foreground/90">
          RAG not configured / unavailable
        </p>
        <p className="mx-auto mt-2 max-w-md text-[0.8125rem] leading-relaxed text-muted-foreground">
          Retrieval debugging lands in Phase 8. No results are shown because none were retrieved —
          this panel never fabricates exhibits.
        </p>
        <code className="mt-5 inline-block rounded-md border border-annot/40 bg-background px-3 py-1.5 font-mono text-[0.7rem] text-annot">
          rag: unbound
        </code>
      </div>
    </section>
  )
}
