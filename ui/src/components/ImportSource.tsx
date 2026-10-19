import { open } from '@tauri-apps/plugin-dialog'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

interface ImportSourceProps {
  value: string
  onChange: (value: string) => void
}

export function ImportSource({ value, onChange }: ImportSourceProps) {
  const { t } = useTranslation('newInstance')
  const browse = async () => {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: t('importSource.filter'), extensions: ['mrpack', 'zip'] }],
    })
    if (path) onChange(path)
  }

  return (
    <div className="flex flex-1 flex-col gap-2">
      <p className="text-sm text-muted-foreground">{t('importSource.description')}</p>
      <div className="flex gap-2">
        <Input
          placeholder={t('importSource.placeholder')}
          value={value}
          onChange={(event) => onChange(event.target.value)}
        />
        <Button variant="outline" onClick={() => void browse()}>
          {t('importSource.browse')}
        </Button>
      </div>
    </div>
  )
}
