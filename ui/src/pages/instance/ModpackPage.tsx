import { RefreshCw } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useSession } from '@/app/sessions'
import type { InstanceSummary, ManagedPack } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { useModpackProgress } from '@/hooks/use-modpack-progress'
import { useModpackVersions, useUpdateModpack } from '@/hooks/use-modpacks'
import { PROVIDER_NAMES } from '@/lib/providers'
import { cn } from '@/lib/utils'

export function ModpackPage({ instance, pack }: { instance: InstanceSummary; pack: ManagedPack }) {
  const { t } = useTranslation('instance')
  const { data: versions = [], error, isPending } = useModpackVersions(pack.provider, pack.projectId)
  const update = useUpdateModpack(instance.id)
  const progress = useModpackProgress(update.isPending)
  const { session } = useSession(instance.id)
  const [selectedId, setSelectedId] = useState(pack.versionId)
  const [isConfirming, setIsConfirming] = useState(false)
  const selected = versions.find((version) => version.id === selectedId)
  const canSwitch = pack.provider === 'modrinth'
  const latest = versions.find((version) => version.releaseType === 'release') ?? versions[0]

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex items-center gap-3">
        <div>
          <p className="font-medium">{pack.name}</p>
          <p className="text-sm text-muted-foreground">
            {latest && latest.id !== pack.versionId
              ? t('modpack.sourceWithUpdate', {
                  version: pack.version,
                  provider: PROVIDER_NAMES[pack.provider],
                  latest: latest.versionNumber,
                })
              : t('modpack.source', { version: pack.version, provider: PROVIDER_NAMES[pack.provider] })}
          </p>
        </div>
        <Button
          size="sm"
          className="ml-auto"
          disabled={
            !canSwitch ||
            !selected ||
            selected.id === pack.versionId ||
            update.isPending ||
            session.status !== 'idle'
          }
          onClick={() => setIsConfirming(true)}
        >
          <RefreshCw />{' '}
          {selected && selected.id !== pack.versionId
            ? t('modpack.switchTo', { version: selected.versionNumber })
            : t('modpack.switchVersion')}
        </Button>
      </div>

      {!canSwitch && (
        <p className="text-sm text-muted-foreground">
          {t('modpack.switchUnsupported', { provider: PROVIDER_NAMES[pack.provider] })}
        </p>
      )}
      {update.isPending && (
        <div className="flex flex-col gap-1">
          <p className="text-sm text-muted-foreground">
            {progress
              ? t('modpack.downloadingFiles', { completed: progress.completed, total: progress.total })
              : t('modpack.downloading')}
          </p>
          {progress && progress.total > 0 && <Progress value={(progress.completed / progress.total) * 100} />}
        </div>
      )}
      {(error ?? update.error) && (
        <p className="text-sm text-destructive">{(error ?? update.error)?.message}</p>
      )}

      <div className="grid min-h-0 flex-1 grid-cols-[16rem_1fr] gap-3">
        <ul className="min-h-0 overflow-auto rounded-md border">
          {isPending && <li className="p-3 text-sm text-muted-foreground">{t('modpack.loadingVersions')}</li>}
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
                {version.id === pack.versionId && <Badge>{t('modpack.installed')}</Badge>}
                {version.releaseType !== 'release' && (
                  <Badge variant="secondary">{t(`modpack.releaseType.${version.releaseType}`)}</Badge>
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
                {selected.changelog ?? t('modpack.noChangelog')}
              </p>
            </>
          ) : (
            <p className="text-sm text-muted-foreground">{t('modpack.selectVersion')}</p>
          )}
        </div>
      </div>

      {isConfirming && selected && (
        <ConfirmDialog
          title={t('modpack.confirmTitle', { version: selected.versionNumber })}
          description={t('modpack.confirmDescription')}
          confirmLabel={t('modpack.confirmSwitch')}
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
