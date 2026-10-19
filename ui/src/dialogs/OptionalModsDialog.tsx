import { useState } from 'react'
import { commands, type OptionalMod } from '@/bindings'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import type { ModpackSelection } from '@/hooks/use-modpacks'
import { unwrap } from '@/lib/command'

interface OptionalModsDialogProps {
  selection: ModpackSelection
  mods: OptionalMod[]
  onConfirm: (chosen: string[]) => void
  onClose: () => void
}

/** Lets the user pick a pack's optional mods by hand or from a share code. */
export function OptionalModsDialog({ selection, mods, onConfirm, onClose }: OptionalModsDialogProps) {
  const [chosen, setChosen] = useState(
    () => new Set(mods.filter((mod) => mod.selected).map((mod) => mod.name)),
  )
  const [shareCode, setShareCode] = useState('')
  const [shareCodeError, setShareCodeError] = useState<string | null>(null)

  const toggle = (name: string, checked: boolean) =>
    setChosen((current) => {
      const next = new Set(current)
      if (checked) next.add(name)
      else next.delete(name)
      return next
    })

  const applyShareCode = async () => {
    setShareCodeError(null)
    try {
      const names = await unwrap(
        commands.resolveShareCode(selection.projectId, selection.versionId, shareCode),
      )
      setChosen(new Set(names))
    } catch (error) {
      setShareCodeError(error instanceof Error ? error.message : String(error))
    }
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Optional Mods</DialogTitle>
          <DialogDescription>
            Pick the extras to install. Mods they depend on are added automatically.
          </DialogDescription>
        </DialogHeader>
        <ul className="max-h-[50vh] overflow-auto rounded-md border">
          {mods.map((mod) => (
            <li key={mod.name} className="border-b last:border-0">
              <label className="flex cursor-pointer items-start gap-3 p-2 hover:bg-accent/50">
                <Checkbox
                  className="mt-0.5"
                  checked={chosen.has(mod.name)}
                  onCheckedChange={(checked) => toggle(mod.name, checked === true)}
                />
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium">
                    {mod.name}
                    {mod.recommended && (
                      <Badge variant="secondary" className="ml-2">
                        recommended
                      </Badge>
                    )}
                  </p>
                  {mod.description && <p className="text-xs text-muted-foreground">{mod.description}</p>}
                </div>
              </label>
            </li>
          ))}
        </ul>
        <div className="flex gap-2">
          <Input
            placeholder="Share code"
            value={shareCode}
            onChange={(event) => setShareCode(event.target.value)}
          />
          <Button variant="outline" disabled={!shareCode.trim()} onClick={() => void applyShareCode()}>
            Use Share Code
          </Button>
        </div>
        {shareCodeError && <p className="text-sm text-destructive">{shareCodeError}</p>}
        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={() => onConfirm([...chosen])}>Install</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
