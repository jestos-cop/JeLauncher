import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, type InstanceSettings } from '@/bindings'
import { unwrap } from '@/lib/command'

const key = (instanceId: string) => ['instance-settings', instanceId]

export function useInstanceSettings(instanceId: string) {
  return useQuery({
    queryKey: key(instanceId),
    queryFn: () => unwrap(commands.getInstanceSettings(instanceId)),
  })
}

export function useSetInstanceSettings(instanceId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (settings: InstanceSettings) => unwrap(commands.setInstanceSettings(instanceId, settings)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: key(instanceId) }),
  })
}
