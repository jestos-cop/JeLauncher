import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, type Server } from '@/bindings'
import { unwrap } from '@/lib/command'

export const serversKey = (instanceId: string) => ['servers', instanceId]

export function useServers(instanceId: string) {
  return useQuery({
    queryKey: serversKey(instanceId),
    queryFn: () => unwrap(commands.listServers(instanceId)),
  })
}

export function useServerStatus(address: string) {
  return useQuery({
    queryKey: ['server-status', address],
    queryFn: () => unwrap(commands.pingServer(address)),
    staleTime: 30_000,
    retry: false,
  })
}

function useServerMutation<T>(instanceId: string, mutationFn: (input: T) => Promise<unknown>) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: serversKey(instanceId) }),
  })
}

export function useSaveServer(instanceId: string) {
  return useServerMutation(instanceId, ({ index, server }: { index: number | null; server: Server }) =>
    unwrap(
      index === null
        ? commands.addServer(instanceId, server)
        : commands.updateServer(instanceId, index, server),
    ),
  )
}

export function useRemoveServer(instanceId: string) {
  return useServerMutation(instanceId, (index: number) => unwrap(commands.removeServer(instanceId, index)))
}

export function useMoveServerUp(instanceId: string) {
  return useServerMutation(instanceId, (index: number) => unwrap(commands.moveServerUp(instanceId, index)))
}
