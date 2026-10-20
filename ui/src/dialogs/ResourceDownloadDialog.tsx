import { Check, Download } from 'lucide-react'
import { useDeferredValue, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type {
  InstanceSummary,
  ProjectHit,
  Provider,
  ResourceFile,
  ResourceKind,
  ResourceSource,
  SortOrder,
} from '@/bindings'
import { ProjectRow } from '@/components/ProjectRow'
import { Badge } from '@/components/ui/badge'
import { SortSelect } from '@/components/SortSelect'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { useInstallVersion, useProjectVersions, useResourceSearch } from '@/hooks/use-modplatform'
import { useChangeVersion, useResources } from '@/hooks/use-resources'
import { PROVIDER_NAMES, RESOURCE_PROVIDERS } from '@/lib/providers'

interface ResourceDownloadDialogProps {
  instance: InstanceSummary
  kind: ResourceKind
  open: boolean
  onOpenChange: (open: boolean) => void
}

/** Key of an installed project, for marking it in search results. */
const projectKey = (provider: Provider, projectId: string) => `${provider}:${projectId}`

export function ResourceDownloadDialog({ instance, kind, open, onOpenChange }: ResourceDownloadDialogProps) {
  const { t } = useTranslation(['instanceDialogs', 'common'])
  const { data: files = [] } = useResources(instance.id, kind)
  const installedFiles = new Map(
    files.flatMap((file) =>
      file.source ? [[projectKey(file.source.provider, file.source.projectId), file]] : [],
    ),
  )
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
                  installed={installedFiles.has(projectKey(hit.provider, hit.id))}
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
              installedFile={installedFiles.get(projectKey(selected.provider, selected.id))}
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
  /** The installed file of this project; picking another version then replaces it. */
  installedFile?: ResourceFile
}

function VersionPanel({ instanceId, kind, project, installedFile }: VersionPanelProps) {
  return (
    <div className="flex min-h-0 flex-col gap-2">
      <h3 className="font-semibold">{project.title}</h3>
      {installedFile?.source ? (
        <SwitchVersionList
          instanceId={instanceId}
          kind={kind}
          project={project}
          fileName={installedFile.fileName}
          source={installedFile.source}
        />
      ) : (
        <InstallVersionList instanceId={instanceId} kind={kind} project={project} />
      )}
    </div>
  )
}

function InstallVersionList({ instanceId, kind, project }: Omit<VersionPanelProps, 'installedFile'>) {
  const { t } = useTranslation(['instanceDialogs', 'common'])
  const install = useInstallVersion(instanceId, kind, project)
  return (
    <VersionList
      instanceId={instanceId}
      kind={kind}
      project={project}
      currentVersionId={undefined}
      error={install.error}
      note={
        install.data && install.data.length > 1
          ? t('resourceDownload.alsoInstalled', { names: install.data.slice(1).join(', ') })
          : null
      }
      isBusy={install.isPending}
      doneVersionId={install.isSuccess ? install.variables : undefined}
      actionLabel={t('common:action.install')}
      onPick={(versionId) => install.mutate(versionId)}
    />
  )
}

interface SwitchVersionListProps {
  instanceId: string
  kind: ResourceKind
  project: ProjectHit
  fileName: string
  source: ResourceSource
  onDone?: () => void
}

/** Versions of an installed project; picking one replaces the installed file. */
function SwitchVersionList({ instanceId, kind, project, fileName, source, onDone }: SwitchVersionListProps) {
  const { t } = useTranslation(['instanceDialogs', 'common'])
  const change = useChangeVersion(instanceId, kind, fileName, source)
  return (
    <VersionList
      instanceId={instanceId}
      kind={kind}
      project={project}
      currentVersionId={source.versionId}
      error={change.error}
      note={null}
      isBusy={change.isPending}
      doneVersionId={undefined}
      actionLabel={t('resourceDownload.switch')}
      onPick={(versionId) => change.mutate(versionId, { onSuccess: onDone })}
    />
  )
}

interface ChangeVersionDialogProps {
  instance: InstanceSummary
  kind: ResourceKind
  fileName: string
  source: ResourceSource
  onClose: () => void
}

/** Picks another version of the project an installed file came from and switches to it. */
export function ChangeResourceVersionDialog({
  instance,
  kind,
  fileName,
  source,
  onClose,
}: ChangeVersionDialogProps) {
  const { t } = useTranslation(['instanceDialogs', 'common'])
  const project: ProjectHit = {
    provider: source.provider,
    id: source.projectId,
    slug: source.slug,
    title: source.name,
    description: '',
    author: '',
    iconUrl: null,
    downloads: 0,
  }
  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="flex h-[70vh] flex-col sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t('resourceDownload.changeTitle', { name: source.name })}</DialogTitle>
        </DialogHeader>
        <SwitchVersionList
          instanceId={instance.id}
          kind={kind}
          project={project}
          fileName={fileName}
          source={source}
          onDone={onClose}
        />
      </DialogContent>
    </Dialog>
  )
}

interface VersionListProps {
  instanceId: string
  kind: ResourceKind
  project: ProjectHit
  currentVersionId: string | undefined
  error: Error | null
  note: string | null
  isBusy: boolean
  doneVersionId: string | undefined
  actionLabel: string
  onPick: (versionId: string) => void
}

function VersionList({
  instanceId,
  kind,
  project,
  currentVersionId,
  error: actionError,
  note,
  isBusy,
  doneVersionId,
  actionLabel,
  onPick,
}: VersionListProps) {
  const { t } = useTranslation(['instanceDialogs', 'common'])
  const { data: versions, error, isPending } = useProjectVersions(instanceId, project, kind)

  return (
    <div className="flex min-h-0 flex-col gap-2">
      {actionError && <p className="text-sm text-destructive">{actionError.message}</p>}
      {note && <p className="text-sm text-muted-foreground">{note}</p>}
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
            const installed = version.id === doneVersionId || version.id === currentVersionId
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
                  disabled={isBusy || installed}
                  onClick={() => onPick(version.id)}
                >
                  {installed ? <Check /> : <Download />}
                  {version.id === currentVersionId
                    ? t('resourceDownload.current')
                    : installed
                      ? t('resourceDownload.installed')
                      : actionLabel}
                </Button>
              </li>
            )
          })}
        </ul>
      </div>
    </div>
  )
}
