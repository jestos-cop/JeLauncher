import { useMutation } from '@tanstack/react-query'
import { useEffect } from 'react'
import { commands, type InstanceSummary } from '@/bindings'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { unwrap } from '@/lib/command'

/** Mount only while open: mounting creates the shortcut and reports where it went. */
export function ShortcutDialog({ instance, onClose }: { instance: InstanceSummary; onClose: () => void }) {
  const create = useMutation({ mutationFn: () => unwrap(commands.createInstanceShortcut(instance.id)) })
  const { mutate } = create

  useEffect(() => mutate(), [mutate])

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Desktop Shortcut</DialogTitle>
          <DialogDescription>
            {create.isPending && `Creating a shortcut for ${instance.name}…`}
            {create.data && 'Double-click the shortcut to start this instance directly.'}
          </DialogDescription>
        </DialogHeader>
        {create.data && <p className="font-mono text-xs break-all">{create.data}</p>}
        {create.error && <p className="text-sm text-destructive">{create.error.message}</p>}
        <DialogFooter>
          <Button onClick={onClose}>Done</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
