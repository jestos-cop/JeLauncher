import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'

interface SaveBarProps {
  isDirty: boolean
  canSave: boolean
  onSave: () => void
  onReset: () => void
}

/** Save and Reset buttons pinned to the bottom of a scrolling settings page. */
export function SaveBar({ isDirty, canSave, onSave, onReset }: SaveBarProps) {
  const { t } = useTranslation(['settings', 'common'])
  return (
    <div className="sticky -bottom-4 -mx-4 -mb-4 mt-auto flex items-center gap-2 border-t bg-background px-4 py-3">
      <Button disabled={!isDirty || !canSave} onClick={onSave}>
        {t('common:action.save')}
      </Button>
      <Button variant="ghost" disabled={!isDirty} onClick={onReset}>
        {t('common:action.reset')}
      </Button>
      {isDirty && <span className="text-sm text-muted-foreground">{t('unsavedChanges')}</span>}
    </div>
  )
}
