import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import * as apiClient from '../../api/client'
import {
  formatAyahDeepLink,
  parseReaderDeepLink,
  ReaderView,
} from './ReaderView'

afterEach(() => {
  cleanup()
  vi.restoreAllMocks()
})

const SAMPLE_AYAHS: apiClient.AyahView[] = [
  {
    canonical: {
      reference: 'quran:uthmani@1.0.0:1:1',
      arabicText: 'بِسْمِ ٱللَّهِ ٱلرَّحْمَـٰنِ ٱلرَّحِيمِ',
    },
    translations: [
      {
        translator: 'Sahih International',
        text: 'In the name of Allah, the Entirely Merciful, the Especially Merciful.',
      },
    ],
  },
]

describe('ReaderView (Tracer Slice — Phase 13-01)', () => {
  it('parses and formats canonical deep links (/read/{slug}@{version}/{surah}:{ayah})', () => {
    const parsed = parseReaderDeepLink('/read/hafs-uthmani@1.0.0/2:255')
    expect(parsed).toEqual({
      edition: 'hafs-uthmani@1.0.0',
      reference: '2:255',
      deepLink: '/read/hafs-uthmani@1.0.0/2:255',
    })
    expect(formatAyahDeepLink('quran:hafs-uthmani@1.0.0:2:255')).toBe(
      '/read/hafs-uthmani@1.0.0/2:255',
    )
  })

  it('renders loading skeletons while fetchAyahs is in flight', () => {
    vi.spyOn(apiClient, 'fetchAyahs').mockImplementation(() => new Promise(() => {}))
    render(<ReaderView initialReference="1:1" />)
    expect(screen.getByTestId('reader-loading')).toBeDefined()
    expect(screen.getByRole('status', { name: 'Loading canonical ayahs' })).toBeDefined()
  })

  it('connects to fetchAyahs and renders canonical Arabic with RTL + unicode-bidi: isolate and strict layer separation', async () => {
    const spy = vi.spyOn(apiClient, 'fetchAyahs').mockResolvedValue({
      api_version: 'v1',
      data: SAMPLE_AYAHS,
      meta: {
        edition: { slug: 'uthmani', version: '1.0.0' },
        corpus_generation: 1,
        canonical_reference: 'quran:uthmani@1.0.0:1:1',
        deep_link: '/read/uthmani@1.0.0/1:1',
        execution_time_ms: 2,
        reproducibility: {},
        research_checksum: 'sha256:abc',
        warnings: [],
      },
    })

    const { container } = render(
      <ReaderView
        deepLinkPath="/read/uthmani@1.0.0/1:1"
        annotations={[
          {
            id: 'ann-1',
            reference: 'quran:uthmani@1.0.0:1:1',
            label: 'Root ب-س-م',
            note: 'Basmala opening invocation',
          },
        ]}
      />,
    )

    await waitFor(() => {
      expect(screen.getByTestId('reader-ayah-list')).toBeDefined()
    })
    expect(spy).toHaveBeenCalledWith('1:1')

    // Canonical Arabic layer invariants
    const canonicalEl = container.querySelector('.ayah-canonical') as HTMLElement
    expect(canonicalEl).not.toBeNull()
    expect(canonicalEl.getAttribute('dir')).toBe('rtl')
    expect(canonicalEl.getAttribute('lang')).toBe('ar')
    expect(canonicalEl.style.unicodeBidi).toBe('isolate')
    expect(canonicalEl.style.color).toContain('--layer-canonical-fg')
    expect(canonicalEl.textContent).toBe('بِسْمِ ٱللَّهِ ٱلرَّحْمَـٰنِ ٱلرَّحِيمِ')

    // Translation layer invariants (never wears .ayah-canonical)
    const translationEl = container.querySelector('[data-translator="Sahih International"]') as HTMLElement
    expect(translationEl).not.toBeNull()
    expect(translationEl.className).toContain('ayah-translation')
    expect(translationEl.className).not.toContain('ayah-canonical')
    expect(translationEl.style.color).toContain('--layer-translation-fg')
    expect(translationEl.textContent).toContain('In the name of Allah')

    // Annotation layer invariants (left rule + --layer-annotation-*)
    const annotationEl = container.querySelector('.ayah-annotation') as HTMLElement
    expect(annotationEl).not.toBeNull()
    expect(annotationEl.className).not.toContain('ayah-canonical')
    expect(annotationEl.style.color).toContain('--layer-annotation-fg')
    expect(annotationEl.style.borderLeft).toContain('--layer-annotation-border')
    expect(annotationEl.textContent).toContain('Basmala opening invocation')

    // Preserves deep link anchor
    const deepLinkEl = container.querySelector('[data-slot="ayah-deep-link"]') as HTMLAnchorElement
    expect(deepLinkEl?.getAttribute('href')).toBe('/read/uthmani@1.0.0/1:1')
  })

  it('renders ErrorState when fetchAyahs rejects and retries on button click', async () => {
    const spy = vi
      .spyOn(apiClient, 'fetchAyahs')
      .mockRejectedValueOnce(new Error('Canonical edition not active'))
      .mockResolvedValueOnce({
        api_version: 'v1',
        data: SAMPLE_AYAHS,
        meta: {
          edition: { slug: 'uthmani', version: '1.0.0' },
          corpus_generation: 1,
          canonical_reference: 'quran:uthmani@1.0.0:1:1',
          deep_link: '/read/uthmani@1.0.0/1:1',
          execution_time_ms: 1,
          reproducibility: {},
          research_checksum: 'sha256:def',
          warnings: [],
        },
      })

    render(<ReaderView initialReference="1:1" />)

    await waitFor(() => {
      expect(screen.getByRole('alert')).toBeDefined()
    })
    expect(screen.getByText('Canonical edition not active')).toBeDefined()

    fireEvent.click(screen.getByRole('button', { name: /Retry/i }))
    await waitFor(() => {
      expect(screen.getByTestId('reader-ayah-list')).toBeDefined()
    })
    expect(spy).toHaveBeenCalledTimes(2)
  })

  it('renders EmptyState when no ayahs are returned', async () => {
    vi.spyOn(apiClient, 'fetchAyahs').mockResolvedValue({
      api_version: 'v1',
      data: [],
      meta: {
        edition: { slug: 'uthmani', version: '1.0.0' },
        corpus_generation: 1,
        canonical_reference: '',
        deep_link: '',
        execution_time_ms: 1,
        reproducibility: {},
        research_checksum: 'sha256:000',
        warnings: [],
      },
    })

    render(<ReaderView initialReference="999:999" />)
    await waitFor(() => {
      expect(screen.getByText('No ayahs loaded for this reference')).toBeDefined()
    })
  })
})
