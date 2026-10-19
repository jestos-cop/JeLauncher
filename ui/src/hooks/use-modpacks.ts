import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

export function useModpackVersions(projectId: string | null) {
  return useQuery({
    queryKey: ['modpack-versions', projectId],
    queryFn: () => unwrap(commands.listModpackVersions(projectId!)),
    enabled: projectId !== null,
  })
}

export function useInstallModpack() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ versionId, name }: { versionId: string; name: string | null }) =>
      unwrap(commands.installModpack(versionId, name)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['instances'] }),
  })
}
