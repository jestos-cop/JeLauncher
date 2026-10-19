import { Copy, Eraser, Upload, WrapText } from 'lucide-react'
import { useMutation } from '@tanstack/react-query'
import { useEffect, useMemo, useRef, useState } from 'react'
import { Trans, useTranslation } from 'react-i18next'
import type { LogLine } from '@/app/sessions'
import { commands, type LogLevel } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { unwrap } from '@/lib/command'
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
  const { t } = useTranslation(['launch', 'common'])
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

  const [isConfirmingUpload, setIsConfirmingUpload] = useState(false)
  const upload = useMutation({
    mutationFn: () => unwrap(commands.uploadLog(lines.map((entry) => entry.line).join('\n'))),
  })

  const copy = () => void navigator.clipboard.writeText(visible.map((entry) => entry.line).join('\n'))

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      <div className="flex items-center gap-2">
        <Input
          className="w-64"
          placeholder={t('log.searchPlaceholder')}
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
        <span className="text-xs text-muted-foreground">{t('log.lines', { count: visible.length })}</span>
        <div className="ml-auto flex gap-1">
          <Button size="sm" variant={wrap ? 'secondary' : 'ghost'} onClick={() => setWrap(!wrap)}>
            <WrapText /> {t('log.wrap')}
          </Button>
          <Button size="sm" variant="ghost" onClick={copy}>
            <Copy /> {t('common:action.copy')}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={lines.length === 0 || upload.isPending}
            onClick={() => setIsConfirmingUpload(true)}
          >
            <Upload /> {upload.isPending ? t('log.uploading') : t('log.upload')}
          </Button>
          <Button size="sm" variant="ghost" onClick={onClear}>
            <Eraser /> {t('log.clear')}
          </Button>
        </div>
      </div>
      {upload.data && (
        <p className="text-sm">
          <Trans
            t={t}
            i18nKey="log.uploaded"
            values={{ url: upload.data }}
            components={{
              link: (
                <button
                  type="button"
                  className="text-primary underline underline-offset-2"
                  onClick={() => void unwrap(commands.openUrl(upload.data))}
                />
              ),
            }}
          />{' '}
          <button
            type="button"
            title={t('log.copyLink')}
            className="text-muted-foreground hover:text-foreground"
            onClick={() => void navigator.clipboard.writeText(upload.data)}
          >
            <Copy className="inline size-3.5" />
          </button>
        </p>
      )}
      {upload.error && <p className="text-sm text-destructive">{upload.error.message}</p>}
      <div
        ref={scrollRef}
        onScroll={onScroll}
        className="min-h-0 flex-1 overflow-auto rounded-md border bg-black/40 p-2 font-mono text-xs"
      >
        {visible.map((entry, index) => (
          <div
            key={index}
            className={cn(
              LEVEL_CLASSES[entry.level],
              wrap ? 'break-all whitespace-pre-wrap' : 'whitespace-pre',
            )}
          >
            {entry.line}
          </div>
        ))}
      </div>
      {isConfirmingUpload && (
        <ConfirmDialog
          title={t('log.confirmUploadTitle')}
          description={t('log.confirmUploadDescription')}
          confirmLabel={t('log.upload')}
          onConfirm={() => {
            setIsConfirmingUpload(false)
            upload.mutate()
          }}
          onClose={() => setIsConfirmingUpload(false)}
        />
      )}
    </div>
  )
}
