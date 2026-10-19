import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { useAddOfflineAccount } from '@/hooks/use-accounts'

interface OfflineAccountDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function OfflineAccountDialog({ open, onOpenChange }: OfflineAccountDialogProps) {
  const { t } = useTranslation(['accounts', 'common'])
  const [username, setUsername] = useState('')
  const addAccount = useAddOfflineAccount()

  const close = () => {
    setUsername('')
    addAccount.reset()
    onOpenChange(false)
  }

  const submit = (event: React.FormEvent) => {
    event.preventDefault()
    addAccount.mutate(username, { onSuccess: close })
  }

  return (
    <Dialog open={open} onOpenChange={(next) => (next ? onOpenChange(true) : close())}>
      <DialogContent className="sm:max-w-sm">
        <form onSubmit={submit} className="flex flex-col gap-4">
          <DialogHeader>
            <DialogTitle>{t('addOffline')}</DialogTitle>
            <DialogDescription>{t('offline.description')}</DialogDescription>
          </DialogHeader>
          <Input
            autoFocus
            placeholder={t('offline.username')}
            value={username}
            onChange={(event) => setUsername(event.target.value)}
          />
          {addAccount.error && <p className="text-sm text-destructive">{addAccount.error.message}</p>}
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={close}>
              {t('common:action.cancel')}
            </Button>
            <Button type="submit" disabled={!username.trim() || addAccount.isPending}>
              {t('common:action.add')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
