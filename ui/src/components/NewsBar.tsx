import { Newspaper } from 'lucide-react'
import { useState } from 'react'
import { NewsDialog } from '@/dialogs/NewsDialog'
import { useNews } from '@/hooks/use-news'

/** The latest Minecraft news headline; opens the full list. */
export function NewsBar() {
  const { data: news } = useNews()
  const [isOpen, setIsOpen] = useState(false)
  const latest = news?.[0]
  if (!news || !latest) return null

  return (
    <>
      <button
        type="button"
        className="flex w-full items-center gap-2 border-b px-3 py-1.5 text-left text-sm hover:bg-accent/50"
        onClick={() => setIsOpen(true)}
      >
        <Newspaper className="size-4 shrink-0 text-muted-foreground" />
        <span className="truncate">{latest.title}</span>
        <span className="ml-auto shrink-0 text-xs text-muted-foreground">More news</span>
      </button>
      {isOpen && <NewsDialog news={news} onClose={() => setIsOpen(false)} />}
    </>
  )
}
