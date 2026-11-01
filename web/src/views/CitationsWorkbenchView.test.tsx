import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import CitationsWorkbenchView from './CitationsWorkbenchView'

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})

const CITATION = {
  citation_id: 'cite-1',
  verdict: 'ExactMatch',
  text_hash: 'sha256:deadbeef',
  urn: 'qai://quran/test@0.1.0/1:1',
  deep_link: '/read/test@0.1.0/1:1',
}

const AYAH = {
  canonical: { reference: '1:1', arabicText: 'abc' },
  translations: [],
}

function stubSuccess() {
  const calls: unknown[][] = []
  vi.stubGlobal(
    'fetch',
    vi.fn(async (url: unknown) => {
      calls.push([url])
      const path = String(url)
      if (path.includes('/citations/')) {
        return {
          ok: true,
          json: async () => ({
            api_version: 'v1',
            data: CITATION,
            meta: { edition: { slug: 'test', version: '0.1.0' } },
          }),
        }
      }
      return {
        ok: true,
        json: async () => ({
          api_version: 'v1',
          data: [AYAH],
          meta: { edition: { slug: 'test', version: '0.1.0' } },
        }),
      }
    }),
  )
  return calls
}

async function openCitation() {
  render(<CitationsWorkbenchView initialId="cite-1" />)
  fireEvent.click(screen.getByRole('button', { name: 'Open' }))
  await screen.findByText(/verdict ExactMatch/)
}

describe('CitationsWorkbenchView verdict panel', () => {
  it('opens an id to the server verdict, hash, URN and deep link', async () => {
    stubSuccess()
    await openCitation()
    expect(screen.getByText(/verdict ExactMatch/)).toBeDefined()
    expect(screen.getByText('sha256:deadbeef')).toBeDefined()
    expect(screen.getByText('qai://quran/test@0.1.0/1:1')).toBeDefined()
    expect(
      screen.getByRole('link', { name: '/read/test@0.1.0/1:1' }),
    ).toBeDefined()
    expect(screen.getByText('byte-identical · 3 code points')).toBeDefined()
  })

  it('re-verify issues a second citation call and the session counter increments', async () => {
    const calls = stubSuccess()
    await openCitation()
    const first = calls.filter((call) => String(call[0]).includes('/citations/'))
    expect(first.length).toBe(1)
    expect(screen.getByText(/verified once against the server/)).toBeDefined()
    fireEvent.click(screen.getByRole('button', { name: 'Re-verify' }))
    await screen.findByText(/verified 2 times against the server/)
    const rerun = calls.filter((call) => String(call[0]).includes('/citations/'))
    expect(rerun.length).toBe(2)
  })

  it('renders a mismatch-shaped error body with its code and no verdict', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: false,
        status: 409,
        json: async () => ({
          error: {
            code: 'QAI-QUR-0323',
            summary: 'quotation does not match canonical text',
            remedy: 're-import the edition and re-open the citation',
          },
        }),
      })),
    )
    render(<CitationsWorkbenchView initialId="cite-bad" />)
    fireEvent.click(screen.getByRole('button', { name: 'Open' }))
    await screen.findByText(/QAI-QUR-0323/)
    expect(screen.getByText(/quotation does not match/)).toBeDefined()
    expect(screen.getByText(/re-import the edition/)).toBeDefined()
    expect(screen.queryByText(/verdict ExactMatch/)).toBeNull()
    expect(screen.queryByRole('button', { name: 'Re-verify' })).toBeNull()
  })
})

describe('CitationsWorkbenchView idle state', () => {
  it('renders a named empty state before any citation is opened', () => {
    render(<CitationsWorkbenchView initialId="" />)
    expect(screen.getByText('no citation is open')).toBeDefined()
    expect(screen.getByText(/the verdict prints here verbatim/)).toBeDefined()
  })
})

describe('CitationsWorkbenchView tamper simulator', () => {
  it('editing one character flips the readout while the verdict is unchanged and issues no request', async () => {
    const calls = stubSuccess()
    await openCitation()
    const before = calls.length
    const box = screen.getByLabelText(/Local copy/) as HTMLTextAreaElement
    expect(box.getAttribute('dir')).toBe('rtl')
    expect(box.getAttribute('lang')).toBe('ar')
    fireEvent.change(box, { target: { value: 'adc' } })
    expect(
      screen.getByText('first difference at code point 1: expected U+0062, found U+0064'),
    ).toBeDefined()
    // The server verdict panel above is untouched by the local edit.
    expect(screen.getByText(/verdict ExactMatch/)).toBeDefined()
    expect(calls.length).toBe(before)
  })

  it('restore returns the readout to byte-identical', async () => {
    stubSuccess()
    await openCitation()
    const box = screen.getByLabelText(/Local copy/)
    fireEvent.change(box, { target: { value: 'adc' } })
    expect(screen.getByText(/first difference at code point/)).toBeDefined()
    fireEvent.click(screen.getByRole('button', { name: 'Restore' }))
    expect(screen.getByText('byte-identical · 3 code points')).toBeDefined()
  })

  it('states the local-copy boundary in the margin-note voice', async () => {
    stubSuccess()
    await openCitation()
    expect(screen.getByText(/edits a copy held in the browser/)).toBeDefined()
    expect(screen.getByText(/does not write to the corpus/)).toBeDefined()
    expect(screen.getByText(/does not change the stored verdict/)).toBeDefined()
    expect(screen.getByText(/does not recompute text_hash/)).toBeDefined()
  })
})
