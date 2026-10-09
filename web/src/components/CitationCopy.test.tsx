import { describe, expect, it, afterEach, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/react'
import CitationCopy from './CitationCopy'
import Router, { parseCitationId } from '../router'

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})

describe('CitationCopy', () => {
  it('echoes the API-provided URN + deep link verbatim (never formats its own)', () => {
    const urn = 'qai://quran/test@0.1.0/2:1'
    const deepLink = '/read/test@0.1.0/2:1'
    const { container } = render(<CitationCopy urn={urn} deepLink={deepLink} />)
    expect(container.querySelector('[data-urn]')?.textContent).toBe(urn)
    expect(container.querySelector('[data-deep-link]')?.textContent).toBe(deepLink)
    // No reconstructed variant anywhere in the output.
    expect(document.body.textContent).not.toContain('qai://quran/test@0.1.0/2:2')
  })
})

describe('citation deep-link route', () => {
  it('carries the citation id on the frozen deep link', () => {
    expect(parseCitationId('/read/test@0.1.0/2:1?citation=cite-9')).toBe('cite-9')
    expect(parseCitationId('/read/test@0.1.0/2:1')).toBeNull()
  })

  it('mounts CitationView for a deep link with ?citation=', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async (url: unknown) => {
        if (String(url).includes('/citations/')) {
          return {
            ok: true,
            json: async () => ({
              api_version: 'v1',
              data: {
                citation_id: 'cite-9',
                urn: 'qai://quran/test@0.1.0/2:1',
                deep_link: '/read/test@0.1.0/2:1',
                verdict: 'ExactMatch',
              },
              meta: {},
            }),
          }
        }
        return {
          ok: true,
          json: async () => ({
            api_version: 'v1',
            data: [
              {
                canonical: { reference: 'quran:test@0.1.0:2:1', arabicText: 'ا' },
                translations: [],
              },
            ],
            meta: {},
          }),
        }
      }),
    )
    render(<Router path="/read/test@0.1.0/2:1?citation=cite-9" />)
    await screen.findByText(/verified at the exact source location/i)
  })
})
