import { Copy, Eraser, WrapText } from 'lucide-react'
import { useEffect, useMemo, useRef, useState } from 'react'
import type { LogLine } from '@/app/sessions'
import type { LogLevel } from '@/bindings'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'

const LEVEL_CLASSES: Record<LogLevel, string> = {
  launcher: 'text-violet-400',
  debug: 'text-muted-foreground',
  info: 'text-foreground',
  warning: 'text-amber-400',
  error: 'text-red-400',
  fatal: 'font-semibold text-red-500',
  unknown: 'text-foreground',
}

interface LogViewProps {
  lines: LogLine[]
  onClear: () => void
}

export function LogView({ lines, onClear }: LogViewProps) {
  const [search, setSearch] = useState('')
  const [wrap, setWrap] = useState(true)
  const scrollRef = useRef<HTMLDivElement>(null)
  const stickToBottom = useRef(true)

  const visible = useMemo(() => {
    const query = search.trim().toLowerCase()
    return query ? lines.filter((entry) => entry.line.toLowerCase().includes(query)) : lines
  }, [lines, search])

  useEffect(() => {
    const element = scrollRef.current
    if (element && stickToBottom.current) element.scrollTop = element.scrollHeight
  }, [visible])

  const onScroll = () => {
    const element = scrollRef.current
    if (element) {
      stickToBottom.current = element.scrollHeight - element.scrollTop - element.clientHeight < 24
    }
  }

  const copy = () => void navigator.clipboard.writeText(visible.map((entry) => entry.line).join('\n'))

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      <div className="flex items-center gap-2">
        <Input
          className="w-64"
          placeholder="Search log"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
        <span className="text-xs text-muted-foreground">{visible.length} lines</span>
        <div className="ml-auto flex gap-1">
          <Button size="sm" variant={wrap ? 'secondary' : 'ghost'} onClick={() => setWrap(!wrap)}>
            <WrapText /> Wrap
          </Button>
          <Button size="sm" variant="ghost" onClick={copy}>
            <Copy /> Copy
          </Button>
          <Button size="sm" variant="ghost" onClick={onClear}>
            <Eraser /> Clear
          </Button>
        </div>
      </div>
      <div
        ref={scrollRef}
        onScroll={onScroll}
        className="min-h-0 flex-1 overflow-auto rounded-md border bg-black/40 p-2 font-mono text-xs"
      >
        {visible.map((entry, index) => (
          <div
            key={index}
            className={cn(LEVEL_CLASSES[entry.level], wrap ? 'break-all whitespace-pre-wrap' : 'whitespace-pre')}
          >
            {entry.line}
          </div>
        ))}
      </div>
    </div>
  )
}
