import { useState } from 'react'
import type { InstanceSummary, LoaderSpec } from '@/bindings'
import { LoaderPicker } from '@/components/LoaderPicker'
import { VersionList } from '@/components/VersionList'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useSetInstanceVersion } from '@/hooks/use-instance-actions'

interface ChangeVersionDialogProps {
  instance: InstanceSummary
  onClose: () => void
}

export function ChangeVersionDialog({ instance, onClose }: ChangeVersionDialogProps) {
  const [version, setVersion] = useState<string | null>(instance.minecraftVersion)
  const [loader, setLoader] = useState<LoaderSpec | null>(instance.loader)
  const setInstanceVersion = useSetInstanceVersion()
  const isValid = version !== null && (loader === null || loader.version !== '')

  const save = () => {
    if (!version) return
    setInstanceVersion.mutate({ id: instance.id, minecraftVersion: version, loader }, { onSuccess: onClose })
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="flex h-[80vh] flex-col sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>Change Version</DialogTitle>
          <DialogDescription>
            Game files for the new version are downloaded on the next launch. Mods may need updating.
          </DialogDescription>
        </DialogHeader>
        <VersionList selected={version} onSelect={setVersion} />
        <LoaderPicker minecraftVersion={version} value={loader} onChange={setLoader} />
        {setInstanceVersion.error && (
          <p className="text-sm text-destructive">{setInstanceVersion.error.message}</p>
        )}
        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button disabled={!isValid || setInstanceVersion.isPending} onClick={save}>
            Save
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
