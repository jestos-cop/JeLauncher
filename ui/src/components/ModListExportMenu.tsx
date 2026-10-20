import { save } from '@tauri-apps/plugin-dialog'
import { ChevronDown, FileText } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { commands, type InstanceSummary, type ModListFormat } from '@/bindings'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { unwrap } from '@/lib/command'

const FORMATS: { format: ModListFormat; label?: string; extension: string }[] = [
  { format: 'plainText', extension: 'txt' },
  { format: 'markdown', label: 'Markdown', extension: 'md' },
  { format: 'html', label: 'HTML', extension: 'html' },
  { format: 'json', label: 'JSON', extension: 'json' },
  { format: 'csv', label: 'CSV', extension: 'csv' },
]

export function ModListExportMenu({ instance }: { instance: InstanceSummary }) {
  const { t } = useTranslation('instanceDialogs')
  const exportList = async (format: ModListFormat, extension: string) => {
    const output = await save({
      defaultPath: `${instance.name} mods.${extension}`,
      filters: [{ name: t('modListExport.fileType'), extensions: [extension] }],
    })
    if (output) await unwrap(commands.exportModList(instance.id, format, output))
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button size="sm" variant="ghost">
          <FileText /> {t('modListExport.button')} <ChevronDown />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-max">
        {FORMATS.map(({ format, label, extension }) => (
          <DropdownMenuItem key={format} onSelect={() => void exportList(format, extension)}>
            {label ?? t('modListExport.plainText')}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
