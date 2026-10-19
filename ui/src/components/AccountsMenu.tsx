import { Check, ChevronDown, Trash2, UserPlus, UserRound } from 'lucide-react'
import { useState } from 'react'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { OfflineAccountDialog } from '@/dialogs/OfflineAccountDialog'
import { useAccounts, useRemoveAccount, useSetDefaultAccount } from '@/hooks/use-accounts'

export function AccountsMenu() {
  const { data: accounts = [] } = useAccounts()
  const setDefault = useSetDefaultAccount()
  const removeAccount = useRemoveAccount()
  const [isAdding, setIsAdding] = useState(false)
  const current = accounts.find((account) => account.isDefault)

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button size="sm" variant="ghost">
            <UserRound /> {current?.username ?? 'No account'} <ChevronDown />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-56">
          {accounts.map((account) => (
            <DropdownMenuItem key={account.id} onSelect={() => setDefault.mutate(account.id)}>
              <Check className={account.isDefault ? '' : 'invisible'} />
              <span className="flex-1">{account.username}</span>
              <span className="text-xs text-muted-foreground">{account.kind.type}</span>
              <button
                type="button"
                aria-label={`Remove ${account.username}`}
                className="text-muted-foreground hover:text-destructive"
                onClick={(event) => {
                  event.stopPropagation()
                  removeAccount.mutate(account.id)
                }}
              >
                <Trash2 className="size-3.5" />
              </button>
            </DropdownMenuItem>
          ))}
          {accounts.length > 0 && <DropdownMenuSeparator />}
          <DropdownMenuItem onSelect={() => setIsAdding(true)}>
            <UserPlus /> Add offline account…
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      <OfflineAccountDialog open={isAdding} onOpenChange={setIsAdding} />
    </>
  )
}
