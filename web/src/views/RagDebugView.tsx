import { Database, Activity } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'

export default function RagDebugView() {
  return (
    <div className="space-y-6">
      <div className="space-y-1">
        <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
          <Database className="h-3.5 w-3.5" />
          <span>System</span>
          <span>/</span>
          <span>RAG Debug</span>
        </div>
        <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">RAG Debugging</h1>
        <p className="text-sm text-muted-foreground">
          Inspect retrieval and generation pipeline diagnostics
        </p>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Activity className="h-5 w-5" />
                Pipeline Diagnostics
              </CardTitle>
              <CardDescription>Debug retrieval, ranking, and context assembly</CardDescription>
            </div>
            <Badge variant="outline">Developer Tool</Badge>
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-center text-muted-foreground py-8">
            <Database className="mx-auto mb-2 h-8 w-8 opacity-50" />
            <p className="text-sm">RAG debugging interface</p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
