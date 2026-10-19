import { Coffee, Monitor, Rocket, Terminal, UsersRound, Wrench } from 'lucide-react'
import { useState } from 'react'
import { useNavigation, type SettingsTab } from '@/app/navigation'
import type { GlobalSettings } from '@/bindings'
import { CommandFields } from '@/components/CommandFields'
import { Field, JavaPicker, NumberInput, PathInput, ThemeSelect } from '@/components/SettingsFields'
import { SidebarLayout, type SidebarTab } from '@/components/SidebarLayout'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { useMemoryHint, useSettings, useUpdateSettings } from '@/hooks/use-settings'
import { AccountsSettings } from '@/pages/settings/AccountsSettings'

const TABS: SidebarTab<SettingsTab>[] = [
  { tab: 'launcher', label: 'Launcher', icon: Rocket },
  { tab: 'accounts', label: 'Accounts', icon: UsersRound },
  { tab: 'java', label: 'Java', icon: Coffee },
  { tab: 'minecraft', label: 'Minecraft', icon: Monitor },
  { tab: 'commands', label: 'Commands', icon: Terminal },
  { tab: 'tools', label: 'Tools', icon: Wrench },
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
      {tab === 'accounts' ? (
        <AccountsSettings />
      ) : (
        saved && <SettingsForm key={JSON.stringify(saved)} saved={saved} tab={tab} />
      )}
    </SidebarLayout>
  )
}

function SettingsForm({ saved, tab }: { saved: GlobalSettings; tab: SettingsTab }) {
  const [draft, setDraft] = useState(saved)
  const update = useUpdateSettings()
  const memoryHint = useMemoryHint(draft.java.maxMemoryMb)
  const isDirty = JSON.stringify(draft) !== JSON.stringify(saved)

  const setJava = (patch: Partial<GlobalSettings['java']>) =>
    setDraft((current) => ({ ...current, java: { ...current.java, ...patch } }))
  const setWindow = (patch: Partial<GlobalSettings['window']>) =>
    setDraft((current) => ({ ...current, window: { ...current.window, ...patch } }))
  const setLauncher = (patch: Partial<GlobalSettings['launcher']>) =>
    setDraft((current) => ({ ...current, launcher: { ...current.launcher, ...patch } }))
  const setTools = (patch: Partial<GlobalSettings['tools']>) =>
    setDraft((current) => ({ ...current, tools: { ...current.tools, ...patch } }))
  const setNetwork = (patch: Partial<GlobalSettings['network']>) =>
    setDraft((current) => ({ ...current, network: { ...current.network, ...patch } }))

  return (
    <div className="flex max-w-4xl flex-col gap-5">
      {tab === 'launcher' && (
        <>
          <h3 className="text-sm font-semibold">Appearance</h3>
          <Field label="Theme">
            <ThemeSelect
              value={draft.appearance.theme}
              onChange={(theme) => setDraft((current) => ({ ...current, appearance: { theme } }))}
            />
          </Field>
          <h3 className="text-sm font-semibold">Console</h3>
          <Field label="Open log on launch">
            <Checkbox
              checked={draft.launcher.showConsole}
              onCheckedChange={(checked) => setLauncher({ showConsole: checked === true })}
            />
          </Field>
          <Field label="Open log on crash">
            <Checkbox
              checked={draft.launcher.showConsoleOnError}
              onCheckedChange={(checked) => setLauncher({ showConsoleOnError: checked === true })}
            />
          </Field>
          <Field label="Close log on clean exit">
            <Checkbox
              checked={draft.launcher.autoCloseConsole}
              onCheckedChange={(checked) => setLauncher({ autoCloseConsole: checked === true })}
            />
          </Field>
          <h3 className="text-sm font-semibold">Game time</h3>
          <Field label="Record play time">
            <Checkbox
              checked={draft.launcher.recordGameTime}
              onCheckedChange={(checked) => setLauncher({ recordGameTime: checked === true })}
            />
          </Field>
          <h3 className="text-sm font-semibold">Network</h3>
          <Field label="Parallel downloads">
            <NumberInput
              value={draft.network.concurrentDownloads}
              min={1}
              max={64}
              onChange={(concurrentDownloads) => setNetwork({ concurrentDownloads })}
            />
          </Field>
          <Field
            label="Request timeout"
            hint="Timeout, retries and proxy apply after restarting the launcher."
          >
            <NumberInput
              value={draft.network.requestTimeoutSecs}
              min={5}
              suffix="s"
              onChange={(requestTimeoutSecs) => setNetwork({ requestTimeoutSecs })}
            />
          </Field>
          <Field
            label="Proxy"
            hint="For example http://proxy.local:8080. Leave empty to use the system proxy."
          >
            <Input
              placeholder="System proxy"
              value={draft.network.proxy ?? ''}
              onChange={(event) => setNetwork({ proxy: event.target.value || null })}
            />
          </Field>
          <Field label="Retries">
            <NumberInput
              value={draft.network.retries}
              min={0}
              max={10}
              onChange={(retries) => setNetwork({ retries })}
            />
          </Field>
          <h3 className="text-sm font-semibold">APIs</h3>
          <Field
            label="CurseForge API key"
            hint="Optional. Replaces the key this build comes with. Network and API changes apply after a restart."
          >
            <Input
              type="password"
              placeholder="Built-in key"
              value={draft.apis.curseforgeApiKey ?? ''}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  apis: { curseforgeApiKey: event.target.value || null },
                }))
              }
            />
          </Field>
        </>
      )}
      {tab === 'java' && (
        <>
          <Field label="Java installation">
            <JavaPicker value={draft.java.path} onChange={(path) => setJava({ path })} />
          </Field>
          <Field
            label="Download Java automatically"
            hint="Fetch Mojang's Java runtime when no installed Java fits the game."
          >
            <Checkbox
              checked={draft.java.autoDownload}
              onCheckedChange={(checked) => setJava({ autoDownload: checked === true })}
            />
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
          <Field label="Maximum memory" hint={memoryHint}>
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

      {tab === 'commands' && (
        <CommandFields
          value={draft.commands}
          onChange={(commands) => setDraft((current) => ({ ...current, commands }))}
        />
      )}

      {tab === 'tools' && (
        <>
          <h3 className="text-sm font-semibold">Profilers</h3>
          <Field label="JProfiler folder" hint="The installation folder that contains bin/jpenable.">
            <PathInput
              directory
              placeholder="Not set"
              value={draft.tools.jprofilerPath}
              onChange={(jprofilerPath) => setTools({ jprofilerPath })}
            />
          </Field>
          <Field label="JProfiler port">
            <NumberInput
              value={draft.tools.jprofilerPort}
              min={1024}
              max={65535}
              onChange={(jprofilerPort) => setTools({ jprofilerPort })}
            />
          </Field>
          <Field label="VisualVM" hint="Leave empty to look for jvisualvm or visualvm on the PATH.">
            <PathInput
              placeholder="Found on the PATH"
              value={draft.tools.visualvmPath}
              onChange={(visualvmPath) => setTools({ visualvmPath })}
            />
          </Field>
          <h3 className="text-sm font-semibold">Editor</h3>
          <Field
            label="Text editor"
            hint="Command that opens instance files, for example code or subl. Leave empty to use the default app."
          >
            <Input
              placeholder="Default app"
              value={draft.tools.editor ?? ''}
              onChange={(event) => setTools({ editor: event.target.value || null })}
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
