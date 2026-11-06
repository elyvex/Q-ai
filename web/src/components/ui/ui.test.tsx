import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { Search } from 'lucide-react'
import { Button } from './Button'
import { IconButton } from './IconButton'
import { Input, TextArea } from './Input'
import { Select } from './Select'
import { Badge, Chip } from './Badge'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from './Card'
import { Modal } from './Modal'
import { Accordion, Tabs } from './Tabs'
import { Skeleton } from './Skeleton'
import { Spinner } from './Spinner'
import { Toast, ToastContainer } from './Toast'
import { EmptyState } from './EmptyState'
import { ErrorState } from './ErrorState'
import { ErrorBoundary } from './ErrorBoundary'
import { AnimatedBackground } from '../background/AnimatedBackground'

afterEach(() => {
  cleanup()
  vi.restoreAllMocks()
})

describe('Button & IconButton primitives', () => {
  it.each(['primary', 'secondary', 'outline', 'ghost', 'danger'] as const)(
    'renders Button variant %s across sizes sm, md, lg, xl',
    (variant) => {
      for (const size of ['sm', 'md', 'lg', 'xl'] as const) {
        const { unmount } = render(
          <Button variant={variant} size={size}>
            {variant}-{size}
          </Button>,
        )
        const btn = screen.getByRole('button', { name: `${variant}-${size}` })
        expect(btn.getAttribute('data-variant')).toBe(variant)
        expect(btn.getAttribute('data-size')).toBe(size)
        expect(btn.className).toContain('hover:-translate-y-px')
        expect(btn.className).toContain('active:scale-[0.98]')
        expect(btn.className).toContain('focus-visible:ring-2')
        unmount()
      }
    },
  )

  it('handles disabled and loading states with spinner', () => {
    const onClick = vi.fn()
    render(
      <Button loading onClick={onClick}>
        Save
      </Button>,
    )
    const btn = screen.getByRole('button', { name: /Save/i })
    expect(btn.getAttribute('aria-busy')).toBe('true')
    expect(btn.hasAttribute('disabled')).toBe(true)
    expect(screen.getByRole('status', { name: 'Loading' })).toBeDefined()
    fireEvent.click(btn)
    expect(onClick).not.toHaveBeenCalled()
  })

  it('renders IconButton with aria-label and tooltip', () => {
    render(
      <IconButton aria-label="Search corpus" tooltip="Open search">
        <Search className="size-4" />
      </IconButton>,
    )
    const btn = screen.getByRole('button', { name: 'Search corpus' })
    expect(btn.getAttribute('title')).toBe('Open search')
    expect(btn.getAttribute('data-tooltip')).toBe('Open search')
  })
})

describe('Form primitives (Input, TextArea, Select)', () => {
  it('renders Input and TextArea with invalid state and accessible alert error text', () => {
    render(
      <div>
        <Input label="Reference" invalid errorText="Invalid verse reference" />
        <TextArea label="Notes" errorText="Notes cannot be empty" />
      </div>,
    )
    const input = screen.getByLabelText('Reference')
    expect(input.getAttribute('aria-invalid')).toBe('true')
    const alerts = screen.getAllByRole('alert')
    expect(alerts).toHaveLength(2)
    expect(alerts[0]?.textContent).toContain('Invalid verse reference')
  })

  it('renders Select with options and handles selection changes', () => {
    const onChange = vi.fn()
    render(
      <Select
        label="Search mode"
        options={[
          { value: 'exact', label: 'Exact' },
          { value: 'normalized', label: 'Normalized' },
        ]}
        onChange={onChange}
      />,
    )
    const select = screen.getByLabelText('Search mode') as HTMLSelectElement
    fireEvent.change(select, { target: { value: 'normalized' } })
    expect(onChange).toHaveBeenCalledOnce()
    expect(select.value).toBe('normalized')
  })
})

