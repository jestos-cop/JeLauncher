import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, type Provider } from '@/bindings'
import { unwrap } from '@/lib/command'

export interface ModpackSelection {
  provider: Provider
  projectId: string
  versionId: string
}

export function useModpackVersions(provider: Provider, projectId: string | null) {
  return useQuery({
    queryKey: ['modpack-versions', provider, projectId],
    queryFn: () => unwrap(commands.listModpackVersions(provider, projectId!)),
    enabled: projectId !== null,
  })
}

export function useInstallModpack() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ selection, name }: { selection: ModpackSelection; name: string | null }) =>
      unwrap(commands.installModpack(selection.provider, selection.projectId, selection.versionId, name)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['instances'] }),
  })
}

export function useImportInstance() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ source, name }: { source: string; name: string | null }) =>
      unwrap(commands.importInstance(source, name)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['instances'] }),
  })
}

export function useUpdateModpack(instanceId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (versionId: string) => unwrap(commands.updateModpack(instanceId, versionId)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['instances'] }),
  })
}
