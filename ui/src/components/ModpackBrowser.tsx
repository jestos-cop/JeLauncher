import { useDeferredValue, useState } from 'react'
import type { ProjectHit } from '@/bindings'
import { ProjectRow } from '@/components/ProjectRow'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useModpackVersions } from '@/hooks/use-modpacks'
import { useResourceSearch } from '@/hooks/use-modplatform'
import { cn } from '@/lib/utils'

interface ModpackBrowserProps {
  selectedVersionId: string | null
  onSelectVersion: (versionId: string | null) => void
}

export function ModpackBrowser({ selectedVersionId, onSelectVersion }: ModpackBrowserProps) {
  const [text, setText] = useState('')
  const [project, setProject] = useState<ProjectHit | null>(null)
  const search = useResourceSearch({
    provider: 'modrinth',
    text: useDeferredValue(text),
    kind: 'modpack',
    sort: 'relevance',
    gameVersion: null,
    loader: null,
  })
  const hits = search.data?.pages.flatMap((page) => page.hits) ?? []

  const selectProject = (hit: ProjectHit) => {
    setProject(hit)
    onSelectVersion(null)
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      <Input
        placeholder="Search Modrinth modpacks"
        value={text}
        onChange={(event) => setText(event.target.value)}
      />
      <div className="grid min-h-0 flex-1 grid-cols-[1fr_16rem] gap-3">
        <div className="min-h-0 overflow-auto rounded-md border">
          {search.error && <p className="p-3 text-sm text-destructive">{search.error.message}</p>}
          <ul>
            {hits.map((hit) => (
              <ProjectRow
                key={hit.id}
                hit={hit}
                isSelected={hit.id === project?.id}
                onSelect={() => selectProject(hit)}
              />
            ))}
          </ul>
          {search.hasNextPage && (
            <Button
              variant="ghost"
              className="w-full"
              disabled={search.isFetchingNextPage}
              onClick={() => void search.fetchNextPage()}
            >
              Load more
            </Button>
          )}
        </div>
        {project ? (
          <ModpackVersions projectId={project.id} selected={selectedVersionId} onSelect={onSelectVersion} />
        ) : (
          <p className="p-3 text-sm text-muted-foreground">Select a modpack to choose a version.</p>
        )}
      </div>
    </div>
  )
}

interface ModpackVersionsProps {
  projectId: string
  selected: string | null
  onSelect: (versionId: string) => void
}

function ModpackVersions({ projectId, selected, onSelect }: ModpackVersionsProps) {
  const { data: versions, error, isPending } = useModpackVersions(projectId)
  return (
    <div className="min-h-0 overflow-auto rounded-md border">
      {isPending && <p className="p-3 text-sm text-muted-foreground">Loading versions…</p>}
      {error && <p className="p-3 text-sm text-destructive">{error.message}</p>}
      <ul>
        {versions?.map((version) => (
          <li key={version.id}>
            <button
              type="button"
              onClick={() => onSelect(version.id)}
              className={cn(
                'flex w-full items-center gap-2 p-2 text-left hover:bg-accent',
                version.id === selected && 'bg-accent',
              )}
            >
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">{version.versionNumber}</p>
                <p className="truncate text-xs text-muted-foreground">{version.name}</p>
              </div>
              {version.releaseType !== 'release' && <Badge variant="secondary">{version.releaseType}</Badge>}
            </button>
          </li>
        ))}
      </ul>
    </div>
  )
}
