import { ArrowUpCircle, Download, FolderOpen, RefreshCw, Trash2 } from 'lucide-react'
import { useMemo, useState } from 'react'
import type { InstanceSummary, ResourceFile, ResourceKind } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { ResourceDownloadDialog } from '@/dialogs/ResourceDownloadDialog'
import { openInstanceFolder } from '@/hooks/use-instance-actions'
import {
  useRemoveResource,
  useResources,
  useResourceUpdates,
  useSetResourceEnabled,
  useUpdateResources,
} from '@/hooks/use-resources'
import { cn } from '@/lib/utils'

const LABELS: Record<Exclude<ResourceKind, 'modpack'>, { plural: string; empty: string }> = {
  mod: { plural: 'Mods', empty: 'No mods installed. Put .jar files into the mods folder.' },
  resourcePack: { plural: 'Resource Packs', empty: 'No resource packs installed.' },
  shaderPack: { plural: 'Shader Packs', empty: 'No shader packs installed.' },
  dataPack: { plural: 'Data Packs', empty: 'No data packs installed.' },
}

interface ResourcesPageProps {
  instance: InstanceSummary
  kind: Exclude<ResourceKind, 'modpack'>
}

export function ResourcesPage({ instance, kind }: ResourcesPageProps) {
  const { data: files = [], error, isPending, refetch } = useResources(instance.id, kind)
  const setEnabled = useSetResourceEnabled(instance.id, kind)
  const remove = useRemoveResource(instance.id, kind)
  const [search, setSearch] = useState('')
  const [pendingRemoval, setPendingRemoval] = useState<ResourceFile | null>(null)
  const [isDownloading, setIsDownloading] = useState(false)
  const [checkUpdates, setCheckUpdates] = useState(false)
  const updates = useResourceUpdates(instance.id, kind, checkUpdates)
  const applyUpdates = useUpdateResources(instance.id, kind)
  const updateFor = (file: ResourceFile) => updates.data?.find((update) => update.fileName === file.fileName)
  const label = LABELS[kind]

  const visible = useMemo(() => {
    const query = search.trim().toLowerCase()
    return files.filter(
      (file) =>
        displayName(file).toLowerCase().includes(query) || file.fileName.toLowerCase().includes(query),
    )
  }, [files, search])

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex items-center gap-2">
        <Input
          className="w-64"
          placeholder={`Search ${label.plural.toLowerCase()}`}
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
        <span className="text-xs text-muted-foreground">
          {files.filter((file) => file.enabled).length} of {files.length} enabled
        </span>
        <div className="ml-auto flex gap-1">
          <Button size="sm" onClick={() => setIsDownloading(true)}>
            <Download /> Download {label.plural}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={updates.isFetching}
            onClick={() => (checkUpdates ? void updates.refetch() : setCheckUpdates(true))}
          >
            <ArrowUpCircle /> {updates.isFetching ? 'Checking…' : 'Check for Updates'}
          </Button>
          <Button size="sm" variant="ghost" onClick={() => refetch()}>
            <RefreshCw /> Refresh
          </Button>
          <Button size="sm" variant="ghost" onClick={() => void openInstanceFolder(instance.id, kind)}>
            <FolderOpen /> Open Folder
          </Button>
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error.message}</p>}
      {setEnabled.error && <p className="text-sm text-destructive">{setEnabled.error.message}</p>}
      {(updates.error ?? applyUpdates.error) && (
        <p className="text-sm text-destructive">{(updates.error ?? applyUpdates.error)?.message}</p>
      )}
      {updates.data && !updates.isFetching && (
        <div className="flex items-center gap-2 rounded-md border px-3 py-2 text-sm">
          {updates.data.length === 0
            ? `All ${label.plural.toLowerCase()} from Modrinth are up to date.`
            : `${updates.data.length} update${updates.data.length === 1 ? '' : 's'} available.`}
          {updates.data.length > 0 && (
            <Button
              size="sm"
              className="ml-auto"
              disabled={applyUpdates.isPending}
              onClick={() => applyUpdates.mutate(updates.data)}
            >
              {applyUpdates.isPending ? 'Updating…' : 'Update All'}
            </Button>
          )}
        </div>
      )}

      <div className="min-h-0 flex-1 overflow-auto rounded-md border">
        <table className="w-full text-sm">
          <thead className="sticky top-0 bg-background text-left text-muted-foreground">
            <tr className="border-b">
              <th className="w-10 p-2" />
              <th className="p-2 font-medium">Name</th>
              <th className="p-2 font-medium">Version</th>
              <th className="p-2 font-medium">{kind === 'mod' ? 'Authors' : 'Description'}</th>
              <th className="p-2 font-medium">File</th>
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
                    aria-label={`Enable ${displayName(file)}`}
                    onCheckedChange={(checked) =>
                      setEnabled.mutate({ fileName: file.fileName, enabled: checked === true })
                    }
                  />
                </td>
                <td className="p-2 font-medium" title={file.metadata?.description}>
                  {displayName(file)}
                </td>
                <td className="p-2">
                  {file.metadata?.version}
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
                <td className="p-2">
                  {kind === 'mod' ? file.metadata?.authors.join(', ') : file.metadata?.description}
                </td>
                <td className="p-2 text-muted-foreground">{file.fileName}</td>
                <td className="p-2">
                  <button
                    type="button"
                    aria-label={`Remove ${displayName(file)}`}
                    className="text-muted-foreground hover:text-destructive"
                    onClick={() => setPendingRemoval(file)}
                  >
                    <Trash2 className="size-4" />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {isPending && <p className="p-3 text-sm text-muted-foreground">Reading files…</p>}
        {!isPending && files.length === 0 && (
          <p className="p-3 text-sm text-muted-foreground">{label.empty}</p>
        )}
      </div>

      {pendingRemoval && (
        <ConfirmDialog
          title={`Remove ${displayName(pendingRemoval)}?`}
          description={`${pendingRemoval.fileName} will be deleted from the instance.`}
          confirmLabel="Remove"
          onConfirm={() =>
            remove.mutate(pendingRemoval.fileName, { onSuccess: () => setPendingRemoval(null) })
          }
          onClose={() => setPendingRemoval(null)}
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
  return file.metadata?.name ?? file.fileName
}
