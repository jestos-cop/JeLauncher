import { useMutation, useQueryClient } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

function useInstanceMutation<T>(mutationFn: (input: T) => Promise<unknown>) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['instances'] }),
  })
}

export function useRenameInstance() {
  return useInstanceMutation(({ id, name }: { id: string; name: string }) =>
    unwrap(commands.renameInstance(id, name)),
  )
}

export function useSetInstanceGroup() {
  return useInstanceMutation(({ id, group }: { id: string; group: string | null }) =>
    unwrap(commands.setInstanceGroup(id, group)),
  )
}

export function useDeleteInstance() {
  return useInstanceMutation((id: string) => unwrap(commands.deleteInstance(id)))
}

export function openInstanceFolder(id: string) {
  return unwrap(commands.openInstanceFolder(id))
}
