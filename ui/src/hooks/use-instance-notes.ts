import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

const key = (instanceId: string) => ['instance-notes', instanceId]

export function useInstanceNotes(instanceId: string) {
  return useQuery({ queryKey: key(instanceId), queryFn: () => unwrap(commands.getInstanceNotes(instanceId)) })
}

export function useSetInstanceNotes(instanceId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (notes: string) => unwrap(commands.setInstanceNotes(instanceId, notes)),
    onSuccess: (_, notes) => queryClient.setQueryData(key(instanceId), notes),
  })
}
