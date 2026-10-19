import { useState } from 'react'
import type { ResourcePackPolicy, Server } from '@/bindings'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

const POLICIES: { value: ResourcePackPolicy; label: string }[] = [
  { value: 'prompt', label: 'Prompt' },
  { value: 'enabled', label: 'Enabled' },
  { value: 'disabled', label: 'Disabled' },
]

interface ServerDialogProps {
  server: Server | null
  error?: string
  pending: boolean
  onSave: (server: Server) => void
  onClose: () => void
}

/** Mount only while open so the fields start from `server` each time. */
export function ServerDialog({ server, error, pending, onSave, onClose }: ServerDialogProps) {
  const [name, setName] = useState(server?.name ?? 'Minecraft Server')
  const [address, setAddress] = useState(server?.address ?? '')
  const [resourcePacks, setResourcePacks] = useState<ResourcePackPolicy>(server?.resourcePacks ?? 'prompt')

  const submit = (event: React.FormEvent) => {
    event.preventDefault()
    onSave({ name: name.trim(), address: address.trim(), icon: server?.icon ?? null, resourcePacks })
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <form onSubmit={submit} className="flex flex-col gap-4">
          <DialogHeader>
            <DialogTitle>{server ? 'Edit Server' : 'Add Server'}</DialogTitle>
          </DialogHeader>
          <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-2">
            <Label htmlFor="server-name">Name</Label>
            <Input id="server-name" value={name} onChange={(event) => setName(event.target.value)} />
            <Label htmlFor="server-address">Address</Label>
            <Input
              id="server-address"
              autoFocus
              placeholder="play.example.com:25565"
              value={address}
              onChange={(event) => setAddress(event.target.value)}
            />
            <Label>Resource packs</Label>
            <Select
              value={resourcePacks}
              onValueChange={(next) => setResourcePacks(next as ResourcePackPolicy)}
            >
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {POLICIES.map((policy) => (
                  <SelectItem key={policy.value} value={policy.value}>
                    {policy.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" disabled={!address.trim() || pending}>
              Save
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
