import { useState } from 'react'
import { VersionList } from '@/components/VersionList'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useCreateInstance } from '@/hooks/use-create-instance'

interface NewInstanceDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function NewInstanceDialog({ open, onOpenChange }: NewInstanceDialogProps) {
  const [name, setName] = useState('')
  const [group, setGroup] = useState('')
  const [version, setVersion] = useState<string | null>(null)
  const createInstance = useCreateInstance()

  const close = () => {
    setName('')
    setGroup('')
    setVersion(null)
    createInstance.reset()
    onOpenChange(false)
  }

  const create = () => {
    if (!version) return
    createInstance.mutate(
      { name: name.trim() || version, minecraftVersion: version, group: group || null, iconKey: null },
      { onSuccess: close },
    )
  }

  return (
    <Dialog open={open} onOpenChange={(next) => (next ? onOpenChange(true) : close())}>
      <DialogContent className="flex h-[80vh] flex-col sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>New Instance</DialogTitle>
        </DialogHeader>

        <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-2">
          <Label htmlFor="instance-name">Name</Label>
          <Input
            id="instance-name"
            placeholder={version ?? 'Instance name'}
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
          <Label htmlFor="instance-group">Group</Label>
          <Input
            id="instance-group"
            placeholder="No group"
            value={group}
            onChange={(event) => setGroup(event.target.value)}
          />
        </div>

        <VersionList selected={version} onSelect={setVersion} />

        {createInstance.error && (
          <p className="text-sm text-destructive">{createInstance.error.message}</p>
        )}
        <DialogFooter>
          <Button variant="ghost" onClick={close}>
            Cancel
          </Button>
          <Button onClick={create} disabled={!version || createInstance.isPending}>
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
