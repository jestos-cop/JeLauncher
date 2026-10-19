import { ExternalLink } from 'lucide-react'
import { commands, type NewsEntry } from '@/bindings'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { unwrap } from '@/lib/command'

export function NewsDialog({ news, onClose }: { news: NewsEntry[]; onClose: () => void }) {
  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="flex max-h-[85vh] flex-col sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Minecraft News</DialogTitle>
        </DialogHeader>
        <ul className="flex min-h-0 flex-col gap-4 overflow-auto pr-1">
          {news.map((entry) => (
            <li key={entry.id} className="flex gap-3">
              {entry.imageUrl && (
                <img
                  src={entry.imageUrl}
                  alt=""
                  loading="lazy"
                  className="h-20 w-36 shrink-0 rounded-md object-cover"
                />
              )}
              <div className="flex min-w-0 flex-col gap-1">
                <p className="font-medium">{entry.title}</p>
                <p className="text-xs text-muted-foreground">{new Date(entry.date).toLocaleDateString()}</p>
                <p className="line-clamp-3 text-sm text-muted-foreground">{entry.text}</p>
                <Button
                  size="sm"
                  variant="link"
                  className="self-start px-0"
                  onClick={() => void unwrap(commands.openUrl(entry.url))}
                >
                  <ExternalLink /> Read more
                </Button>
              </div>
            </li>
          ))}
        </ul>
      </DialogContent>
    </Dialog>
  )
}
