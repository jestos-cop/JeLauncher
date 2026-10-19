import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

const screenshotsKey = (instanceId: string) => ['screenshots', instanceId]

export function useScreenshots(instanceId: string) {
  return useQuery({
    queryKey: screenshotsKey(instanceId),
    queryFn: () => unwrap(commands.listScreenshots(instanceId)),
  })
}

function useScreenshotMutation<T>(instanceId: string, mutationFn: (input: T) => Promise<unknown>) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: screenshotsKey(instanceId) }),
  })
}

export function useDeleteScreenshot(instanceId: string) {
  return useScreenshotMutation(instanceId, (fileName: string) =>
    unwrap(commands.deleteScreenshot(instanceId, fileName)),
  )
}

export function useRenameScreenshot(instanceId: string) {
  return useScreenshotMutation(instanceId, ({ fileName, newName }: { fileName: string; newName: string }) =>
    unwrap(commands.renameScreenshot(instanceId, fileName, newName)),
  )
}

export function openScreenshotsFolder(instanceId: string) {
  return unwrap(commands.openScreenshotsFolder(instanceId))
}
