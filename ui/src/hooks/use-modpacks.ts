import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, type PackOptions, type Provider } from '@/bindings'
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
    mutationFn: ({ selection, options }: { selection: ModpackSelection; options: PackOptions }) =>
      unwrap(commands.installModpack(selection.provider, selection.projectId, selection.versionId, options)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['instances'] }),
  })
}

export function useImportInstance() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ source, options }: { source: string; options: PackOptions }) =>
      unwrap(commands.importInstance(source, options)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['instances'] }),
  })
}

export function useOptionalMods(selection: ModpackSelection | null) {
  return useQuery({
    queryKey: ['optional-mods', selection?.projectId, selection?.versionId],
    queryFn: () => unwrap(commands.listOptionalMods(selection!.projectId, selection!.versionId)),
    enabled: selection !== null,
  })
}

export function useUpdateModpack(instanceId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (versionId: string) => unwrap(commands.updateModpack(instanceId, versionId)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['instances'] }),
  })
}
