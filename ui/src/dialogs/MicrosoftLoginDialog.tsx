import { Copy, ExternalLink } from 'lucide-react'
import { useEffect } from 'react'
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
          <DialogTitle>Sign in with Microsoft</DialogTitle>
          <DialogDescription>
            Open the Microsoft page, enter the code and approve the sign-in. This window updates by itself.
          </DialogDescription>
        </DialogHeader>
        {prompt ? (
          <div className="flex flex-col items-center gap-3">
            <button
              type="button"
              title="Copy code"
              className="flex items-center gap-2 rounded-md border px-4 py-2 font-mono text-2xl tracking-widest hover:bg-accent"
              onClick={() => void navigator.clipboard.writeText(prompt.userCode)}
            >
              {prompt.userCode} <Copy className="size-4 text-muted-foreground" />
            </button>
            <Button onClick={() => void unwrap(commands.openUrl(prompt.verificationUri))}>
              <ExternalLink /> Open {prompt.verificationUri.replace(/^https?:\/\//, '')}
            </Button>
            {finish.isPending && <p className="text-sm text-muted-foreground">Waiting for approval…</p>}
          </div>
        ) : (
          !error && <p className="text-sm text-muted-foreground">Requesting a sign-in code…</p>
        )}
        {error && <p className="text-sm text-destructive">{error.message}</p>}
        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
