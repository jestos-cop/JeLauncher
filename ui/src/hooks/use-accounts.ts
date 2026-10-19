import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

const ACCOUNTS_KEY = ['accounts']

export function useAccounts() {
  return useQuery({
    queryKey: ACCOUNTS_KEY,
    queryFn: () => unwrap(commands.listAccounts()),
  })
}

function useAccountMutation<T>(mutationFn: (input: T) => Promise<unknown>) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ACCOUNTS_KEY }),
  })
}

export function useAddOfflineAccount() {
  return useAccountMutation((username: string) => unwrap(commands.addOfflineAccount(username)))
}

export function useSetDefaultAccount() {
  return useAccountMutation((id: string) => unwrap(commands.setDefaultAccount(id)))
}

export function useRemoveAccount() {
  return useAccountMutation((id: string) => unwrap(commands.removeAccount(id)))
}

export function useMicrosoftLogin() {
  const queryClient = useQueryClient()
  const begin = useMutation({ mutationFn: () => unwrap(commands.beginMicrosoftLogin()) })
  const finish = useMutation({
    mutationFn: () => unwrap(commands.finishMicrosoftLogin()),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ACCOUNTS_KEY }),
  })
  return { begin, finish }
}

export function cancelMicrosoftLogin() {
  return commands.cancelMicrosoftLogin()
}
