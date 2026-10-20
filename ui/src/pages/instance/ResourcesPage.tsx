import {
  ArrowUpCircle,
  Download,
  ExternalLink,
  FolderOpen,
  History,
  Lock,
  LockOpen,
  MoreHorizontal,
  RefreshCw,
  TriangleAlert,
  ScanSearch,
  Trash2,
  Unlink,
} from 'lucide-react'
import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { commands, type InstanceSummary, type ResourceFile, type ResourceKind } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { ModListExportMenu } from '@/components/ModListExportMenu'
import { RelatedChangesDialog } from '@/components/RelatedChangesDialog'
import { ResourceFilterMenu } from '@/components/ResourceFilterMenu'
import { SourceBadge } from '@/components/SourceBadge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Input } from '@/components/ui/input'
import { ChangeResourceVersionDialog, ResourceDownloadDialog } from '@/dialogs/ResourceDownloadDialog'
import { ResourceUpdateDialog } from '@/dialogs/ResourceUpdateDialog'
import { openInstanceFolder } from '@/hooks/use-instance-actions'
import { useSettings } from '@/hooks/use-settings'
import {
  useRemoveResource,
  useResources,
  useResourceUpdates,
  useForgetSource,
  useIdentifySources,
  useSetResourceEnabled,
  affectedByToggle,
  useSetUpdateLock,
  useUpdateResources,
} from '@/hooks/use-resources'
import { unwrap } from '@/lib/command'
import { PROVIDER_NAMES } from '@/lib/providers'
import { matchesFilters, NO_FILTERS, type ResourceFilters } from '@/lib/resource-filters'
import { cn } from '@/lib/utils'

interface ResourcesPageProps {
  instance: InstanceSummary
  kind: Exclude<ResourceKind, 'modpack'>
}

