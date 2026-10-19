import { useState } from 'react'
import type { CopyOptions, InstanceSummary } from '@/bindings'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useCopyInstance } from '@/hooks/use-instance-actions'

type CopyFlag = Exclude<keyof CopyOptions, 'name' | 'group'>

const FLAGS: { flag: CopyFlag; label: string }[] = [
  { flag: 'copySaves', label: 'Worlds' },
  { flag: 'copyMods', label: 'Mods and configs' },
  { flag: 'copyResourcePacks', label: 'Resource packs' },
  { flag: 'copyShaderPacks', label: 'Shader packs' },
  { flag: 'copyServers', label: 'Servers' },
  { flag: 'copyScreenshots', label: 'Screenshots' },
  { flag: 'copyGameOptions', label: 'Game options' },
  { flag: 'keepPlayTime', label: 'Keep play time' },
  { flag: 'useHardLinks', label: 'Hard-link files instead of copying (worlds are always copied)' },
]

interface CopyInstanceDialogProps {
  instance: InstanceSummary
  onClose: () => void
}

export function CopyInstanceDialog({ instance, onClose }: CopyInstanceDialogProps) {
  const copy = useCopyInstance()
  const [options, setOptions] = useState<CopyOptions>({
    name: `Copy of ${instance.name}`,
    group: instance.group,
    copySaves: true,
    copyGameOptions: true,
    copyResourcePacks: true,
    copyShaderPacks: true,
    copyServers: true,
    copyMods: true,
    copyScreenshots: true,
    keepPlayTime: false,
    useHardLinks: false,
  })
  const update = (patch: Partial<CopyOptions>) => setOptions((current) => ({ ...current, ...patch }))

  const submit = (event: React.FormEvent) => {
    event.preventDefault()
    copy.mutate(
      { id: instance.id, options: { ...options, name: options.name.trim(), group: options.group || null } },
      { onSuccess: onClose },
    )
  }

  return (
    <Dialog open onOpenChange={(open) => !open && !copy.isPending && onClose()}>
      <DialogContent className="sm:max-w-md">
        <form onSubmit={submit} className="flex flex-col gap-4">
          <DialogHeader>
            <DialogTitle>Copy {instance.name}</DialogTitle>
          </DialogHeader>
          <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-2">
            <Label htmlFor="copy-name">Name</Label>
            <Input
              id="copy-name"
              autoFocus
              value={options.name}
              onChange={(event) => update({ name: event.target.value })}
            />
            <Label htmlFor="copy-group">Group</Label>
            <Input
              id="copy-group"
              placeholder="No group"
              value={options.group ?? ''}
              onChange={(event) => update({ group: event.target.value })}
            />
          </div>
          <div className="flex flex-col gap-2">
            {FLAGS.map(({ flag, label }) => (
              <label key={flag} className="flex cursor-pointer items-center gap-2 text-sm">
                <Checkbox
                  checked={options[flag]}
                  onCheckedChange={(checked) => update({ [flag]: checked === true })}
                />
                {label}
              </label>
            ))}
          </div>
          {copy.error && <p className="text-sm text-destructive">{copy.error.message}</p>}
          <DialogFooter>
            <Button type="button" variant="ghost" disabled={copy.isPending} onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" disabled={!options.name.trim() || copy.isPending}>
              {copy.isPending ? 'Copying…' : 'Copy'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
