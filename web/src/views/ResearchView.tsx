import { Search, BookOpen, Network, Sparkles } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'

export default function ResearchView({ query }: { query: string }) {
  void query
  return (
    <div className="space-y-6">
      <div className="space-y-1">
        <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
          <Search className="h-3.5 w-3.5" />
          <span>Research</span>
          <span>/</span>
          <span>Workspace</span>
        </div>
        <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">Research Workspace</h1>
        <p className="text-sm text-muted-foreground">
          Cross-reference Quran, hadith, and comparative scripture with provenance tracking
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <Card className="group cursor-pointer transition-all hover:shadow-md hover:border-primary/20">
          <CardHeader>
            <div className="flex items-center gap-2">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/10">
                <BookOpen className="h-4 w-4 text-primary" />
              </div>
              <CardTitle className="text-base">Text Search</CardTitle>
            </div>
            <CardDescription>Search across canonical texts and translations</CardDescription>
          </CardHeader>
        </Card>

        <Card className="group cursor-pointer transition-all hover:shadow-md hover:border-primary/20">
          <CardHeader>
            <div className="flex items-center gap-2">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-provenance-trans-bg">
                <Network className="h-4 w-4 text-provenance-trans" />
              </div>
              <CardTitle className="text-base">Knowledge Graph</CardTitle>
            </div>
            <CardDescription>Explore relationships between concepts and references</CardDescription>
          </CardHeader>
        </Card>

        <Card className="group cursor-pointer transition-all hover:shadow-md hover:border-primary/20">
          <CardHeader>
            <div className="flex items-center gap-2">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-provenance-comp-bg">
                <Sparkles className="h-4 w-4 text-provenance-comp" />
              </div>
              <CardTitle className="text-base">RAG Research</CardTitle>
              <Badge variant="outline" className="text-xs">Coming Soon</Badge>
            </div>
            <CardDescription>AI-assisted research with full provenance</CardDescription>
          </CardHeader>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Getting Started</CardTitle>
          <CardDescription>Build your research workflow</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-sm text-muted-foreground">
          <p>• Use the Reader to explore canonical Quranic text with byte-exact rendering</p>
          <p>• Navigate via the sidebar or use ⌘K for quick access</p>
          <p>• All content maintains provenance - canonical, translation, and annotation layers are distinct</p>
        </CardContent>
      </Card>
    </div>
  )
}
