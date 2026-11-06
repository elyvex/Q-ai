import * as React from 'react'
import { AlertTriangle, RotateCcw } from 'lucide-react'
import { Button } from './Button'

export interface ErrorBoundaryProps {
  children: React.ReactNode
  fallback?: React.ReactNode
  onReset?: () => void
  onReport?: (error: Error) => void
}

interface ErrorBoundaryState {
  error: Error | null
}

export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props)
    this.state = { error: null }
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error }
  }

  override componentDidCatch(error: Error, info: React.ErrorInfo) {
    void info
    this.props.onReport?.(error)
  }

  handleReset = () => {
    this.setState({ error: null })
    this.props.onReset?.()
  }

  override render() {
    const { error } = this.state
    if (error) {
      if (this.props.fallback) {
        return this.props.fallback
      }
      return (
        <div
          role="alert"
          data-slot="error-boundary"
          className="rounded-lg border border-[#B91C1C]/30 bg-[#FEF2F2] p-6 text-[#1C1917]"
        >
          <div className="flex items-start gap-3">
            <AlertTriangle className="mt-0.5 size-5 shrink-0 text-[#B91C1C]" aria-hidden="true" />
            <div className="flex-1">
              <h2 className="text-sm font-semibold">View failed to render</h2>
              <p className="mt-1 font-mono text-xs opacity-90">{error.message}</p>
              <div className="mt-4 flex items-center gap-2">
                <Button type="button" variant="outline" size="sm" onClick={this.handleReset}>
                  <RotateCcw className="mr-1 size-3.5" aria-hidden="true" />
                  Reload view
                </Button>
                {this.props.onReport && (
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => this.props.onReport?.(error)}
                  >
                    Report issue
                  </Button>
                )}
              </div>
            </div>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}

export default ErrorBoundary
