import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

export const worldsKey = (instanceId: string) => ['worlds', instanceId]

export function useWorlds(instanceId: string) {
  return useQuery({ queryKey: worldsKey(instanceId), queryFn: () => unwrap(commands.listWorlds(instanceId)) })
}

export function useDeleteWorld(instanceId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (folder: string) => unwrap(commands.deleteWorld(instanceId, folder)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: worldsKey(instanceId) }),
  })
}

export function useBackupWorld(instanceId: string) {
  return useMutation({ mutationFn: (folder: string) => unwrap(commands.backupWorld(instanceId, folder)) })
}

export function openSavesFolder(instanceId: string) {
  return unwrap(commands.openSavesFolder(instanceId))
}
