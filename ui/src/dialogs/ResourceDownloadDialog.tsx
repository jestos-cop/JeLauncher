import { Check, Download } from 'lucide-react'
import { useDeferredValue, useState } from 'react'
import type { InstanceSummary, ProjectHit, Provider, ResourceKind, SortOrder } from '@/bindings'
import { ProjectRow } from '@/components/ProjectRow'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { useInstallVersion, useProjectVersions, useResourceSearch } from '@/hooks/use-modplatform'

const PROVIDERS: Record<Provider, string> = { modrinth: 'Modrinth', curseForge: 'CurseForge' }

const SORT_LABELS: Record<SortOrder, string> = {
  relevance: 'Relevance',
  downloads: 'Downloads',
  updated: 'Recently updated',
  newest: 'Newest',
}

interface ResourceDownloadDialogProps {
  instance: InstanceSummary
  kind: ResourceKind
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function ResourceDownloadDialog({ instance, kind, open, onOpenChange }: ResourceDownloadDialogProps) {
  const [provider, setProvider] = useState<Provider>('modrinth')
  const [text, setText] = useState('')
  const [sort, setSort] = useState<SortOrder>('relevance')
  const [selected, setSelected] = useState<ProjectHit | null>(null)
  const search = useResourceSearch({
    provider,
    text: useDeferredValue(text),
    kind,
    sort,
    gameVersion: instance.minecraftVersion,
    loader: instance.loader?.kind ?? null,
  })
  const hits = search.data?.pages.flatMap((page) => page.hits) ?? []

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex h-[85vh] flex-col sm:max-w-5xl">
        <DialogHeader>
          <DialogTitle>
            Download for {instance.name} · Minecraft {instance.minecraftVersion}
            {instance.loader && ` · ${instance.loader.kind}`}
          </DialogTitle>
        </DialogHeader>

        <div className="flex items-center gap-2">
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            value={provider}
            onValueChange={(next) => {
              if (!next) return
              setProvider(next as Provider)
              setSelected(null)
            }}
          >
            {Object.entries(PROVIDERS).map(([value, label]) => (
              <ToggleGroupItem key={value} value={value}>
                {label}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
          <Input
            autoFocus
            className="flex-1"
            placeholder={`Search ${PROVIDERS[provider]}`}
            value={text}
            onChange={(event) => setText(event.target.value)}
          />
          <Select value={sort} onValueChange={(value) => setSort(value as SortOrder)}>
            <SelectTrigger className="w-44">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {Object.entries(SORT_LABELS).map(([value, label]) => (
                <SelectItem key={value} value={value}>
                  {label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="grid min-h-0 flex-1 grid-cols-[1fr_20rem] gap-3">
          <div className="min-h-0 overflow-auto rounded-md border">
            {search.error && <p className="p-3 text-sm text-destructive">{search.error.message}</p>}
            <ul>
              {hits.map((hit) => (
                <ProjectRow
                  key={hit.id}
                  hit={hit}
                  isSelected={hit.id === selected?.id}
                  onSelect={() => setSelected(hit)}
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
            {search.isPending && <p className="p-3 text-sm text-muted-foreground">Searching…</p>}
          </div>
          {selected ? (
            <VersionPanel
              key={`${selected.provider}-${selected.id}`}
              instanceId={instance.id}
              kind={kind}
              project={selected}
            />
          ) : (
            <p className="p-3 text-sm text-muted-foreground">Select a project to see compatible versions.</p>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}

interface VersionPanelProps {
  instanceId: string
  kind: ResourceKind
  project: ProjectHit
}

function VersionPanel({ instanceId, kind, project }: VersionPanelProps) {
  const { data: versions, error, isPending } = useProjectVersions(instanceId, project, kind)
  const install = useInstallVersion(instanceId, kind, project)

  return (
    <div className="flex min-h-0 flex-col gap-2">
      <h3 className="font-semibold">{project.title}</h3>
      {install.error && <p className="text-sm text-destructive">{install.error.message}</p>}
      {install.data && install.data.length > 1 && (
        <p className="text-sm text-muted-foreground">Also installed: {install.data.slice(1).join(', ')}</p>
      )}
      {error && <p className="text-sm text-destructive">{error.message}</p>}
      <div className="min-h-0 flex-1 overflow-auto rounded-md border">
        {isPending && <p className="p-3 text-sm text-muted-foreground">Loading versions…</p>}
        {versions?.length === 0 && (
          <p className="p-3 text-sm text-muted-foreground">No versions for this instance.</p>
        )}
        <ul>
          {versions?.map((version) => {
            const installed = install.isSuccess && install.variables === version.id
            return (
              <li key={version.id} className="flex items-center gap-2 border-b p-2 last:border-0">
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{version.versionNumber}</p>
                  <p className="text-xs text-muted-foreground">
                    {new Date(version.published).toLocaleDateString()}
                  </p>
                </div>
                {version.releaseType !== 'release' && (
                  <Badge variant="secondary">{version.releaseType}</Badge>
                )}
                <Button
                  size="sm"
                  variant={installed ? 'secondary' : 'default'}
                  disabled={install.isPending || installed}
                  onClick={() => install.mutate(version.id)}
                >
                  {installed ? <Check /> : <Download />}
                  {installed ? 'Installed' : 'Install'}
                </Button>
              </li>
            )
          })}
        </ul>
      </div>
    </div>
  )
}
