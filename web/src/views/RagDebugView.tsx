// RAG debug view: typed unavailability (D-11). There is deliberately no
// retrieval code path here — rendering results would present fiction as
// evidence. Retrieval lands in Phase 8.
export default function RagDebugView() {
  return (
    <section>
      <p className="rag-state">RAG not configured / unavailable</p>
      <p>Retrieval debugging lands in Phase 8. No results are shown because none were retrieved.</p>
    </section>
  )
}