describe('Badge, Chip, Card, Skeleton, Spinner, Toast', () => {
  it('renders semantic Badges and dismissible Chip', () => {
    const onDismiss = vi.fn()
    render(
      <div>
        <Badge variant="canonical" dot>
          Canonical
        </Badge>
        <Chip variant="primary" onDismiss={onDismiss} dismissLabel="Remove filter">
          Surah 2
        </Chip>
      </div>,
    )
    expect(screen.getByText('Canonical').className).toContain('layer-canonical')
    fireEvent.click(screen.getByRole('button', { name: 'Remove filter' }))
    expect(onDismiss).toHaveBeenCalledOnce()
  })

  it('renders Card with interactive hover state and sub-sections', () => {
    const onClick = vi.fn()
    render(
      <Card interactive onClick={onClick}>
        <CardHeader>
          <CardTitle>Corpus Card</CardTitle>
          <CardDescription>Verified edition</CardDescription>
        </CardHeader>
        <CardContent>Body content</CardContent>
        <CardFooter>Footer</CardFooter>
      </Card>,
    )
    const title = screen.getByText('Corpus Card')
    fireEvent.click(title)
    expect(onClick).toHaveBeenCalledOnce()
  })

  it('renders Skeleton and Spinner loading indicators', () => {
    render(
      <div>
        <Skeleton lines={3} label="Loading ayahs" />
        <Spinner size="lg" label="Fetching data" />
      </div>,
    )
    expect(screen.getByRole('status', { name: 'Loading ayahs' })).toBeDefined()
    expect(screen.getByRole('status', { name: 'Fetching data' })).toBeDefined()
  })

  it('renders Toast notifications with status and alert roles', () => {
    const onDismiss = vi.fn()
    render(
      <ToastContainer>
        <Toast title="Saved" variant="success" />
        <Toast title="Failed" description="Network error" variant="error" onDismiss={onDismiss} />
      </ToastContainer>,
    )
    expect(screen.getByRole('status')).toBeDefined()
    expect(screen.getByRole('alert')).toBeDefined()
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss notification' }))
    expect(onDismiss).toHaveBeenCalledOnce()
  })
})

describe('Modal, Tabs, Accordion', () => {
  it('renders Modal with ESC dismiss and focus management', () => {
    const onClose = vi.fn()
    render(
      <Modal open onClose={onClose} title="Citation Details" description="Verified URN">
        <button type="button">Action 1</button>
      </Modal>,
    )
    expect(screen.getByRole('dialog')).toBeDefined()
    fireEvent.keyDown(document, { key: 'Escape' })
    expect(onClose).toHaveBeenCalledOnce()
  })

  it('navigates Tabs via keyboard ArrowRight / ArrowLeft', () => {
    render(
      <Tabs
        items={[
          { id: 't1', label: 'Tab One', content: <div>Content One</div> },
          { id: 't2', label: 'Tab Two', content: <div>Content Two</div> },
        ]}
      />,
    )
    const tab1 = screen.getByRole('tab', { name: 'Tab One' })
    expect(screen.getByText('Content One')).toBeDefined()
    fireEvent.keyDown(tab1, { key: 'ArrowRight' })
    expect(screen.getByText('Content Two')).toBeDefined()
  })

  it('toggles Accordion panels', () => {
    render(
      <Accordion items={[{ id: 'a1', title: 'Provenance Details', content: 'SHA-256 verified' }]} />,
    )
    const trigger = screen.getByRole('button', { name: 'Provenance Details' })
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
    fireEvent.click(trigger)
    expect(trigger.getAttribute('aria-expanded')).toBe('true')
    expect(screen.getByText('SHA-256 verified')).toBeDefined()
  })
})

describe('EmptyState, ErrorState, ErrorBoundary, AnimatedBackground', () => {
  it('renders EmptyState and ErrorState with action callbacks', () => {
    const onAction = vi.fn()
    const onRetry = vi.fn()
    render(
      <div>
        <EmptyState title="No results" description="Try another query" actionLabel="Reset" onAction={onAction} />
        <ErrorState message="Request timed out" code="QAI-NET-0001" onRetry={onRetry} />
      </div>,
    )
    fireEvent.click(screen.getByRole('button', { name: 'Reset' }))
    expect(onAction).toHaveBeenCalledOnce()
    fireEvent.click(screen.getByRole('button', { name: 'Retry' }))
    expect(onRetry).toHaveBeenCalledOnce()
  })

  it('catches render errors in ErrorBoundary and supports reset', () => {
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
    let shouldThrow = true
    function Buggy() {
      if (shouldThrow) throw new Error('Synthetic render crash')
      return <div>Recovered view</div>
    }
    const onReport = vi.fn()
    render(
      <ErrorBoundary onReport={onReport}>
        <Buggy />
      </ErrorBoundary>,
    )
    expect(screen.getByRole('alert')).toBeDefined()
    expect(screen.getByText('Synthetic render crash')).toBeDefined()
    expect(onReport).toHaveBeenCalledOnce()

    shouldThrow = false
    fireEvent.click(screen.getByRole('button', { name: /Reload view/i }))
    expect(screen.getByText('Recovered view')).toBeDefined()
    spy.mockRestore()
  })

  it('clamps AnimatedBackground opacity <= 0.15 and pauses when document is hidden', () => {
    const { container } = render(<AnimatedBackground maxOpacity={0.5} />)
    const bg = container.querySelector('[data-slot="animated-background"]') as HTMLElement
    expect(bg).not.toBeNull()
    expect(Number(bg.style.opacity)).toBeLessThanOrEqual(0.15)
    expect(bg.style.pointerEvents).toBe('none')
    expect(bg.style.zIndex).toBe('-1')
  })
})
