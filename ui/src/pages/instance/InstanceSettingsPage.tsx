import { useState, type ReactNode } from 'react'
import type { GlobalSettings, InstanceSettings } from '@/bindings'
import { Field, JavaPicker, NumberInput } from '@/components/SettingsFields'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useInstanceSettings, useSetInstanceSettings } from '@/hooks/use-instance-settings'
import { useSettings } from '@/hooks/use-settings'

export function InstanceSettingsPage({ instanceId }: { instanceId: string }) {
  const { data: saved } = useInstanceSettings(instanceId)
  const { data: global } = useSettings()
  if (!saved || !global) return null
  return <SettingsForm key={JSON.stringify(saved)} instanceId={instanceId} saved={saved} global={global} />
}

interface SettingsFormProps {
  instanceId: string
  saved: InstanceSettings
  global: GlobalSettings
}

function SettingsForm({ instanceId, saved, global }: SettingsFormProps) {
  const [draft, setDraft] = useState(saved)
  const save = useSetInstanceSettings(instanceId)
  const set = (patch: Partial<InstanceSettings>) => setDraft((current) => ({ ...current, ...patch }))
  const isDirty = JSON.stringify(draft) !== JSON.stringify(saved)

  const javaOverridden =
    draft.javaPath !== null ||
    draft.minMemoryMb !== null ||
    draft.maxMemoryMb !== null ||
    draft.jvmArgs !== null
  const windowOverridden =
    draft.windowWidth !== null || draft.windowHeight !== null || draft.windowMaximized !== null

  const toggleJava = (on: boolean) =>
    set(
      on
        ? {
            javaPath: global.java.path,
            minMemoryMb: global.java.minMemoryMb,
            maxMemoryMb: global.java.maxMemoryMb,
            jvmArgs: global.java.jvmArgs,
          }
        : { javaPath: null, minMemoryMb: null, maxMemoryMb: null, jvmArgs: null },
    )
  const toggleWindow = (on: boolean) =>
    set(
      on
        ? {
            windowWidth: global.window.width,
            windowHeight: global.window.height,
            windowMaximized: global.window.maximized,
          }
        : { windowWidth: null, windowHeight: null, windowMaximized: null },
    )

  const minMemory = draft.minMemoryMb ?? global.java.minMemoryMb
  const maxMemory = draft.maxMemoryMb ?? global.java.maxMemoryMb

  return (
    <div className="flex max-w-4xl flex-col gap-6">
      <OverrideGroup title="Java" overridden={javaOverridden} onToggle={toggleJava}>
        <Field label="Java installation">
          <JavaPicker value={draft.javaPath ?? global.java.path} onChange={(javaPath) => set({ javaPath })} />
        </Field>
        <Field label="Minimum memory">
          <NumberInput
            value={minMemory}
            min={128}
            step={128}
            suffix="MiB"
            onChange={(v) => set({ minMemoryMb: v })}
          />
        </Field>
        <Field label="Maximum memory">
          <NumberInput
            value={maxMemory}
            min={512}
            step={256}
            suffix="MiB"
            onChange={(v) => set({ maxMemoryMb: v })}
          />
        </Field>
        <Field label="JVM arguments">
          <Input
            value={draft.jvmArgs ?? global.java.jvmArgs}
            onChange={(event) => set({ jvmArgs: event.target.value })}
          />
        </Field>
      </OverrideGroup>

      <OverrideGroup title="Game window" overridden={windowOverridden} onToggle={toggleWindow}>
        <Field label="Window size">
          <div className="flex items-center gap-2">
            <NumberInput
              value={draft.windowWidth ?? global.window.width}
              min={320}
              onChange={(windowWidth) => set({ windowWidth })}
            />
            <span className="text-muted-foreground">×</span>
            <NumberInput
              value={draft.windowHeight ?? global.window.height}
              min={240}
              onChange={(windowHeight) => set({ windowHeight })}
            />
          </div>
        </Field>
        <Field label="Start maximized">
          <Checkbox
            checked={draft.windowMaximized ?? global.window.maximized}
            onCheckedChange={(checked) => set({ windowMaximized: checked === true })}
          />
        </Field>
      </OverrideGroup>

      {minMemory > maxMemory && (
        <p className="text-sm text-destructive">Minimum memory must not exceed maximum memory.</p>
      )}
      {save.error && <p className="text-sm text-destructive">{save.error.message}</p>}
      <div className="flex gap-2">
        <Button
          disabled={!isDirty || save.isPending || minMemory > maxMemory}
          onClick={() => save.mutate(draft)}
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

interface OverrideGroupProps {
  title: string
  overridden: boolean
  onToggle: (overridden: boolean) => void
  children: ReactNode
}

function OverrideGroup({ title, overridden, onToggle, children }: OverrideGroupProps) {
  return (
    <section className="flex flex-col gap-4 rounded-md border p-4">
      <div className="flex items-center gap-3">
        <h2 className="font-semibold">{title}</h2>
        <Label className="ml-auto gap-2 font-normal">
          <Checkbox checked={overridden} onCheckedChange={(checked) => onToggle(checked === true)} />
          Override global settings
        </Label>
      </div>
      <fieldset disabled={!overridden} className="flex flex-col gap-4 disabled:opacity-50">
        {children}
      </fieldset>
    </section>
  )
}
