import { Check, ChevronDown, LogIn, Settings, Trash2, UserPlus, UserRound } from 'lucide-react'
import { useCallback, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigation } from '@/app/navigation'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { MicrosoftLoginDialog } from '@/dialogs/MicrosoftLoginDialog'
import { OfflineAccountDialog } from '@/dialogs/OfflineAccountDialog'
import { useAccounts, useRemoveAccount, useSetDefaultAccount } from '@/hooks/use-accounts'

export function AccountsMenu() {
  const { t } = useTranslation('main')
  const { data: accounts = [] } = useAccounts()
  const setDefault = useSetDefaultAccount()
  const removeAccount = useRemoveAccount()
  const [isAdding, setIsAdding] = useState(false)
  const [isSigningIn, setIsSigningIn] = useState(false)
  const closeSignIn = useCallback(() => setIsSigningIn(false), [])
  const { navigate } = useNavigation()
  const current = accounts.find((account) => account.isDefault)

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button size="sm" variant="ghost">
            <UserRound /> {current?.username ?? t('accounts.none')} <ChevronDown />
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
                aria-label={t('accounts.remove', { name: account.username })}
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
          <DropdownMenuItem onSelect={() => setIsSigningIn(true)}>
            <LogIn /> {t('accounts.addMicrosoft')}
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={() => setIsAdding(true)}>
            <UserPlus /> {t('accounts.addOffline')}
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={() => navigate({ page: 'settings', tab: 'accounts' })}>
            <Settings /> {t('accounts.manage')}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      <OfflineAccountDialog open={isAdding} onOpenChange={setIsAdding} />
      {isSigningIn && <MicrosoftLoginDialog onClose={closeSignIn} />}
    </>
  )
}
