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
      <p data-urn>{urn}</p>
      <p data-deep-link>{deepLink}</p>
      <button type="button" onClick={() => void copy()}>
        copy citation
      </button>
    </div>
  )
}
