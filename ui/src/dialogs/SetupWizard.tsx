import { useState } from 'react'
import type { GlobalSettings } from '@/bindings'
import { Field, JavaPicker, NumberInput, ThemeSelect } from '@/components/SettingsFields'
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

const STEPS = ['Welcome', 'Appearance', 'Java', 'Account'] as const

/** Walks a new user through the settings that matter before the first launch. */
export function SetupWizard({ saved }: { saved: GlobalSettings }) {
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
          <DialogTitle>{step === 0 ? 'Welcome to JeLauncher' : STEPS[step]}</DialogTitle>
          <DialogDescription>
            Step {step + 1} of {STEPS.length}
          </DialogDescription>
        </DialogHeader>

        <div className="flex min-h-48 flex-col gap-4">
          {step === 0 && (
            <p className="text-sm text-muted-foreground">
              A few quick questions to get you playing. Everything here can be changed later in Settings.
            </p>
          )}
          {step === 1 && (
            <Field label="Theme">
              <ThemeSelect
                value={draft.appearance.theme}
                onChange={(theme) => setDraft((current) => ({ ...current, appearance: { theme } }))}
              />
            </Field>
          )}
          {step === 2 && (
            <>
              <Field label="Java installation">
                <JavaPicker value={draft.java.path} onChange={(path) => setJava({ path })} />
              </Field>
              <Field
                label="Download Java automatically"
                hint="Recommended: the right Java is fetched per game version."
              >
                <Checkbox
                  checked={draft.java.autoDownload}
                  onCheckedChange={(checked) => setJava({ autoDownload: checked === true })}
                />
              </Field>
              <Field label="Maximum memory" hint={memoryHint}>
                <NumberInput
                  value={draft.java.maxMemoryMb}
                  min={512}
                  step={256}
                  suffix="MiB"
                  onChange={(maxMemoryMb) => setJava({ maxMemoryMb })}
                />
              </Field>
            </>
          )}
          {step === 3 &&
            (accounts.length > 0 ? (
              <p className="text-sm text-muted-foreground">
                You already have {accounts.length === 1 ? 'an account' : `${accounts.length} accounts`}.
              </p>
            ) : (
              <>
                <p className="text-sm text-muted-foreground">
                  Add an offline account to play singleplayer and offline-mode servers. Microsoft sign-in
                  comes later.
                </p>
                <Input
                  autoFocus
                  placeholder="Username"
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
            Skip setup
          </Button>
          <div className="flex gap-2">
            {step > 0 && (
              <Button variant="outline" onClick={() => setStep(step - 1)}>
                Back
              </Button>
            )}
            <Button disabled={update.isPending || addAccount.isPending} onClick={next}>
              {isLast ? 'Finish' : 'Next'}
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
