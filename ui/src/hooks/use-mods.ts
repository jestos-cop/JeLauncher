import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

const modsKey = (instanceId: string) => ['mods', instanceId]

export function useMods(instanceId: string) {
  return useQuery({
    queryKey: modsKey(instanceId),
    queryFn: () => unwrap(commands.listMods(instanceId)),
  })
}

function useModMutation<T>(instanceId: string, mutationFn: (input: T) => Promise<unknown>) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: modsKey(instanceId) }),
  })
}

export function useSetModEnabled(instanceId: string) {
  return useModMutation(instanceId, ({ fileName, enabled }: { fileName: string; enabled: boolean }) =>
    unwrap(commands.setModEnabled(instanceId, fileName, enabled)),
  )
}

export function useRemoveMod(instanceId: string) {
  return useModMutation(instanceId, (fileName: string) => unwrap(commands.removeMod(instanceId, fileName)))
}
