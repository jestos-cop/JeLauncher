import { Check, LogIn, RefreshCw, Trash2, UserPlus, UserRound } from 'lucide-react'
import { useCallback, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { AccountSummary } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { MicrosoftLoginDialog } from '@/dialogs/MicrosoftLoginDialog'
import { OfflineAccountDialog } from '@/dialogs/OfflineAccountDialog'
import { useAccounts, useRefreshAccount, useRemoveAccount, useSetDefaultAccount } from '@/hooks/use-accounts'

export function AccountsSettings() {
  const { t } = useTranslation(['accounts', 'common'])
  const { data: accounts = [], error, isPending } = useAccounts()
  const setDefault = useSetDefaultAccount()
  const removeAccount = useRemoveAccount()
  const refreshAccount = useRefreshAccount()
  const [isAdding, setIsAdding] = useState(false)
  const [isSigningIn, setIsSigningIn] = useState(false)
  const [pendingRemoval, setPendingRemoval] = useState<AccountSummary | null>(null)
  const closeSignIn = useCallback(() => setIsSigningIn(false), [])
  const mutationError = setDefault.error ?? removeAccount.error ?? refreshAccount.error

  return (
    <div className="flex max-w-3xl flex-col gap-3">
      <div className="flex items-center gap-2">
        <span className="text-sm text-muted-foreground">{t('defaultHint')}</span>
        <div className="ml-auto flex gap-1">
          <Button size="sm" onClick={() => setIsSigningIn(true)}>
            <LogIn /> {t('addMicrosoft')}
          </Button>
          <Button size="sm" variant="outline" onClick={() => setIsAdding(true)}>
            <UserPlus /> {t('addOffline')}
          </Button>
        </div>
      </div>

      {(error ?? mutationError) && (
        <p className="text-sm text-destructive">{(error ?? mutationError)?.message}</p>
      )}

      <div className="rounded-md border">
        {accounts.map((account) => (
          <div key={account.id} className="flex items-center gap-3 border-b p-2 last:border-0">
            <UserRound className="size-8 rounded bg-muted p-1.5 text-muted-foreground" />
            <div className="min-w-0 flex-1">
              <p className="truncate font-medium">{account.username}</p>
              <p className="text-xs text-muted-foreground">{t(`kind.${account.kind.type}`)}</p>
            </div>
            {account.isDefault ? (
              <Badge variant="secondary">
                <Check /> {t('default')}
              </Badge>
            ) : (
              <Button size="sm" variant="ghost" onClick={() => setDefault.mutate(account.id)}>
                {t('useByDefault')}
              </Button>
            )}
            {account.kind.type === 'microsoft' && (
              <button
                type="button"
                title={t('signInAgain')}
                aria-label={t('refresh', { username: account.username })}
                className="text-muted-foreground hover:text-foreground disabled:opacity-50"
                disabled={refreshAccount.isPending}
                onClick={() => refreshAccount.mutate(account.id)}
              >
                <RefreshCw className={refreshAccount.isPending ? 'size-4 animate-spin' : 'size-4'} />
              </button>
            )}
            <button
              type="button"
              title={t('common:action.remove')}
              aria-label={t('remove', { username: account.username })}
              className="text-muted-foreground hover:text-destructive"
              onClick={() => setPendingRemoval(account)}
            >
              <Trash2 className="size-4" />
            </button>
          </div>
        ))}
        {isPending && <p className="p-3 text-sm text-muted-foreground">{t('loading')}</p>}
        {!isPending && accounts.length === 0 && (
          <p className="p-3 text-sm text-muted-foreground">{t('empty')}</p>
        )}
      </div>

      <OfflineAccountDialog open={isAdding} onOpenChange={setIsAdding} />
      {isSigningIn && <MicrosoftLoginDialog onClose={closeSignIn} />}
      {pendingRemoval && (
        <ConfirmDialog
          title={t('removeDialog.title', { username: pendingRemoval.username })}
          description={t('removeDialog.description')}
          confirmLabel={t('common:action.remove')}
          onConfirm={() =>
            removeAccount.mutate(pendingRemoval.id, { onSuccess: () => setPendingRemoval(null) })
          }
          onClose={() => setPendingRemoval(null)}
        />
      )}
    </div>
  )
}
