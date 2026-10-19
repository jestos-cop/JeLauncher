import { useMutation, useQuery } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

export type ExportFormat = 'zip' | 'mrpack'

export interface ExportRequest {
  format: ExportFormat
  output: string
  blockedPaths: string[]
  version: string
  summary: string
}

export function useExportEntries(instanceId: string) {
  return useQuery({
    queryKey: ['export-entries', instanceId],
    queryFn: () => unwrap(commands.listExportEntries(instanceId)),
  })
}

export function useExportInstance(instanceId: string) {
  return useMutation({
    mutationFn: ({ format, output, blockedPaths, version, summary }: ExportRequest) =>
      unwrap(
        format === 'zip'
          ? commands.exportInstance(instanceId, output, blockedPaths)
          : commands.exportMrpack(instanceId, output, blockedPaths, version, summary || null),
      ),
  })
}
