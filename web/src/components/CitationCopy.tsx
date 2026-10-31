// Citation copy control (D-17/ADR-0111): emits the API-provided stable
// URN + deep link verbatim. The format is frozen in the `citations`
// helper — the SPA never reconstructs it.
export default function CitationCopy({ urn, deepLink }: { urn: string; deepLink: string }) {
  async function copy() {
    try {
      await navigator.clipboard.writeText(`${urn}\n${deepLink}`)
    } catch {
      // Clipboard unavailable (non-secure context): selection still works.
    }
  }
  return (
    <div>
      <p data-urn className="break-all font-mono text-xs">
        {urn}
      </p>
      <p data-deep-link className="break-all font-mono text-xs text-muted-foreground">
        {deepLink}
      </p>
      <button
        type="button"
        onClick={() => void copy()}
        className="mt-2 rounded-lg bg-emerald-700 px-2.5 py-1 text-xs font-medium text-white transition-colors hover:bg-emerald-700/80"
      >
        copy citation
      </button>
    </div>
  )
}
