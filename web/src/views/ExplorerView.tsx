import { Terminal, Code2 } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'

export default function ExplorerView() {
  return (
    <div className="space-y-6">
      <div className="space-y-1">
        <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
          <Terminal className="h-3.5 w-3.5" />
          <span>System</span>
          <span>/</span>
          <span>API Explorer</span>
        </div>
        <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">API Explorer</h1>
        <p className="text-sm text-muted-foreground">
          Interactive exploration of the Q-ai HTTP API
        </p>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Code2 className="h-5 w-5" />
                API Endpoints
              </CardTitle>
              <CardDescription>Test and inspect API responses</CardDescription>
            </div>
            <Badge variant="outline">Developer Tool</Badge>
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-center text-muted-foreground py-8">
            <Terminal className="mx-auto mb-2 h-8 w-8 opacity-50" />
            <p className="text-sm">API explorer interface</p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
