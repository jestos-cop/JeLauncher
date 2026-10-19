import { useSession } from '@/app/sessions'
import { LogView } from '@/components/LogView'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'

interface LogDialogProps {
  instanceId: string
  instanceName: string
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function LogDialog({ instanceId, instanceName, open, onOpenChange }: LogDialogProps) {
  const { session, clearLog } = useSession(instanceId)
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex h-[85vh] flex-col sm:max-w-5xl">
        <DialogHeader>
          <DialogTitle>{instanceName} — Minecraft Log</DialogTitle>
        </DialogHeader>
        <LogView lines={session.log} onClear={clearLog} />
      </DialogContent>
    </Dialog>
  )
}
