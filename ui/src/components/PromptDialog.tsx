import { useState } from 'react'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'

interface PromptDialogProps {
  title: string
  initialValue: string
  placeholder?: string
  confirmLabel: string
  allowEmpty?: boolean
  error?: string | null
  onConfirm: (value: string) => void
  onClose: () => void
}

/** Mount only while open so the input starts from `initialValue` each time. */
export function PromptDialog({
  title,
  initialValue,
  placeholder,
  confirmLabel,
  allowEmpty = false,
  error,
  onConfirm,
  onClose,
}: PromptDialogProps) {
  const [value, setValue] = useState(initialValue)

  const submit = (event: React.FormEvent) => {
    event.preventDefault()
    onConfirm(value)
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-sm">
        <form onSubmit={submit} className="flex flex-col gap-4">
          <DialogHeader>
            <DialogTitle>{title}</DialogTitle>
          </DialogHeader>
          <Input
            autoFocus
            placeholder={placeholder}
            value={value}
            onChange={(event) => setValue(event.target.value)}
          />
          {error && <p className="text-sm text-destructive">{error}</p>}
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" disabled={!allowEmpty && !value.trim()}>
              {confirmLabel}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
