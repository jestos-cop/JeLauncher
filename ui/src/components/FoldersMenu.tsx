import { ChevronDown, Folder } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { commands, type LauncherFolder } from '@/bindings'
import { Button } from '@/components/ui/button'
import { ToolbarLabel } from '@/components/ToolbarLabel'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { unwrap } from '@/lib/command'

const FOLDERS: LauncherFolder[] = ['instances', 'java', 'root']

export function FoldersMenu() {
  const { t } = useTranslation('main')
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button size="sm" variant="ghost" title={t('folders.title')}>
          <Folder /> <ToolbarLabel>{t('folders.title')}</ToolbarLabel> <ChevronDown />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-max">
        {FOLDERS.map((folder) => (
          <DropdownMenuItem key={folder} onSelect={() => void unwrap(commands.openLauncherFolder(folder))}>
            {t(`folders.${folder}`)}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
