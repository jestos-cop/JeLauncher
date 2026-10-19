import { useState } from 'react'
import type { LoaderSpec } from '@/bindings'
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
import { useModpackProgress } from '@/hooks/use-modpack-progress'
import { useImportInstance, useInstallModpack } from '@/hooks/use-modpacks'

type Source = 'vanilla' | 'modrinth' | 'import'

interface NewInstanceDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function NewInstanceDialog({ open, onOpenChange }: NewInstanceDialogProps) {
  const [source, setSource] = useState<Source>('vanilla')
  const [name, setName] = useState('')
  const [group, setGroup] = useState('')
  const [version, setVersion] = useState<string | null>(null)
  const [loader, setLoader] = useState<LoaderSpec | null>(null)
  const [modpackVersion, setModpackVersion] = useState<string | null>(null)
  const [importSource, setImportSource] = useState('')
  const createInstance = useCreateInstance()
  const installModpack = useInstallModpack()
  const importInstance = useImportInstance()
  const downloading = installModpack.isPending || importInstance.isPending
  const progress = useModpackProgress(downloading)
  const busy = createInstance.isPending || downloading
  const error = createInstance.error ?? installModpack.error ?? importInstance.error

  const close = () => {
    if (busy) return
    setName('')
    setGroup('')
    setVersion(null)
    setLoader(null)
    setModpackVersion(null)
    setImportSource('')
    createInstance.reset()
    installModpack.reset()
    importInstance.reset()
    onOpenChange(false)
  }

  const canCreate = {
    vanilla: version !== null && (loader === null || loader.version !== ''),
    modrinth: modpackVersion !== null,
    import: importSource.trim() !== '',
  }[source]

  const create = () => {
    if (source === 'import') {
      importInstance.mutate({ source: importSource, name: name.trim() || null }, { onSuccess: close })
    } else if (source === 'modrinth' && modpackVersion) {
      installModpack.mutate({ versionId: modpackVersion, name: name.trim() || null }, { onSuccess: close })
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
          <DialogTitle>New Instance</DialogTitle>
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            value={source}
            onValueChange={(next) => next && setSource(next as Source)}
          >
            <ToggleGroupItem value="vanilla">Custom</ToggleGroupItem>
            <ToggleGroupItem value="modrinth">Modrinth</ToggleGroupItem>
            <ToggleGroupItem value="import">Import</ToggleGroupItem>
          </ToggleGroup>
        </DialogHeader>

        <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-2">
          <Label htmlFor="instance-name">Name</Label>
          <Input
            id="instance-name"
            placeholder={source === 'vanilla' ? (version ?? 'Instance name') : 'Name from the pack'}
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
          {source === 'vanilla' && (
            <>
              <Label htmlFor="instance-group">Group</Label>
              <Input
                id="instance-group"
                placeholder="No group"
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
        ) : source === 'modrinth' ? (
          <ModpackBrowser selectedVersionId={modpackVersion} onSelectVersion={setModpackVersion} />
        ) : (
          <ImportSource value={importSource} onChange={setImportSource} />
        )}

        {downloading && (
          <div className="flex flex-col gap-1">
            <p className="text-sm text-muted-foreground">
              {progress ? `Downloading files ${progress.completed}/${progress.total}` : 'Preparing…'}
            </p>
            {progress && progress.total > 0 && (
              <Progress value={(progress.completed / progress.total) * 100} />
            )}
          </div>
        )}
        {error && <p className="text-sm text-destructive">{error.message}</p>}
        <DialogFooter>
          <Button variant="ghost" disabled={busy} onClick={close}>
            Cancel
          </Button>
          <Button onClick={create} disabled={!canCreate || busy}>
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
