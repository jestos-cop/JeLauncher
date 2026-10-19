import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { GlobalSettings } from '@/bindings'
import { Field, JavaPicker, LanguageSelect, NumberInput, ThemeSelect } from '@/components/SettingsFields'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { useAccounts, useAddOfflineAccount } from '@/hooks/use-accounts'
import { useMemoryHint, useUpdateSettings } from '@/hooks/use-settings'
import { i18n, resolveLanguage } from '@/i18n'

const STEPS = ['welcome', 'appearance', 'java', 'account'] as const

/** Walks a new user through the settings that matter before the first launch. */
export function SetupWizard({ saved }: { saved: GlobalSettings }) {
  const { t } = useTranslation(['setup', 'common', 'settings', 'accounts'])
  const [step, setStep] = useState(0)
  const [draft, setDraft] = useState(saved)
  const [username, setUsername] = useState('')
  const { data: accounts = [] } = useAccounts()
  const addAccount = useAddOfflineAccount()
  const update = useUpdateSettings()
  const memoryHint = useMemoryHint(draft.java.maxMemoryMb)
  const isLast = step === STEPS.length - 1

  const setJava = (patch: Partial<GlobalSettings['java']>) =>
    setDraft((current) => ({ ...current, java: { ...current.java, ...patch } }))
  const finish = (settings: GlobalSettings) =>
    update.mutate({ ...settings, launcher: { ...settings.launcher, setupCompleted: true } })
  const next = () => {
    if (!isLast) return setStep(step + 1)
    if (username.trim()) addAccount.mutate(username, { onSuccess: () => finish(draft) })
    else finish(draft)
  }

  return (
    <Dialog open>
      <DialogContent
        className="sm:max-w-2xl"
        showCloseButton={false}
        onInteractOutside={(event) => event.preventDefault()}
      >
        <DialogHeader>
          <DialogTitle>{step === 0 ? t('welcomeTitle') : t(`steps.${STEPS[step]}`)}</DialogTitle>
          <DialogDescription>{t('progress', { step: step + 1, total: STEPS.length })}</DialogDescription>
        </DialogHeader>

        <div className="flex min-h-48 flex-col gap-4">
          {step === 0 && <p className="text-sm text-muted-foreground">{t('intro')}</p>}
          {step === 1 && (
            <>
              <Field label={t('settings:launcher.language')}>
                <LanguageSelect
                  value={draft.appearance.language}
                  onChange={(language) => {
                    void i18n.changeLanguage(resolveLanguage(language))
                    setDraft((current) => ({ ...current, appearance: { ...current.appearance, language } }))
                  }}
                />
              </Field>
              <Field label={t('settings:launcher.theme')}>
                <ThemeSelect
                  value={draft.appearance.theme}
                  onChange={(theme) =>
                    setDraft((current) => ({ ...current, appearance: { ...current.appearance, theme } }))
                  }
                />
              </Field>
            </>
          )}
          {step === 2 && (
            <>
              <Field label={t('settings:java.installation')}>
                <JavaPicker value={draft.java.path} onChange={(path) => setJava({ path })} />
              </Field>
              <Field label={t('settings:java.autoDownload')} hint={t('autoDownloadHint')}>
                <Checkbox
                  checked={draft.java.autoDownload}
                  onCheckedChange={(checked) => setJava({ autoDownload: checked === true })}
                />
              </Field>
              <Field label={t('settings:java.maxMemory')} hint={memoryHint}>
                <NumberInput
                  value={draft.java.maxMemoryMb}
                  min={512}
                  step={256}
                  suffix={t('settings:units.mib')}
                  onChange={(maxMemoryMb) => setJava({ maxMemoryMb })}
                />
              </Field>
            </>
          )}
          {step === 3 &&
            (accounts.length > 0 ? (
              <p className="text-sm text-muted-foreground">
                {t('existingAccounts', { count: accounts.length })}
              </p>
            ) : (
              <>
                <p className="text-sm text-muted-foreground">{t('offlineIntro')}</p>
                <Input
                  autoFocus
                  placeholder={t('accounts:offline.username')}
                  value={username}
                  onChange={(event) => setUsername(event.target.value)}
                />
              </>
            ))}
          {(addAccount.error ?? update.error) && (
            <p className="text-sm text-destructive">{(addAccount.error ?? update.error)?.message}</p>
          )}
        </div>

        <DialogFooter className="sm:justify-between">
          <Button variant="ghost" disabled={update.isPending} onClick={() => finish(saved)}>
            {t('skip')}
          </Button>
          <div className="flex gap-2">
            {step > 0 && (
              <Button variant="outline" onClick={() => setStep(step - 1)}>
                {t('common:action.back')}
              </Button>
            )}
            <Button disabled={update.isPending || addAccount.isPending} onClick={next}>
              {isLast ? t('common:action.finish') : t('common:action.next')}
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
