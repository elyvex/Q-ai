import { Network, GitBranch } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'

export default function GraphView({ lines, truncated }: { lines: string[]; truncated: boolean }) {
  void lines
  void truncated
  return (
    <div className="space-y-6">
      <div className="space-y-1">
        <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
          <Network className="h-3.5 w-3.5" />
          <span>Research</span>
          <span>/</span>
          <span>Knowledge Graph</span>
        </div>
        <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">Knowledge Graph</h1>
        <p className="text-sm text-muted-foreground">
          Explore relationships between texts, concepts, and references
        </p>
      </div>

      <Card className="min-h-[400px]">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <GitBranch className="h-5 w-5" />
                Graph Visualization
              </CardTitle>
              <CardDescription>Interactive exploration of textual relationships</CardDescription>
            </div>
            <Badge variant="outline">Planned</Badge>
          </div>
        </CardHeader>
        <CardContent className="flex h-[300px] items-center justify-center">
          <div className="text-center text-muted-foreground">
            <Network className="mx-auto mb-2 h-8 w-8 opacity-50" />
            <p className="text-sm">Knowledge graph visualization coming soon</p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
