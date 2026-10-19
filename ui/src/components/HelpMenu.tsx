import { ChevronDown, CircleHelp, Code, FileText, Info } from 'lucide-react'
import { useState } from 'react'
import { commands } from '@/bindings'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { AboutDialog } from '@/dialogs/AboutDialog'
import { unwrap } from '@/lib/command'

export function HelpMenu() {
  const [isAboutOpen, setIsAboutOpen] = useState(false)
  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button size="sm" variant="ghost">
            <CircleHelp /> Help <ChevronDown />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          <DropdownMenuItem onSelect={() => void unwrap(commands.openRepository())}>
            <Code /> Source code and issues
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={() => void unwrap(commands.openLauncherLog())}>
            <FileText /> Open launcher log
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={() => setIsAboutOpen(true)}>
            <Info /> About JeLauncher
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      {isAboutOpen && <AboutDialog onClose={() => setIsAboutOpen(false)} />}
    </>
  )
}
