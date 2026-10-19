import { Coffee, Monitor } from 'lucide-react'
import { useState } from 'react'
import { useNavigation, type SettingsTab } from '@/app/navigation'
import type { GlobalSettings } from '@/bindings'
import { Field, JavaPicker, NumberInput } from '@/components/SettingsFields'
import { SidebarLayout, type SidebarTab } from '@/components/SidebarLayout'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { useSettings, useUpdateSettings } from '@/hooks/use-settings'

const TABS: SidebarTab<SettingsTab>[] = [
  { tab: 'java', label: 'Java', icon: Coffee },
  { tab: 'minecraft', label: 'Minecraft', icon: Monitor },
]

export function SettingsPage({ tab }: { tab: SettingsTab }) {
  const { navigate } = useNavigation()
  const { data: saved } = useSettings()

  return (
    <SidebarLayout
      title="Settings"
      tabs={TABS}
      current={tab}
      onTabChange={(next) => navigate({ page: 'settings', tab: next })}
      onBack={() => navigate({ page: 'main' })}
    >
      {saved && <SettingsForm key={JSON.stringify(saved)} saved={saved} tab={tab} />}
    </SidebarLayout>
  )
}

function SettingsForm({ saved, tab }: { saved: GlobalSettings; tab: SettingsTab }) {
  const [draft, setDraft] = useState(saved)
  const update = useUpdateSettings()
  const isDirty = JSON.stringify(draft) !== JSON.stringify(saved)

  const setJava = (patch: Partial<GlobalSettings['java']>) =>
    setDraft((current) => ({ ...current, java: { ...current.java, ...patch } }))
  const setWindow = (patch: Partial<GlobalSettings['window']>) =>
    setDraft((current) => ({ ...current, window: { ...current.window, ...patch } }))

  return (
    <div className="flex max-w-4xl flex-col gap-5">
      {tab === 'java' && (
        <>
          <Field label="Java installation">
            <JavaPicker value={draft.java.path} onChange={(path) => setJava({ path })} />
          </Field>
          <Field label="Minimum memory">
            <NumberInput
              value={draft.java.minMemoryMb}
              min={128}
              step={128}
              suffix="MiB"
              onChange={(minMemoryMb) => setJava({ minMemoryMb })}
            />
          </Field>
          <Field label="Maximum memory" hint="Memory the game may use. More is not always faster.">
            <NumberInput
              value={draft.java.maxMemoryMb}
              min={512}
              step={256}
              suffix="MiB"
              onChange={(maxMemoryMb) => setJava({ maxMemoryMb })}
            />
          </Field>
          <Field label="JVM arguments">
            <Input
              placeholder="-XX:+UseG1GC"
              value={draft.java.jvmArgs}
              onChange={(event) => setJava({ jvmArgs: event.target.value })}
            />
          </Field>
        </>
      )}
      {tab === 'minecraft' && (
        <>
          <Field label="Window size">
            <div className="flex items-center gap-2">
              <NumberInput value={draft.window.width} min={320} onChange={(width) => setWindow({ width })} />
              <span className="text-muted-foreground">×</span>
              <NumberInput
                value={draft.window.height}
                min={240}
                onChange={(height) => setWindow({ height })}
              />
            </div>
          </Field>
          <Field label="Start maximized">
            <Checkbox
              checked={draft.window.maximized}
              onCheckedChange={(checked) => setWindow({ maximized: checked === true })}
            />
          </Field>
        </>
      )}

      {draft.java.minMemoryMb > draft.java.maxMemoryMb && (
        <p className="text-sm text-destructive">Minimum memory must not exceed maximum memory.</p>
      )}
      {update.error && <p className="text-sm text-destructive">{update.error.message}</p>}
      <div className="flex gap-2">
        <Button
          disabled={!isDirty || update.isPending || draft.java.minMemoryMb > draft.java.maxMemoryMb}
          onClick={() => update.mutate(draft)}
        >
          Save
        </Button>
        <Button variant="ghost" disabled={!isDirty} onClick={() => setDraft(saved)}>
          Reset
        </Button>
      </div>
    </div>
  )
}
