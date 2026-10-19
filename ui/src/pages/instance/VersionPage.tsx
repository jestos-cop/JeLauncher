import { FileCode, Pencil, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { InstanceSummary } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { Button } from '@/components/ui/button'
import { ChangeVersionDialog } from '@/dialogs/ChangeVersionDialog'
import { useOpenInstanceFile, useSetInstanceVersion } from '@/hooks/use-instance-actions'

const LOADER_NAMES = {
  fabric: 'Fabric Loader',
  quilt: 'Quilt Loader',
  forge: 'Forge',
  neoforge: 'NeoForge',
} as const

export function VersionPage({ instance }: { instance: InstanceSummary }) {
  const { t } = useTranslation(['instance', 'common'])
  const [dialog, setDialog] = useState<'change' | 'removeLoader' | null>(null)
  const setInstanceVersion = useSetInstanceVersion()
  const openFile = useOpenInstanceFile()
  const close = () => setDialog(null)
  const { loader } = instance

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-sm text-muted-foreground">{t('version.components')}</span>
        <Button
          size="sm"
          variant="ghost"
          className="ml-auto"
          onClick={() => openFile.mutate({ id: instance.id, file: 'config' })}
        >
          <FileCode /> {t('version.editFile', { file: 'instance.json' })}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          onClick={() => openFile.mutate({ id: instance.id, file: 'gameOptions' })}
        >
          <FileCode /> {t('version.editFile', { file: 'options.txt' })}
        </Button>
        <Button size="sm" variant="ghost" onClick={() => setDialog('change')}>
          <Pencil /> {loader ? t('version.change') : t('version.changeOrInstallLoader')}
        </Button>
      </div>

      {openFile.error && <p className="text-sm text-destructive">{openFile.error.message}</p>}
      {setInstanceVersion.error && (
        <p className="text-sm text-destructive">{setInstanceVersion.error.message}</p>
      )}

      <div className="rounded-md border">
        <table className="w-full text-sm">
          <thead className="text-left text-muted-foreground">
            <tr className="border-b">
              <th className="p-2 font-medium">{t('columns.name')}</th>
              <th className="p-2 font-medium">{t('columns.version')}</th>
              <th className="w-10 p-2" />
            </tr>
          </thead>
          <tbody>
            <tr className="border-b last:border-0">
              <td className="p-2 font-medium">Minecraft</td>
              <td className="p-2">{instance.minecraftVersion}</td>
              <td className="p-2" />
            </tr>
            {loader && (
              <tr>
                <td className="p-2 font-medium">{LOADER_NAMES[loader.kind]}</td>
                <td className="p-2">{loader.version}</td>
                <td className="p-2">
                  <button
                    type="button"
                    title={t('version.removeLoader')}
                    aria-label={t('version.removeLoader')}
                    className="text-muted-foreground hover:text-destructive"
                    onClick={() => setDialog('removeLoader')}
                  >
                    <Trash2 className="size-4" />
                  </button>
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {dialog === 'change' && <ChangeVersionDialog instance={instance} onClose={close} />}
      {dialog === 'removeLoader' && loader && (
        <ConfirmDialog
          title={t('confirm.remove', { name: LOADER_NAMES[loader.kind] })}
          description={t('version.removeLoaderDescription')}
          confirmLabel={t('common:action.remove')}
          onConfirm={() =>
            setInstanceVersion.mutate(
              { id: instance.id, minecraftVersion: instance.minecraftVersion, loader: null },
              { onSuccess: close },
            )
          }
          onClose={close}
        />
      )}
    </div>
  )
}
