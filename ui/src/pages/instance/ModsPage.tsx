import { Download, FolderOpen, RefreshCw, Trash2 } from 'lucide-react'
import { useMemo, useState } from 'react'
import type { InstanceSummary, ModFile } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { ResourceDownloadDialog } from '@/dialogs/ResourceDownloadDialog'
import { openInstanceFolder } from '@/hooks/use-instance-actions'
import { useMods, useRemoveMod, useSetModEnabled } from '@/hooks/use-mods'
import { cn } from '@/lib/utils'

export function ModsPage({ instance }: { instance: InstanceSummary }) {
  const instanceId = instance.id
  const { data: mods = [], error, isPending, refetch } = useMods(instanceId)
  const setEnabled = useSetModEnabled(instanceId)
  const removeMod = useRemoveMod(instanceId)
  const [search, setSearch] = useState('')
  const [pendingRemoval, setPendingRemoval] = useState<ModFile | null>(null)
  const [isDownloading, setIsDownloading] = useState(false)

  const visible = useMemo(() => {
    const query = search.trim().toLowerCase()
    return mods.filter(
      (mod) => modName(mod).toLowerCase().includes(query) || mod.fileName.toLowerCase().includes(query),
    )
  }, [mods, search])

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex items-center gap-2">
        <Input
          className="w-64"
          placeholder="Search mods"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <span className="text-xs text-muted-foreground">
          {mods.filter((mod) => mod.enabled).length} of {mods.length} enabled
        </span>
        <div className="ml-auto flex gap-1">
          <Button size="sm" onClick={() => setIsDownloading(true)}>
            <Download /> Download Mods
          </Button>
          <Button size="sm" variant="ghost" onClick={() => refetch()}>
            <RefreshCw /> Refresh
          </Button>
          <Button size="sm" variant="ghost" onClick={() => void openInstanceFolder(instanceId, 'mods')}>
            <FolderOpen /> Open Folder
          </Button>
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error.message}</p>}
      {setEnabled.error && <p className="text-sm text-destructive">{setEnabled.error.message}</p>}

      <div className="min-h-0 flex-1 overflow-auto rounded-md border">
        <table className="w-full text-sm">
          <thead className="sticky top-0 bg-background text-left text-muted-foreground">
            <tr className="border-b">
              <th className="w-10 p-2" />
              <th className="p-2 font-medium">Name</th>
              <th className="p-2 font-medium">Version</th>
              <th className="p-2 font-medium">Authors</th>
              <th className="p-2 font-medium">File</th>
              <th className="w-10 p-2" />
            </tr>
          </thead>
          <tbody>
            {visible.map((mod) => (
              <tr
                key={mod.fileName}
                className={cn(
                  'border-b last:border-0 hover:bg-accent/50',
                  !mod.enabled && 'text-muted-foreground',
                )}
              >
                <td className="p-2">
                  <Checkbox
                    checked={mod.enabled}
                    aria-label={`Enable ${modName(mod)}`}
                    onCheckedChange={(checked) =>
                      setEnabled.mutate({ fileName: mod.fileName, enabled: checked === true })
                    }
                  />
                </td>
                <td className="p-2 font-medium" title={mod.metadata?.description}>
                  {modName(mod)}
                </td>
                <td className="p-2">{mod.metadata?.version}</td>
                <td className="p-2">{mod.metadata?.authors.join(', ')}</td>
                <td className="p-2 text-muted-foreground">{mod.fileName}</td>
                <td className="p-2">
                  <button
                    type="button"
                    aria-label={`Remove ${modName(mod)}`}
                    className="text-muted-foreground hover:text-destructive"
                    onClick={() => setPendingRemoval(mod)}
                  >
                    <Trash2 className="size-4" />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {isPending && <p className="p-3 text-sm text-muted-foreground">Reading mods…</p>}
        {!isPending && mods.length === 0 && (
          <p className="p-3 text-sm text-muted-foreground">
            No mods installed. Put .jar files into the mods folder.
          </p>
        )}
      </div>

      {pendingRemoval && (
        <ConfirmDialog
          title={`Remove ${modName(pendingRemoval)}?`}
          description={`${pendingRemoval.fileName} will be deleted from the mods folder.`}
          confirmLabel="Remove"
          onConfirm={() =>
            removeMod.mutate(pendingRemoval.fileName, { onSuccess: () => setPendingRemoval(null) })
          }
          onClose={() => setPendingRemoval(null)}
        />
      )}

      <ResourceDownloadDialog
        instance={instance}
        kind="mod"
        open={isDownloading}
        onOpenChange={setIsDownloading}
      />
    </div>
  )
}

function modName(mod: ModFile) {
  return mod.metadata?.name ?? mod.fileName
}
