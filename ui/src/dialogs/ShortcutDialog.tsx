import { useMutation } from '@tanstack/react-query'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation(['instanceDialogs', 'common'])
  const create = useMutation({ mutationFn: () => unwrap(commands.createInstanceShortcut(instance.id)) })
  const { mutate } = create

  useEffect(() => mutate(), [mutate])

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t('shortcut.title')}</DialogTitle>
          <DialogDescription>
            {create.isPending && t('shortcut.creating', { name: instance.name })}
            {create.data && t('shortcut.created')}
          </DialogDescription>
        </DialogHeader>
        {create.data && <p className="font-mono text-xs break-all">{create.data}</p>}
        {create.error && <p className="text-sm text-destructive">{create.error.message}</p>}
        <DialogFooter>
          <Button onClick={onClose}>{t('common:action.done')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
