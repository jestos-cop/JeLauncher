import { useEffect, useState } from 'react'
import { events, type DownloadProgress, type LoaderSpec } from '@/bindings'
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
import { useInstallModpack } from '@/hooks/use-modpacks'

type Source = 'vanilla' | 'modrinth'

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
  const createInstance = useCreateInstance()
  const installModpack = useInstallModpack()
  const progress = useModpackProgress(installModpack.isPending)
  const busy = createInstance.isPending || installModpack.isPending
  const error = createInstance.error ?? installModpack.error

  const close = () => {
    if (busy) return
    setName('')
    setGroup('')
    setVersion(null)
    setLoader(null)
    setModpackVersion(null)
    createInstance.reset()
    installModpack.reset()
    onOpenChange(false)
  }

  const canCreate =
    source === 'vanilla'
      ? version !== null && (loader === null || loader.version !== '')
      : modpackVersion !== null

  const create = () => {
    if (source === 'modrinth' && modpackVersion) {
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
          </ToggleGroup>
        </DialogHeader>

        <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-2">
          <Label htmlFor="instance-name">Name</Label>
          <Input
            id="instance-name"
            placeholder={source === 'vanilla' ? (version ?? 'Instance name') : 'Modpack name'}
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
        ) : (
          <ModpackBrowser selectedVersionId={modpackVersion} onSelectVersion={setModpackVersion} />
        )}

        {installModpack.isPending && (
          <div className="flex flex-col gap-1">
            <p className="text-sm text-muted-foreground">
              {progress
                ? `Downloading files ${progress.completed}/${progress.total}`
                : 'Downloading modpack…'}
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

function useModpackProgress(active: boolean) {
  const [progress, setProgress] = useState<DownloadProgress | null>(null)
  useEffect(() => {
    if (!active) return
    const unlisten = events.modpackProgressEvent.listen(({ payload }) => setProgress(payload.progress))
    return () => {
      void unlisten.then((stop) => stop())
      setProgress(null)
    }
  }, [active])
  return progress
}
