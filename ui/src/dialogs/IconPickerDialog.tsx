import { open } from '@tauri-apps/plugin-dialog'
import { ImagePlus, X } from 'lucide-react'
import type { InstanceSummary } from '@/bindings'
import { InstanceIcon } from '@/components/InstanceIcon'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { useAddCustomIcon, useCustomIcons, useRemoveCustomIcon, useSetInstanceIcon } from '@/hooks/use-icons'
import { BUILTIN_ICONS } from '@/lib/builtin-icons'
import { cn } from '@/lib/utils'

interface IconPickerDialogProps {
  instance: InstanceSummary
  onClose: () => void
}

export function IconPickerDialog({ instance, onClose }: IconPickerDialogProps) {
  const { data: customIcons = [] } = useCustomIcons()
  const addIcon = useAddCustomIcon()
  const removeIcon = useRemoveCustomIcon()
  const setIcon = useSetInstanceIcon()
  const error = addIcon.error ?? removeIcon.error ?? setIcon.error

  const choose = (iconKey: string) => setIcon.mutate({ id: instance.id, iconKey }, { onSuccess: onClose })
  const importIcon = async () => {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }],
    })
    if (path) addIcon.mutate(path)
  }

  const tile = (iconKey: string) => (
    <button
      type="button"
      title={iconKey}
      className={cn(
        'flex size-14 items-center justify-center rounded-md hover:bg-accent',
        iconKey === instance.iconKey && 'bg-accent ring-1 ring-ring',
      )}
      onClick={() => choose(iconKey)}
    >
      <InstanceIcon iconKey={iconKey} className="size-9" />
    </button>
  )

  return (
    <Dialog open onOpenChange={(isOpen) => !isOpen && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Choose an Icon</DialogTitle>
        </DialogHeader>
        <div className="grid grid-cols-7 gap-1">
          {Object.keys(BUILTIN_ICONS).map((iconKey) => (
            <div key={iconKey}>{tile(iconKey)}</div>
          ))}
          {customIcons.map((icon) => (
            <div key={icon.key} className="group relative">
              {tile(icon.key)}
              <button
                type="button"
                aria-label="Remove icon"
                className="absolute -top-1 -right-1 hidden rounded-full bg-destructive p-0.5 text-white group-hover:block"
                onClick={() => removeIcon.mutate(icon.key)}
              >
                <X className="size-3" />
              </button>
            </div>
          ))}
        </div>
        {error && <p className="text-sm text-destructive">{error.message}</p>}
        <DialogFooter className="sm:justify-between">
          <Button variant="outline" onClick={() => void importIcon()}>
            <ImagePlus /> Add Icon…
          </Button>
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
