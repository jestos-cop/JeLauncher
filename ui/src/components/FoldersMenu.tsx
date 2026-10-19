import { ChevronDown, Folder } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { commands, type LauncherFolder } from '@/bindings'
import { Button } from '@/components/ui/button'
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
        <Button size="sm" variant="ghost">
          <Folder /> {t('folders.title')} <ChevronDown />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        {FOLDERS.map((folder) => (
          <DropdownMenuItem key={folder} onSelect={() => void unwrap(commands.openLauncherFolder(folder))}>
            {t(`folders.${folder}`)}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
