import { Plus, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { CommandSettings } from '@/bindings'
import { Field } from '@/components/SettingsFields'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

interface CommandFieldsProps {
  value: CommandSettings
  onChange: (value: CommandSettings) => void
}

export function CommandFields({ value, onChange }: CommandFieldsProps) {
  const { t } = useTranslation('settings')
  const set = (patch: Partial<CommandSettings>) => onChange({ ...value, ...patch })
  const entries = Object.entries(value.environment).map(([key, entry]) => [key, entry ?? ''] as const)

  const setEntry = (index: number, key: string, entryValue: string) =>
    set({
      environment: Object.fromEntries(entries.map((entry, i) => (i === index ? [key, entryValue] : entry))),
    })
  const removeEntry = (index: number) =>
    set({ environment: Object.fromEntries(entries.filter((_, i) => i !== index)) })
  const addEntry = () => set({ environment: { ...value.environment, [uniqueKey(value.environment)]: '' } })

  return (
    <>
      <Field label={t('commands.preLaunch')} hint={t('commands.preLaunchHint')}>
        <Input value={value.preLaunch} onChange={(event) => set({ preLaunch: event.target.value })} />
      </Field>
      <Field label={t('commands.wrapper')} hint={t('commands.wrapperHint')}>
        <Input value={value.wrapper} onChange={(event) => set({ wrapper: event.target.value })} />
      </Field>
      <Field label={t('commands.postExit')} hint={t('commands.postExitHint')}>
        <Input value={value.postExit} onChange={(event) => set({ postExit: event.target.value })} />
      </Field>
      <Field label={t('commands.environment')}>
        <div className="flex flex-col gap-2">
          {entries.map(([key, entryValue], index) => (
            <div key={index} className="flex gap-2">
              <Input
                className="w-56 font-mono"
                placeholder={t('commands.namePlaceholder')}
                value={key}
                onChange={(event) => setEntry(index, event.target.value, entryValue)}
              />
              <Input
                className="font-mono"
                placeholder={t('commands.valuePlaceholder')}
                value={entryValue}
                onChange={(event) => setEntry(index, key, event.target.value)}
              />
              <Button
                size="icon"
                variant="ghost"
                aria-label={t('commands.removeVariable', { name: key })}
                onClick={() => removeEntry(index)}
              >
                <X />
              </Button>
            </div>
          ))}
          <Button size="sm" variant="ghost" className="self-start" onClick={addEntry}>
            <Plus /> {t('commands.addVariable')}
          </Button>
        </div>
      </Field>
    </>
  )
}

function uniqueKey(environment: CommandSettings['environment']) {
  let index = 1
  while (`VARIABLE_${index}` in environment) index += 1
  return `VARIABLE_${index}`
}
