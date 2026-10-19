import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, type ResourceKind } from '@/bindings'
import { unwrap } from '@/lib/command'

export const resourcesKey = (instanceId: string, kind: ResourceKind) => ['resources', instanceId, kind]

export function useResources(instanceId: string, kind: ResourceKind) {
  return useQuery({
    queryKey: resourcesKey(instanceId, kind),
    queryFn: () => unwrap(commands.listResources(instanceId, kind)),
  })
}

function useResourceMutation<T>(
  instanceId: string,
  kind: ResourceKind,
  mutationFn: (input: T) => Promise<unknown>,
) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: resourcesKey(instanceId, kind) }),
  })
}

export function useSetResourceEnabled(instanceId: string, kind: ResourceKind) {
  return useResourceMutation(
    instanceId,
    kind,
    ({ fileName, enabled }: { fileName: string; enabled: boolean }) =>
      unwrap(commands.setResourceEnabled(instanceId, kind, fileName, enabled)),
  )
}

export function useRemoveResource(instanceId: string, kind: ResourceKind) {
  return useResourceMutation(instanceId, kind, (fileName: string) =>
    unwrap(commands.removeResource(instanceId, kind, fileName)),
  )
}
