// Shared page masthead: mono eyebrow, serif title, one lede, optional
// actions. Every route opens with it so the reading rhythm stays identical.
export function PageHeader({
  eyebrow,
  title,
  lede,
  actions,
  children,
}: {
  eyebrow: string
  title: React.ReactNode
  lede?: React.ReactNode
  actions?: React.ReactNode
  children?: React.ReactNode
}) {
  return (
    <header className="border-b pb-6">
      <div className="flex flex-wrap items-start justify-between gap-x-8 gap-y-4">
        <div className="min-w-0 max-w-2xl">
          <p className="kicker mb-3">{eyebrow}</p>
          <h1 className="display text-[1.9rem] text-foreground sm:text-[2.3rem]">{title}</h1>
          {lede !== undefined && (
            <p className="mt-2.5 text-[0.875rem] leading-relaxed text-muted-foreground">{lede}</p>
          )}
        </div>
        {actions !== undefined && <div className="flex flex-wrap items-center gap-2">{actions}</div>}
      </div>
      {children}
    </header>
  )
}
