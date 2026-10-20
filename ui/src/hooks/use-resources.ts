import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  commands,
  type IdentifyReport,
  type ResourceKind,
  type ResourceSource,
  type ResourceUpdate,
} from '@/bindings'
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
    async ({ fileNames, enabled }: { fileNames: string[]; enabled: boolean }) => {
      for (const fileName of fileNames) {
        await unwrap(commands.setResourceEnabled(instanceId, kind, fileName, enabled))
      }
    },
  )
}

/** Files that should be switched along with `fileNames` because of their dependencies. */
export function affectedByToggle(
  instanceId: string,
  kind: ResourceKind,
  fileNames: string[],
  enable: boolean,
) {
  return unwrap(commands.resourcesAffectedByToggle(instanceId, kind, fileNames, enable))
}

/** Disables `disable`, then deletes `remove`. */
export function useRemoveResources(instanceId: string, kind: ResourceKind) {
  return useResourceMutation(
    instanceId,
    kind,
    async ({ remove, disable }: { remove: string[]; disable: string[] }) => {
      for (const fileName of disable) {
        await unwrap(commands.setResourceEnabled(instanceId, kind, fileName, false))
      }
      for (const fileName of remove) {
        await unwrap(commands.removeResource(instanceId, kind, fileName))
      }
    },
  )
}

/** Files that need `fileNames` and dependencies only they need. */
export function useRemovalPlan(instanceId: string, kind: ResourceKind, fileNames: string[]) {
  return useQuery({
    queryKey: ['removal-plan', instanceId, kind, [...fileNames].sort()],
    queryFn: () => unwrap(commands.resourcesRemovalPlan(instanceId, kind, fileNames)),
    placeholderData: keepPreviousData,
    gcTime: 0,
  })
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
        await unwrap(
          commands.updateResource(
            instanceId,
            kind,
            update.fileName,
            update.provider,
            update.version.projectId,
            update.version.id,
          ),
        )
      }
    },
    onSettled: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: resourcesKey(instanceId, kind) }),
        queryClient.invalidateQueries({ queryKey: updatesKey(instanceId, kind) }),
      ]),
  })
}

export function useSetUpdateLock(instanceId: string, kind: ResourceKind) {
  return useResourceMutation(
    instanceId,
    kind,
    ({ fileName, locked }: { fileName: string; locked: boolean }) =>
      unwrap(commands.setResourceUpdateLock(instanceId, kind, fileName, locked)),
  )
}

export function useForgetSource(instanceId: string, kind: ResourceKind) {
  return useResourceMutation(instanceId, kind, (fileName: string) =>
    unwrap(commands.forgetResourceSource(instanceId, kind, fileName)),
  )
}

/** Replaces an installed file with another version of the project it came from. */
export function useChangeVersion(
  instanceId: string,
  kind: ResourceKind,
  fileName: string,
  source: ResourceSource,
) {
  return useResourceMutation(instanceId, kind, (versionId: string) =>
    unwrap(commands.updateResource(instanceId, kind, fileName, source.provider, source.projectId, versionId)),
  )
}
