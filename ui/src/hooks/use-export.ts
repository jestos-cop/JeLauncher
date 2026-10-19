import { useMutation, useQuery } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

export type ExportFormat = 'zip' | 'mrpack' | 'curseforge'

export interface ExportRequest {
  format: ExportFormat
  output: string
  blockedPaths: string[]
  version: string
  /** Modrinth summary or CurseForge author. */
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
        {
          zip: () => commands.exportInstance(instanceId, output, blockedPaths),
          mrpack: () => commands.exportMrpack(instanceId, output, blockedPaths, version, summary || null),
          curseforge: () => commands.exportCurseforgePack(instanceId, output, blockedPaths, version, summary),
        }[format](),
      ),
  })
}
