import { Save } from 'lucide-react'
import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { useInstanceNotes, useSetInstanceNotes } from '@/hooks/use-instance-notes'

export function NotesPage({ instanceId }: { instanceId: string }) {
  const { data: notes, error } = useInstanceNotes(instanceId)
  if (error) return <p className="text-sm text-destructive">{error.message}</p>
  if (notes === undefined) return <p className="text-sm text-muted-foreground">Loading notes…</p>
  return <NotesEditor instanceId={instanceId} saved={notes} />
}

function NotesEditor({ instanceId, saved }: { instanceId: string; saved: string }) {
  const [draft, setDraft] = useState(saved)
  const save = useSetInstanceNotes(instanceId)
  const isDirty = draft !== saved

  const submit = () => {
    if (isDirty) save.mutate(draft)
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex items-center gap-2">
        <span className="text-sm text-muted-foreground">
          {isDirty ? 'Unsaved changes' : 'Notes are stored with the instance.'}
        </span>
        <Button size="sm" className="ml-auto" disabled={!isDirty || save.isPending} onClick={submit}>
          <Save /> Save
        </Button>
      </div>
      {save.error && <p className="text-sm text-destructive">{save.error.message}</p>}
      <Textarea
        className="min-h-0 flex-1 resize-none font-mono text-sm"
        placeholder="Seeds, coordinates, server passwords, mod setup notes…"
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={(event) => {
          if ((event.metaKey || event.ctrlKey) && event.key === 's') {
            event.preventDefault()
            submit()
          }
        }}
      />
    </div>
  )
}
