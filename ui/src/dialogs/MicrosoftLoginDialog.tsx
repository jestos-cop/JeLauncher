import { Copy, ExternalLink } from 'lucide-react'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { commands } from '@/bindings'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { cancelMicrosoftLogin, useMicrosoftLogin } from '@/hooks/use-accounts'
import { unwrap } from '@/lib/command'

/** Mount only while open: mounting starts a sign-in, unmounting cancels it. */
export function MicrosoftLoginDialog({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation(['accounts', 'common'])
  const { begin, finish } = useMicrosoftLogin()
  const { mutate: beginLogin } = begin
  const { mutate: finishLogin } = finish

  useEffect(() => {
    beginLogin(undefined, { onSuccess: () => finishLogin(undefined, { onSuccess: onClose }) })
    return () => void cancelMicrosoftLogin()
  }, [beginLogin, finishLogin, onClose])

  const prompt = begin.data
  const error = begin.error ?? finish.error

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t('microsoft.title')}</DialogTitle>
          <DialogDescription>{t('microsoft.description')}</DialogDescription>
        </DialogHeader>
        {prompt ? (
          <div className="flex flex-col items-center gap-3">
            <button
              type="button"
              title={t('microsoft.copyCode')}
              className="flex items-center gap-2 rounded-md border px-4 py-2 font-mono text-2xl tracking-widest hover:bg-accent"
              onClick={() => void navigator.clipboard.writeText(prompt.userCode)}
            >
              {prompt.userCode} <Copy className="size-4 text-muted-foreground" />
            </button>
            <Button onClick={() => void unwrap(commands.openUrl(prompt.verificationUri))}>
              <ExternalLink />{' '}
              {t('microsoft.open', { url: prompt.verificationUri.replace(/^https?:\/\//, '') })}
            </Button>
            {finish.isPending && <p className="text-sm text-muted-foreground">{t('microsoft.waiting')}</p>}
          </div>
        ) : (
          !error && <p className="text-sm text-muted-foreground">{t('microsoft.requesting')}</p>
        )}
        {error && <p className="text-sm text-destructive">{error.message}</p>}
        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>
            {t('common:action.cancel')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
