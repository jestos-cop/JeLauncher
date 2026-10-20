import { Check, Download } from 'lucide-react'
import { useDeferredValue, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { InstanceSummary, ProjectHit, Provider, ResourceKind, SortOrder } from '@/bindings'
import { ProjectRow } from '@/components/ProjectRow'
import { Badge } from '@/components/ui/badge'
import { SortSelect } from '@/components/SortSelect'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { useInstallVersion, useProjectVersions, useResourceSearch } from '@/hooks/use-modplatform'
import { PROVIDER_NAMES, RESOURCE_PROVIDERS } from '@/lib/providers'

interface ResourceDownloadDialogProps {
  instance: InstanceSummary
  kind: ResourceKind
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function ResourceDownloadDialog({ instance, kind, open, onOpenChange }: ResourceDownloadDialogProps) {
  const { t } = useTranslation(['instanceDialogs', 'common'])
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
            {instance.loader
              ? t('resourceDownload.titleWithLoader', {
                  instance: instance.name,
                  version: instance.minecraftVersion,
                  loader: instance.loader.kind,
                })
              : t('resourceDownload.title', { instance: instance.name, version: instance.minecraftVersion })}
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
            {RESOURCE_PROVIDERS.map((value) => (
              <ToggleGroupItem key={value} value={value}>
                {PROVIDER_NAMES[value]}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
          <Input
            autoFocus
            className="flex-1"
            placeholder={t('resourceDownload.search', { provider: PROVIDER_NAMES[provider] })}
            value={text}
            onChange={(event) => setText(event.target.value)}
          />
          <SortSelect provider={provider} value={sort} onChange={setSort} />
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
                {t('resourceDownload.loadMore')}
              </Button>
            )}
            {search.isPending && (
              <p className="p-3 text-sm text-muted-foreground">{t('resourceDownload.searching')}</p>
            )}
          </div>
          {selected ? (
            <VersionPanel
              key={`${selected.provider}-${selected.id}`}
              instanceId={instance.id}
              kind={kind}
              project={selected}
            />
          ) : (
            <p className="p-3 text-sm text-muted-foreground">{t('resourceDownload.selectProject')}</p>
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
  const { t } = useTranslation(['instanceDialogs', 'common'])
  const { data: versions, error, isPending } = useProjectVersions(instanceId, project, kind)
  const install = useInstallVersion(instanceId, kind, project)

  return (
    <div className="flex min-h-0 flex-col gap-2">
      <h3 className="font-semibold">{project.title}</h3>
      {install.error && <p className="text-sm text-destructive">{install.error.message}</p>}
      {install.data && install.data.length > 1 && (
        <p className="text-sm text-muted-foreground">
          {t('resourceDownload.alsoInstalled', { names: install.data.slice(1).join(', ') })}
        </p>
      )}
      {error && <p className="text-sm text-destructive">{error.message}</p>}
      <div className="min-h-0 flex-1 overflow-auto rounded-md border">
        {isPending && (
          <p className="p-3 text-sm text-muted-foreground">{t('resourceDownload.loadingVersions')}</p>
        )}
        {versions?.length === 0 && (
          <p className="p-3 text-sm text-muted-foreground">{t('resourceDownload.noVersions')}</p>
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
                  <Badge variant="secondary">{t(`releaseType.${version.releaseType}`)}</Badge>
                )}
                <Button
                  size="sm"
                  variant={installed ? 'secondary' : 'default'}
                  disabled={install.isPending || installed}
                  onClick={() => install.mutate(version.id)}
                >
                  {installed ? <Check /> : <Download />}
                  {installed ? t('resourceDownload.installed') : t('common:action.install')}
                </Button>
              </li>
            )
          })}
        </ul>
      </div>
    </div>
  )
}
