import { Check, Trash2, UserPlus, UserRound } from 'lucide-react'
import { useState } from 'react'
import type { AccountSummary } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { OfflineAccountDialog } from '@/dialogs/OfflineAccountDialog'
import { useAccounts, useRemoveAccount, useSetDefaultAccount } from '@/hooks/use-accounts'

export function AccountsSettings() {
  const { data: accounts = [], error, isPending } = useAccounts()
  const setDefault = useSetDefaultAccount()
  const removeAccount = useRemoveAccount()
  const [isAdding, setIsAdding] = useState(false)
  const [pendingRemoval, setPendingRemoval] = useState<AccountSummary | null>(null)
  const mutationError = setDefault.error ?? removeAccount.error

  return (
    <div className="flex max-w-3xl flex-col gap-3">
      <div className="flex items-center gap-2">
        <span className="text-sm text-muted-foreground">
          The default account is used to launch every instance.
        </span>
        <div className="ml-auto flex gap-1">
          <Button size="sm" disabled title="Microsoft sign-in is not available yet">
            Add Microsoft Account
          </Button>
          <Button size="sm" variant="outline" onClick={() => setIsAdding(true)}>
            <UserPlus /> Add Offline Account
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
              <p className="text-xs text-muted-foreground capitalize">{account.kind.type}</p>
            </div>
            {account.isDefault ? (
              <Badge variant="secondary">
                <Check /> Default
              </Badge>
            ) : (
              <Button size="sm" variant="ghost" onClick={() => setDefault.mutate(account.id)}>
                Use by default
              </Button>
            )}
            <button
              type="button"
              title="Remove"
              aria-label={`Remove ${account.username}`}
              className="text-muted-foreground hover:text-destructive"
              onClick={() => setPendingRemoval(account)}
            >
              <Trash2 className="size-4" />
            </button>
          </div>
        ))}
        {isPending && <p className="p-3 text-sm text-muted-foreground">Loading accounts…</p>}
        {!isPending && accounts.length === 0 && (
          <p className="p-3 text-sm text-muted-foreground">Add an account to launch the game.</p>
        )}
      </div>

      <OfflineAccountDialog open={isAdding} onOpenChange={setIsAdding} />
      {pendingRemoval && (
        <ConfirmDialog
          title={`Remove ${pendingRemoval.username}?`}
          description="The account is removed from the launcher. Your worlds and instances are not affected."
          confirmLabel="Remove"
          onConfirm={() =>
            removeAccount.mutate(pendingRemoval.id, { onSuccess: () => setPendingRemoval(null) })
          }
          onClose={() => setPendingRemoval(null)}
        />
      )}
    </div>
  )
}
