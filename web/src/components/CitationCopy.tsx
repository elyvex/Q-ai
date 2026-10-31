// Citation copy control (D-17/ADR-0111): emits the API-provided stable
// URN + deep link verbatim. The format is frozen in the `citations`
// helper — the SPA never reconstructs it.
import { Copy } from 'lucide-react'

export default function CitationCopy({ urn, deepLink }: { urn: string; deepLink: string }) {
  async function copy() {
    try {
      await navigator.clipboard.writeText(`${urn}\n${deepLink}`)
    } catch {
      // Clipboard unavailable (non-secure context): selection still works.
    }
  }
  return (
    <div className="space-y-1.5">
      <p data-urn className="break-all rounded border border-canon/25 bg-background px-2.5 py-1.5 font-mono text-xs text-foreground/90">
        {urn}
      </p>
      <p data-deep-link className="break-all rounded bg-muted px-2.5 py-1.5 font-mono text-xs text-muted-foreground">
        {deepLink}
      </p>
      <button
        type="button"
        onClick={() => void copy()}
        className="btn btn-sm mt-1"
      >
        <Copy className="size-3" aria-hidden="true" />
        copy citation
      </button>
    </div>
  )
}
