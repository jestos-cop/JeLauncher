import { ExternalLink, FolderOpen } from 'lucide-react'
import { commands, type InstalledPack } from '@/bindings'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { openInstanceFolder } from '@/hooks/use-instance-actions'
import { unwrap } from '@/lib/command'

interface BlockedFilesDialogProps {
  installed: InstalledPack
  onClose: () => void
}

/** Lists pack files whose authors only allow downloads from the CurseForge website. */
export function BlockedFilesDialog({ installed, onClose }: BlockedFilesDialogProps) {
  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>Some files need a manual download</DialogTitle>
          <DialogDescription>
            The instance was created, but these authors only allow downloads from the CurseForge website.
            Download each file and put it into the folder shown next to it.
          </DialogDescription>
        </DialogHeader>
        <ul className="max-h-80 overflow-auto rounded-md border">
          {installed.blocked.map((file) => (
            <li key={file.fileName} className="flex items-center gap-3 border-b p-2 last:border-0">
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">{file.projectName}</p>
                <p className="truncate text-xs text-muted-foreground">
                  {file.fileName} → {file.installDir}
                </p>
              </div>
              <Button size="sm" variant="outline" onClick={() => void unwrap(commands.openUrl(file.url))}>
                <ExternalLink /> Download page
              </Button>
            </li>
          ))}
        </ul>
        <DialogFooter>
          <Button variant="outline" onClick={() => void openInstanceFolder(installed.instanceId, 'mod')}>
            <FolderOpen /> Open mods folder
          </Button>
          <Button onClick={onClose}>Done</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
