import { ChevronDown, Folder } from 'lucide-react'
import { commands, type LauncherFolder } from '@/bindings'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { unwrap } from '@/lib/command'

const FOLDERS: { folder: LauncherFolder; label: string }[] = [
  { folder: 'instances', label: 'Instances' },
  { folder: 'java', label: 'Java runtimes' },
  { folder: 'root', label: 'Launcher data' },
]

export function FoldersMenu() {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button size="sm" variant="ghost">
          <Folder /> Folders <ChevronDown />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        {FOLDERS.map(({ folder, label }) => (
          <DropdownMenuItem key={folder} onSelect={() => void unwrap(commands.openLauncherFolder(folder))}>
            {label}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
