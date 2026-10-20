import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, type IdentifyReport, type ResourceKind, type ResourceUpdate } from '@/bindings'
import { unwrap } from '@/lib/command'

export const resourcesKey = (instanceId: string, kind: ResourceKind) => ['resources', instanceId, kind]

export function useResources(instanceId: string, kind: ResourceKind) {
  return useQuery({
    queryKey: resourcesKey(instanceId, kind),
    queryFn: () => unwrap(commands.listResources(instanceId, kind)),
  })
}

function useResourceMutation<T, R>(
  instanceId: string,
  kind: ResourceKind,
  mutationFn: (input: T) => Promise<R>,
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

const updatesKey = (instanceId: string, kind: ResourceKind) => ['resource-updates', instanceId, kind]

/** Update check on demand: nothing is fetched until `enabled` becomes true. The check also
 * identifies sources, so the file list is refreshed afterwards. */
export function useResourceUpdates(instanceId: string, kind: ResourceKind, enabled: boolean) {
  const queryClient = useQueryClient()
  return useQuery({
    queryKey: updatesKey(instanceId, kind),
    queryFn: async () => {
      const updates = await unwrap(commands.checkResourceUpdates(instanceId, kind))
      void queryClient.invalidateQueries({ queryKey: resourcesKey(instanceId, kind) })
      return updates
    },
    enabled,
    staleTime: Infinity,
  })
}

export function useIdentifySources(instanceId: string, kind: ResourceKind) {
  return useResourceMutation<void, IdentifyReport>(instanceId, kind, () =>
    unwrap(commands.identifyResourceSources(instanceId, kind)),
  )
}

export function useUpdateResources(instanceId: string, kind: ResourceKind) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (updates: ResourceUpdate[]) => {
      for (const update of updates) {
        await unwrap(commands.updateResource(instanceId, kind, update.fileName, update.version.id))
      }
    },
    onSettled: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: resourcesKey(instanceId, kind) }),
        queryClient.invalidateQueries({ queryKey: updatesKey(instanceId, kind) }),
      ]),
  })
}