export function ResourcesPage({ instance, kind }: ResourcesPageProps) {
  const { t } = useTranslation(['instance', 'common'])
  const { data: files = [], error, isPending, refetch } = useResources(instance.id, kind)
  const setEnabled = useSetResourceEnabled(instance.id, kind)
  const remove = useRemoveResource(instance.id, kind)
  const [search, setSearch] = useState('')
  const [filters, setFilters] = useState<ResourceFilters>(NO_FILTERS)
  const [pendingRemoval, setPendingRemoval] = useState<{ file: ResourceFile; dependents: string[] } | null>(
    null,
  )
  const [pendingToggle, setPendingToggle] = useState<{
    file: ResourceFile
    enable: boolean
    related: string[]
  } | null>(null)
  const nameOf = (fileName: string) => {
    const file = files.find(
      (candidate) => candidate.fileName === fileName || enabledName(candidate) === fileName,
    )
    return file ? displayName(file) : fileName
  }
  const [linkError, setLinkError] = useState<Error | null>(null)

  const toggle = async (file: ResourceFile, enable: boolean) => {
    try {
      const related = await affectedByToggle(instance.id, kind, [file.fileName], enable)
      if (related.length === 0) setEnabled.mutate({ fileNames: [file.fileName], enabled: enable })
      else setPendingToggle({ file, enable, related })
    } catch (error) {
      setLinkError(error instanceof Error ? error : new Error(String(error)))
    }
  }

  const startRemoval = async (file: ResourceFile) => {
    try {
      const dependents = file.enabled ? await affectedByToggle(instance.id, kind, [file.fileName], false) : []
      setPendingRemoval({ file, dependents })
    } catch (error) {
      setLinkError(error instanceof Error ? error : new Error(String(error)))
    }
  }
  const [isDownloading, setIsDownloading] = useState(false)
  const [checkUpdates, setCheckUpdates] = useState(false)
  const [isReviewing, setIsReviewing] = useState(false)
  const updates = useResourceUpdates(instance.id, kind, checkUpdates)
  const identify = useIdentifySources(instance.id, kind)
  const setLock = useSetUpdateLock(instance.id, kind)
  const forgetSource = useForgetSource(instance.id, kind)
  const [changing, setChanging] = useState<ResourceFile | null>(null)
  const { data: settings } = useSettings()
  const tracksSources = settings?.launcher.trackResourceSources ?? true
  const applyUpdates = useUpdateResources(instance.id, kind)
  const updateFor = (file: ResourceFile) => updates.data?.find((update) => update.fileName === file.fileName)

  const visible = useMemo(() => {
    const query = search.trim().toLowerCase()
    const hasUpdate = (file: ResourceFile) =>
      updates.data?.some((update) => update.fileName === file.fileName) ?? false
    return files.filter(
      (file) =>
        (displayName(file).toLowerCase().includes(query) ||
          file.fileName.toLowerCase().includes(query) ||
          (file.source !== null && PROVIDER_NAMES[file.source.provider].toLowerCase().includes(query))) &&
        matchesFilters(file, filters, hasUpdate),
    )
  }, [files, search, filters, updates.data])

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <Input
          className="w-64"
          placeholder={t(`resources.${kind}.search`)}
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
        <ResourceFilterMenu
          filters={filters}
          onChange={setFilters}
          withSources={tracksSources}
          withUpdates={updates.data !== undefined}
        />
        <span className="text-xs whitespace-nowrap text-muted-foreground">
          {t('resources.enabledCount', {
            enabled: files.filter((file) => file.enabled).length,
            total: files.length,
          })}
          {visible.length !== files.length && ` · ${t('resources.shownCount', { shown: visible.length })}`}
        </span>
        <div className="ml-auto flex gap-1">
          <Button size="sm" onClick={() => setIsDownloading(true)}>
            <Download /> {t(`resources.${kind}.download`)}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={updates.isFetching}
            onClick={() => (checkUpdates ? void updates.refetch() : setCheckUpdates(true))}
          >
            <ArrowUpCircle /> {updates.isFetching ? t('resources.checking') : t('resources.checkUpdates')}
          </Button>
          {tracksSources && (
            <Button
              size="sm"
              variant="ghost"
              disabled={identify.isPending}
              onClick={() => identify.mutate(undefined)}
            >
              <ScanSearch />{' '}
              {identify.isPending ? t('resources.identifying') : t('resources.identifySources')}
            </Button>
          )}
          <Button size="sm" variant="ghost" onClick={() => refetch()}>
            <RefreshCw /> {t('common:action.refresh')}
          </Button>
          {kind === 'mod' && <ModListExportMenu instance={instance} />}
          <Button size="sm" variant="ghost" onClick={() => void openInstanceFolder(instance.id, kind)}>
            <FolderOpen /> {t('common:action.openFolder')}
          </Button>
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error.message}</p>}
      {(setEnabled.error ?? linkError) && (
        <p className="text-sm text-destructive">{(setEnabled.error ?? linkError)?.message}</p>
      )}
      {(identify.error ?? setLock.error ?? forgetSource.error) && (
        <p className="text-sm text-destructive">
          {(identify.error ?? setLock.error ?? forgetSource.error)?.message}
        </p>
      )}
      {identify.data && (
        <p className="text-sm text-muted-foreground">{t('resources.identified', identify.data)}</p>
      )}
      {(updates.error ?? applyUpdates.error) && (
        <p className="text-sm text-destructive">{(updates.error ?? applyUpdates.error)?.message}</p>
      )}
      {updates.data && !updates.isFetching && (
        <div className="flex items-center gap-2 rounded-md border px-3 py-2 text-sm">
          {updates.data.length === 0
            ? t(`resources.${kind}.upToDate`)
            : t('resources.updatesAvailable', { count: updates.data.length })}
          {updates.data.length > 0 && (
            <Button
              size="sm"
              className="ml-auto"
              disabled={applyUpdates.isPending}
              onClick={() => setIsReviewing(true)}
            >
              {applyUpdates.isPending ? t('resources.updating') : t('resources.reviewUpdates')}
            </Button>
          )}
        </div>
      )}

      <div className="min-h-0 flex-1 overflow-auto rounded-md border">
        <table className="w-full text-sm">
          <thead className="sticky top-0 bg-background text-left text-muted-foreground">
            <tr className="border-b">
              <th className="w-10 p-2" />
              <th className="p-2 font-medium">{t('columns.name')}</th>
              <th className="p-2 font-medium">{t('columns.version')}</th>
              {tracksSources && <th className="p-2 font-medium">{t('columns.source')}</th>}
              <th className="p-2 font-medium">
                {kind === 'mod' ? t('columns.authors') : t('columns.description')}
              </th>
              <th className="p-2 font-medium">{t('columns.file')}</th>
              <th className="w-10 p-2" />
            </tr>
          </thead>
          <tbody>
            {visible.map((file) => (
              <tr
                key={file.fileName}
                className={cn(
                  'border-b last:border-0 hover:bg-accent/50',
                  !file.enabled && 'text-muted-foreground',
                )}
              >
                <td className="p-2">
                  <Checkbox
                    checked={file.enabled}
                    aria-label={t('item.enable', { name: displayName(file) })}
                    onCheckedChange={(checked) => void toggle(file, checked === true)}
                  />
                </td>
                <td className="p-2" title={file.metadata?.description}>
                  <p className="font-medium">{displayName(file)}</p>
                  {file.requires.length > 0 && (
                    <p className="text-xs text-muted-foreground">
                      {t('resources.requires', { names: file.requires.map(nameOf).join(', ') })}
                    </p>
                  )}
                  {file.requiredBy.length > 0 && (
                    <p className="text-xs text-muted-foreground">
                      {t('resources.requiredBy', { names: file.requiredBy.map(nameOf).join(', ') })}
                    </p>
                  )}
                </td>
                <td className="p-2">
                  {file.incompatible && (
                    <TriangleAlert
                      className="mr-1 inline size-3.5 text-amber-500"
                      aria-label={t('resources.incompatible', { version: instance.minecraftVersion })}
                    >
                      <title>{t('resources.incompatible', { version: instance.minecraftVersion })}</title>
                    </TriangleAlert>
                  )}
                  {file.metadata?.version ?? file.source?.versionNumber}
                  {updateFor(file) && (
                    <button
                      type="button"
                      className="ml-2 inline-flex items-center gap-1 text-xs text-primary hover:underline disabled:opacity-50"
                      disabled={applyUpdates.isPending}
                      onClick={() => {
                        const update = updateFor(file)
                        if (update) applyUpdates.mutate([update])
                      }}
                    >
                      <ArrowUpCircle className="size-3" /> {updateFor(file)?.version.versionNumber}
                    </button>
                  )}
                </td>
                {tracksSources && (
                  <td className="p-2">
                    <span className="inline-flex items-center gap-1.5">
                      <SourceBadge source={file.source} />
                      {file.source?.lockUpdate && (
                        <Lock
                          className="size-3 text-muted-foreground"
                          aria-label={t('resources.updatesLocked')}
                        >
                          <title>{t('resources.updatesLocked')}</title>
                        </Lock>
                      )}
                    </span>
                  </td>
                )}
                <td className="p-2">
                  {kind === 'mod' ? file.metadata?.authors.join(', ') : file.metadata?.description}
                </td>
                <td className="p-2 text-muted-foreground">{file.fileName}</td>
                <td className="p-2">
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <button
                        type="button"
                        aria-label={t('resources.moreActions')}
                        className="text-muted-foreground hover:text-foreground"
                      >
                        <MoreHorizontal className="size-4" />
                      </button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end" className="w-max">
                      {file.source && (
                        <>
                          <DropdownMenuItem
                            onSelect={() => void unwrap(commands.openUrl(file.source!.homepage))}
                          >
                            <ExternalLink />
                            {t('resources.viewOn', { provider: PROVIDER_NAMES[file.source.provider] })}
                          </DropdownMenuItem>
                          <DropdownMenuItem onSelect={() => setChanging(file)}>
                            <History /> {t('resources.changeVersion')}
                          </DropdownMenuItem>
                          <DropdownMenuItem
                            onSelect={() =>
                              setLock.mutate({ fileName: file.fileName, locked: !file.source?.lockUpdate })
                            }
                          >
                            {file.source.lockUpdate ? <LockOpen /> : <Lock />}
                            {file.source.lockUpdate
                              ? t('resources.unlockUpdates')
                              : t('resources.lockUpdates')}
                          </DropdownMenuItem>
                          <DropdownMenuItem onSelect={() => forgetSource.mutate(file.fileName)}>
                            <Unlink /> {t('resources.forgetSource')}
                          </DropdownMenuItem>
                          <DropdownMenuSeparator />
                        </>
                      )}
                      <DropdownMenuItem variant="destructive" onSelect={() => void startRemoval(file)}>
                        <Trash2 /> {t('common:action.remove')}
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {isPending && <p className="p-3 text-sm text-muted-foreground">{t('resources.reading')}</p>}
        {!isPending && files.length === 0 && (
          <p className="p-3 text-sm text-muted-foreground">{t(`resources.${kind}.empty`)}</p>
        )}
        {files.length > 0 && visible.length === 0 && (
          <p className="p-3 text-sm text-muted-foreground">{t('resources.nothingMatches')}</p>
        )}
      </div>

      {pendingRemoval && pendingRemoval.dependents.length === 0 && (
        <ConfirmDialog
          title={t('confirm.remove', { name: displayName(pendingRemoval.file) })}
          description={t('resources.removeDescription', { file: pendingRemoval.file.fileName })}
          confirmLabel={t('common:action.remove')}
          onConfirm={() =>
            remove.mutate(pendingRemoval.file.fileName, { onSuccess: () => setPendingRemoval(null) })
          }
          onClose={() => setPendingRemoval(null)}
        />
      )}
      {pendingRemoval && pendingRemoval.dependents.length > 0 && (
        <RelatedChangesDialog
          title={t('resources.removeRelatedTitle', { name: displayName(pendingRemoval.file) })}
          description={t('resources.removeRelated', { name: displayName(pendingRemoval.file) })}
          names={pendingRemoval.dependents.map(nameOf)}
          withRelatedLabel={t('resources.disableAndRemove')}
          onlySelectedLabel={t('resources.onlyRemove')}
          onWithRelated={() => {
            const { file, dependents } = pendingRemoval
            setPendingRemoval(null)
            setEnabled.mutate(
              { fileNames: dependents, enabled: false },
              { onSuccess: () => remove.mutate(file.fileName) },
            )
          }}
          onOnlySelected={() => {
            remove.mutate(pendingRemoval.file.fileName)
            setPendingRemoval(null)
          }}
          onClose={() => setPendingRemoval(null)}
        />
      )}
      {pendingToggle && (
        <RelatedChangesDialog
          title={t(pendingToggle.enable ? 'resources.relatedEnableTitle' : 'resources.relatedDisableTitle')}
          description={t(pendingToggle.enable ? 'resources.relatedEnable' : 'resources.relatedDisable', {
            name: displayName(pendingToggle.file),
          })}
          names={pendingToggle.related.map(nameOf)}
          withRelatedLabel={t('resources.toggleRelated')}
          onlySelectedLabel={t('resources.toggleOnlyThis')}
          onWithRelated={() => {
            const { file, enable, related } = pendingToggle
            setPendingToggle(null)
            setEnabled.mutate({ fileNames: [file.fileName, ...related], enabled: enable })
          }}
          onOnlySelected={() => {
            const { file, enable } = pendingToggle
            setPendingToggle(null)
            setEnabled.mutate({ fileNames: [file.fileName], enabled: enable })
          }}
          onClose={() => setPendingToggle(null)}
        />
      )}
      {changing?.source && (
        <ChangeResourceVersionDialog
          instance={instance}
          kind={kind}
          fileName={changing.fileName}
          source={changing.source}
          onClose={() => setChanging(null)}
        />
      )}
      {isReviewing && updates.data && (
        <ResourceUpdateDialog
          updates={updates.data}
          files={files}
          onApply={(selected) => {
            setIsReviewing(false)
            applyUpdates.mutate(selected)
          }}
          onClose={() => setIsReviewing(false)}
        />
      )}
      <ResourceDownloadDialog
        instance={instance}
        kind={kind}
        open={isDownloading}
        onOpenChange={setIsDownloading}
      />
    </div>
  )
}

function displayName(file: ResourceFile) {
  return file.metadata?.name ?? file.source?.name ?? file.fileName
}

function enabledName(file: ResourceFile) {
  return file.fileName.replace(/\.disabled$/, '')
}
