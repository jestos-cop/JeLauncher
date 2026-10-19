import { RefreshCw } from 'lucide-react'
import { useState } from 'react'
import { useSession } from '@/app/sessions'
import type { InstanceSummary, ManagedPack } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { useModpackProgress } from '@/hooks/use-modpack-progress'
import { useModpackVersions, useUpdateModpack } from '@/hooks/use-modpacks'
import { cn } from '@/lib/utils'

export function ModpackPage({ instance, pack }: { instance: InstanceSummary; pack: ManagedPack }) {
  const { data: versions = [], error, isPending } = useModpackVersions(pack.projectId)
  const update = useUpdateModpack(instance.id)
  const progress = useModpackProgress(update.isPending)
  const { session } = useSession(instance.id)
  const [selectedId, setSelectedId] = useState(pack.versionId)
  const [isConfirming, setIsConfirming] = useState(false)
  const selected = versions.find((version) => version.id === selectedId)
  const latest = versions.find((version) => version.releaseType === 'release') ?? versions[0]

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex items-center gap-3">
        <div>
          <p className="font-medium">{pack.name}</p>
          <p className="text-sm text-muted-foreground">
            Version {pack.version} from Modrinth
            {latest && latest.id !== pack.versionId && ` · ${latest.versionNumber} is available`}
          </p>
        </div>
        <Button
          size="sm"
          className="ml-auto"
          disabled={
            !selected || selected.id === pack.versionId || update.isPending || session.status !== 'idle'
          }
          onClick={() => setIsConfirming(true)}
        >
          <RefreshCw />{' '}
          {selected && selected.id !== pack.versionId
            ? `Switch to ${selected.versionNumber}`
            : 'Switch version'}
        </Button>
      </div>

      {update.isPending && (
        <div className="flex flex-col gap-1">
          <p className="text-sm text-muted-foreground">
            {progress ? `Downloading files ${progress.completed}/${progress.total}` : 'Downloading modpack…'}
          </p>
          {progress && progress.total > 0 && <Progress value={(progress.completed / progress.total) * 100} />}
        </div>
      )}
      {(error ?? update.error) && (
        <p className="text-sm text-destructive">{(error ?? update.error)?.message}</p>
      )}

      <div className="grid min-h-0 flex-1 grid-cols-[16rem_1fr] gap-3">
        <ul className="min-h-0 overflow-auto rounded-md border">
          {isPending && <li className="p-3 text-sm text-muted-foreground">Loading versions…</li>}
          {versions.map((version) => (
            <li key={version.id}>
              <button
                type="button"
                onClick={() => setSelectedId(version.id)}
                className={cn(
                  'flex w-full items-center gap-2 p-2 text-left hover:bg-accent',
                  version.id === selectedId && 'bg-accent',
                )}
              >
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{version.versionNumber}</p>
                  <p className="truncate text-xs text-muted-foreground">
                    {new Date(version.published).toLocaleDateString()}
                  </p>
                </div>
                {version.id === pack.versionId && <Badge>Installed</Badge>}
                {version.releaseType !== 'release' && (
                  <Badge variant="secondary">{version.releaseType}</Badge>
                )}
              </button>
            </li>
          ))}
        </ul>
        <div className="min-h-0 overflow-auto rounded-md border p-3">
          {selected ? (
            <>
              <h3 className="mb-2 font-medium">{selected.name}</h3>
              <p className="text-sm whitespace-pre-wrap text-muted-foreground">
                {selected.changelog ?? 'No changelog for this version.'}
              </p>
            </>
          ) : (
            <p className="text-sm text-muted-foreground">Select a version to see its changelog.</p>
          )}
        </div>
      </div>

      {isConfirming && selected && (
        <ConfirmDialog
          title={`Switch to ${selected.versionNumber}?`}
          description="Files from the current pack version are replaced. Worlds, screenshots and your own files are kept."
          confirmLabel="Switch"
          onConfirm={() => {
            setIsConfirming(false)
            update.mutate(selected.id)
          }}
          onClose={() => setIsConfirming(false)}
        />
      )}
    </div>
  )
}
