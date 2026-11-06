import * as React from 'react'
import { ChevronDown } from 'lucide-react'
import { cn } from 'cn'

export interface TabItem {
  id: string
  label: string
  content: React.ReactNode
  disabled?: boolean
}

export interface TabsProps {
  items: TabItem[]
  defaultValue?: string
  value?: string
  onValueChange?: (value: string) => void
  ariaLabel?: string
  className?: string
}

export function Tabs({
  items,
  defaultValue,
  value,
  onValueChange,
  ariaLabel = 'Tabs',
  className,
}: TabsProps) {
  const [internalValue, setInternalValue] = React.useState(defaultValue ?? items[0]?.id ?? '')
  const activeId = value ?? internalValue
  const tabRefs = React.useRef<Array<HTMLButtonElement | null>>([])
  const baseId = React.useId()

  function selectTab(id: string) {
    if (value === undefined) setInternalValue(id)
    onValueChange?.(id)
  }

  function handleKeyDown(event: React.KeyboardEvent, index: number) {
    const enabledIndices = items
      .map((item, idx) => (item.disabled ? -1 : idx))
      .filter((idx) => idx >= 0)
    if (enabledIndices.length === 0) return

    const pos = enabledIndices.indexOf(index)
    let nextIndex: number | undefined

    if (event.key === 'ArrowRight') {
      event.preventDefault()
      nextIndex = enabledIndices[(pos + 1) % enabledIndices.length]
    } else if (event.key === 'ArrowLeft') {
      event.preventDefault()
      nextIndex = enabledIndices[(pos - 1 + enabledIndices.length) % enabledIndices.length]
    } else if (event.key === 'Home') {
      event.preventDefault()
      nextIndex = enabledIndices[0]
    } else if (event.key === 'End') {
      event.preventDefault()
      nextIndex = enabledIndices[enabledIndices.length - 1]
    }

    if (nextIndex !== undefined) {
      const targetItem = items[nextIndex]
      if (targetItem) {
        selectTab(targetItem.id)
        tabRefs.current[nextIndex]?.focus()
      }
    }
  }

  const activeItem = items.find((item) => item.id === activeId) ?? items[0]

  return (
    <div data-slot="tabs" className={cn('w-full', className)}>
      <div
        role="tablist"
        aria-label={ariaLabel}
        className="inline-flex flex-wrap gap-1 rounded-lg border border-border bg-muted/40 p-1"
      >
        {items.map((item, index) => {
          const isSelected = item.id === activeItem?.id
          return (
            <button
              key={item.id}
              ref={(el) => {
                tabRefs.current[index] = el
              }}
              id={`${baseId}-tab-${item.id}`}
              type="button"
              role="tab"
              aria-selected={isSelected}
              aria-controls={`${baseId}-panel-${item.id}`}
              tabIndex={isSelected ? 0 : -1}
              disabled={item.disabled}
              onClick={() => selectTab(item.id)}
              onKeyDown={(e) => handleKeyDown(e, index)}
              className={cn(
                'rounded-md px-3 py-1.5 text-xs font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50',
                isSelected
                  ? 'bg-background text-foreground shadow-xs'
                  : 'text-muted-foreground hover:text-foreground',
              )}
            >
              {item.label}
            </button>
          )
        })}
      </div>

      {activeItem && (
        <div
          id={`${baseId}-panel-${activeItem.id}`}
          role="tabpanel"
          aria-labelledby={`${baseId}-tab-${activeItem.id}`}
          className="mt-3 outline-none"
        >
          {activeItem.content}
        </div>
      )}
    </div>
  )
}

export interface AccordionItem {
  id: string
  title: string
  content: React.ReactNode
}

export interface AccordionProps {
  items: AccordionItem[]
  defaultOpenIds?: string[]
  className?: string
}

export function Accordion({ items, defaultOpenIds = [], className }: AccordionProps) {
  const [openIds, setOpenIds] = React.useState<Set<string>>(() => new Set(defaultOpenIds))
  const baseId = React.useId()

  function toggle(id: string) {
    setOpenIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  return (
    <div data-slot="accordion" className={cn('divide-y rounded-lg border', className)}>
      {items.map((item) => {
        const isOpen = openIds.has(item.id)
        return (
          <div key={item.id} data-slot="accordion-item">
            <button
              id={`${baseId}-acc-btn-${item.id}`}
              type="button"
              aria-expanded={isOpen}
              aria-controls={`${baseId}-acc-panel-${item.id}`}
              onClick={() => toggle(item.id)}
              className="flex w-full items-center justify-between px-4 py-3 text-left text-sm font-medium text-foreground transition-colors hover:bg-muted/40 focus-visible:ring-2 focus-visible:ring-ring outline-none"
            >
              <span>{item.title}</span>
              <ChevronDown
                className={cn(
                  'size-4 text-muted-foreground transition-transform duration-150',
                  isOpen && 'rotate-180',
                )}
                aria-hidden="true"
              />
            </button>
            {isOpen && (
              <div
                id={`${baseId}-acc-panel-${item.id}`}
                role="region"
                aria-labelledby={`${baseId}-acc-btn-${item.id}`}
                className="border-t px-4 py-3 text-sm text-muted-foreground"
              >
                {item.content}
              </div>
            )}
          </div>
        )
      })}
    </div>
  )
}

export default Tabs
