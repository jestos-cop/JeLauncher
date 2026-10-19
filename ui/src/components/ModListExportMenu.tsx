import { save } from '@tauri-apps/plugin-dialog'
import { ChevronDown, FileText } from 'lucide-react'
import { commands, type InstanceSummary, type ModListFormat } from '@/bindings'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { unwrap } from '@/lib/command'

const FORMATS: { format: ModListFormat; label: string; extension: string }[] = [
  { format: 'plainText', label: 'Plain text', extension: 'txt' },
  { format: 'markdown', label: 'Markdown', extension: 'md' },
  { format: 'html', label: 'HTML', extension: 'html' },
  { format: 'json', label: 'JSON', extension: 'json' },
  { format: 'csv', label: 'CSV', extension: 'csv' },
]

export function ModListExportMenu({ instance }: { instance: InstanceSummary }) {
  const exportList = async (format: ModListFormat, extension: string) => {
    const output = await save({
      defaultPath: `${instance.name} mods.${extension}`,
      filters: [{ name: 'Mod list', extensions: [extension] }],
    })
    if (output) await unwrap(commands.exportModList(instance.id, format, output))
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button size="sm" variant="ghost">
          <FileText /> Export List <ChevronDown />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {FORMATS.map(({ format, label, extension }) => (
          <DropdownMenuItem key={format} onSelect={() => void exportList(format, extension)}>
            {label}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
