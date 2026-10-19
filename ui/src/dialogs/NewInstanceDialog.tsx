import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { commands, type InstalledPack, type LoaderSpec, type Provider } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { ImportSource } from '@/components/ImportSource'
import { LoaderPicker } from '@/components/LoaderPicker'
import { ModpackBrowser } from '@/components/ModpackBrowser'
import { VersionList } from '@/components/VersionList'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Progress } from '@/components/ui/progress'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { useCreateInstance } from '@/hooks/use-create-instance'
import { BlockedFilesDialog } from '@/dialogs/BlockedFilesDialog'
import { OptionalModsDialog } from '@/dialogs/OptionalModsDialog'
import { useModpackProgress } from '@/hooks/use-modpack-progress'
import {
  type ModpackSelection,
  useImportInstance,
  useInstallModpack,
  useOptionalMods,
} from '@/hooks/use-modpacks'
import { failure, untrustedHosts } from '@/lib/command'

type Source = 'vanilla' | Provider | 'import'

interface NewInstanceDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Opens the Import tab with this file or URL filled in; an empty string opens it blank. */
  initialImport?: string
}

export function NewInstanceDialog({ open, onOpenChange, initialImport }: NewInstanceDialogProps) {
  const { t } = useTranslation(['newInstance', 'common'])
  const [source, setSource] = useState<Source>(initialImport === undefined ? 'vanilla' : 'import')
  const [name, setName] = useState('')
  const [group, setGroup] = useState('')
  const [version, setVersion] = useState<string | null>(null)
  const [loader, setLoader] = useState<LoaderSpec | null>(null)
  const [modpack, setModpack] = useState<ModpackSelection | null>(null)
  const [pendingDownloads, setPendingDownloads] = useState<InstalledPack | null>(null)
  const [importSource, setImportSource] = useState(initialImport ?? '')
  const [isChoosingMods, setIsChoosingMods] = useState(false)
  const [optionalChoice, setOptionalChoice] = useState<string[] | null>(null)
  const optionalMods = useOptionalMods(modpack?.provider === 'atLauncher' ? modpack : null)
  const createInstance = useCreateInstance()
  const installModpack = useInstallModpack()
  const importInstance = useImportInstance()
  const downloading = installModpack.isPending || importInstance.isPending
  const progress = useModpackProgress(downloading)
  const busy = createInstance.isPending || downloading
  const packError = installModpack.error ?? importInstance.error
  const hostsToTrust = untrustedHosts(packError)
  const [trustDeclined, setTrustDeclined] = useState(false)
  const error = failure(createInstance.error ?? packError) ?? (trustDeclined ? packError : null)

  const close = () => {
    if (busy) return
    setName('')
    setGroup('')
    setVersion(null)
    setLoader(null)
    setModpack(null)
    setOptionalChoice(null)
    setPendingDownloads(null)
    setImportSource('')
    createInstance.reset()
    installModpack.reset()
    importInstance.reset()
    onOpenChange(false)
  }

  const canCreate =
    source === 'vanilla'
      ? version !== null && (loader === null || loader.version !== '')
      : source === 'import'
        ? importSource.trim() !== ''
        : modpack?.provider === source

  const finish = (installed: InstalledPack) =>
    installed.blocked.length > 0 ? setPendingDownloads(installed) : close()

  const create = (allowUntrustedHosts = false, chosenMods = optionalChoice) => {
    const options = { name: name.trim() || null, allowUntrustedHosts, optionalMods: chosenMods }
    setTrustDeclined(false)
    if (source === 'import') {
      importInstance.mutate({ source: importSource, options }, { onSuccess: finish })
    } else if (source !== 'vanilla' && modpack) {
      if (chosenMods === null && optionalMods.data && optionalMods.data.length > 0) {
        setIsChoosingMods(true)
        return
      }
      installModpack.mutate({ selection: modpack, options }, { onSuccess: finish })
    } else if (version) {
      createInstance.mutate(
        {
          name: name.trim() || version,
          minecraftVersion: version,
          loader,
          group: group || null,
          iconKey: null,
        },
        { onSuccess: close },
      )
    }
  }

  return (
    <Dialog open={open} onOpenChange={(next) => (next ? onOpenChange(true) : close())}>
      <DialogContent className="flex h-[85vh] flex-col sm:max-w-4xl">
        <DialogHeader className="flex-row items-center gap-4">
          <DialogTitle>{t('title')}</DialogTitle>
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            value={source}
            onValueChange={(next) => next && setSource(next as Source)}
          >
            <ToggleGroupItem value="vanilla">{t('source.custom')}</ToggleGroupItem>
            <ToggleGroupItem value="modrinth">Modrinth</ToggleGroupItem>
            <ToggleGroupItem value="curseForge">CurseForge</ToggleGroupItem>
            <ToggleGroupItem value="ftb">FTB</ToggleGroupItem>
            <ToggleGroupItem value="technic">Technic</ToggleGroupItem>
            <ToggleGroupItem value="atLauncher">ATLauncher</ToggleGroupItem>
            <ToggleGroupItem value="import">{t('source.import')}</ToggleGroupItem>
          </ToggleGroup>
        </DialogHeader>

        <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-2">
          <Label htmlFor="instance-name">{t('field.name')}</Label>
          <Input
            id="instance-name"
            placeholder={
              source === 'vanilla'
                ? (version ?? t('placeholder.instanceName'))
                : t('placeholder.nameFromPack')
            }
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
          {source === 'vanilla' && (
            <>
              <Label htmlFor="instance-group">{t('field.group')}</Label>
              <Input
                id="instance-group"
                placeholder={t('placeholder.noGroup')}
                value={group}
                onChange={(event) => setGroup(event.target.value)}
              />
            </>
          )}
        </div>

        {source === 'vanilla' ? (
          <>
            <VersionList selected={version} onSelect={setVersion} />
            <LoaderPicker minecraftVersion={version} value={loader} onChange={setLoader} />
          </>
        ) : source !== 'import' ? (
          <ModpackBrowser
            key={source}
            provider={source}
            selected={modpack}
            onSelect={(selection) => {
              setModpack(selection)
              setOptionalChoice(null)
            }}
          />
        ) : (
          <ImportSource value={importSource} onChange={setImportSource} />
        )}

        {downloading && (
          <div className="flex flex-col gap-1">
            <p className="text-sm text-muted-foreground">
              {progress
                ? t('progress.downloading', { completed: progress.completed, total: progress.total })
                : t('progress.preparing')}
            </p>
            {progress && progress.total > 0 && (
              <Progress value={(progress.completed / progress.total) * 100} />
            )}
          </div>
        )}
        {error && <p className="text-sm text-destructive">{error.message}</p>}
        <DialogFooter>
          <Button
            variant="ghost"
            disabled={createInstance.isPending}
            onClick={downloading ? () => void commands.cancelModpackInstall() : close}
          >
            {t('common:action.cancel')}
          </Button>
          <Button onClick={() => create()} disabled={!canCreate || busy || optionalMods.isFetching}>
            {t('common:action.create')}
          </Button>
        </DialogFooter>
      </DialogContent>
      {hostsToTrust && !trustDeclined && (
        <ConfirmDialog
          title={t('untrustedHosts.title')}
          description={t('untrustedHosts.description', { hosts: hostsToTrust.join(', ') })}
          confirmLabel={t('untrustedHosts.confirm')}
          onConfirm={() => create(true)}
          onClose={() => setTrustDeclined(true)}
        />
      )}
      {isChoosingMods && modpack && optionalMods.data && (
        <OptionalModsDialog
          selection={modpack}
          mods={optionalMods.data}
          onConfirm={(chosen) => {
            setIsChoosingMods(false)
            setOptionalChoice(chosen)
            create(false, chosen)
          }}
          onClose={() => setIsChoosingMods(false)}
        />
      )}
      {pendingDownloads && <BlockedFilesDialog installed={pendingDownloads} onClose={close} />}
    </Dialog>
  )
}
