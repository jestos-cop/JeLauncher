import { useQuery } from '@tanstack/react-query'
import { Code } from 'lucide-react'
import { commands } from '@/bindings'
import { Logo } from '@/components/Logo'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { unwrap } from '@/lib/command'

export function AboutDialog({ onClose }: { onClose: () => void }) {
  const { data: info } = useQuery({ queryKey: ['app-info'], queryFn: () => commands.appInfo() })

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader className="items-center text-center">
          <Logo className="size-16" />
          <DialogTitle>JeLauncher</DialogTitle>
          <DialogDescription>A Minecraft launcher built with Rust, Tauri and React.</DialogDescription>
        </DialogHeader>
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
          <dt className="text-muted-foreground">Version</dt>
          <dd>{info?.version}</dd>
          <dt className="text-muted-foreground">License</dt>
          <dd>{info?.license}</dd>
        </dl>
        <p className="text-xs text-muted-foreground">
          Minecraft is a trademark of Mojang Studios. JeLauncher is not affiliated with Mojang or Microsoft.
        </p>
        <Button variant="outline" onClick={() => void unwrap(commands.openRepository())}>
          <Code /> Source code
        </Button>
      </DialogContent>
    </Dialog>
  )
}
