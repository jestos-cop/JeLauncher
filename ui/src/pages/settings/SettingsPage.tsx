import { Coffee, Monitor, Rocket, Terminal, UsersRound, Wrench } from 'lucide-react'
import { useState } from 'react'
import { Trans, useTranslation } from 'react-i18next'
import { useNavigation, type SettingsTab } from '@/app/navigation'
import { commands, type GlobalSettings, type SourceLookupOrder, type UpdateChannel } from '@/bindings'
import { CommandFields } from '@/components/CommandFields'
import {
  Field,
  JavaPicker,
  SettingsSection,
  LanguageSelect,
  NumberInput,
  PathInput,
  ThemeSelect,
} from '@/components/SettingsFields'
import { SaveBar } from '@/components/SaveBar'
import { SidebarLayout, type SidebarTab } from '@/components/SidebarLayout'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useMemoryHint, useSettings, useUpdateSettings } from '@/hooks/use-settings'
import { unwrap } from '@/lib/command'
import { AccountsSettings } from '@/pages/settings/AccountsSettings'

const UPDATE_CHANNELS: UpdateChannel[] = ['release', 'beta', 'alpha']
const SOURCE_LOOKUP_ORDERS: SourceLookupOrder[] = ['modrinthFirst', 'curseForgeFirst']
const MODRINTH_TOKEN_DOCS = 'https://docs.modrinth.com/api/#authentication'

const TABS: Omit<SidebarTab<SettingsTab>, 'label'>[] = [
  { tab: 'launcher', icon: Rocket },
  { tab: 'accounts', icon: UsersRound },
  { tab: 'java', icon: Coffee },
  { tab: 'minecraft', icon: Monitor },
  { tab: 'commands', icon: Terminal },
  { tab: 'tools', icon: Wrench },
]

export function SettingsPage({ tab }: { tab: SettingsTab }) {
  const { t } = useTranslation(['settings', 'common'])
  const { navigate } = useNavigation()
  const { data: saved } = useSettings()

  return (
    <SidebarLayout
      title={t('title')}
      tabs={TABS.map((entry) => ({ ...entry, label: t(`tabs.${entry.tab}`) }))}
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
  const { t } = useTranslation(['settings', 'common'])
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
  const setApis = (patch: Partial<GlobalSettings['apis']>) =>
    setDraft((current) => ({ ...current, apis: { ...current.apis, ...patch } }))
  const setNetwork = (patch: Partial<GlobalSettings['network']>) =>
    setDraft((current) => ({ ...current, network: { ...current.network, ...patch } }))

  return (
    <div className="flex min-h-full max-w-4xl flex-col gap-5">
      {tab === 'launcher' && (
        <>
          <SettingsSection title={t('launcher.appearance')}>
            <Field label={t('launcher.theme')}>
              <ThemeSelect
                value={draft.appearance.theme}
                onChange={(theme) =>
                  setDraft((current) => ({ ...current, appearance: { ...current.appearance, theme } }))
                }
              />
            </Field>
            <Field label={t('launcher.language')}>
              <LanguageSelect
                value={draft.appearance.language}
                onChange={(language) =>
                  setDraft((current) => ({ ...current, appearance: { ...current.appearance, language } }))
                }
              />
            </Field>
            <Field label={t('launcher.showNewsBar')}>
              <Checkbox
                checked={draft.appearance.showNewsBar}
                onCheckedChange={(checked) =>
                  setDraft((current) => ({
                    ...current,
                    appearance: { ...current.appearance, showNewsBar: checked === true },
                  }))
                }
              />
            </Field>
          </SettingsSection>
          <SettingsSection title={t('launcher.console')}>
            <Field label={t('launcher.showConsole')}>
              <Checkbox
                checked={draft.launcher.showConsole}
                onCheckedChange={(checked) => setLauncher({ showConsole: checked === true })}
              />
            </Field>
            <Field label={t('launcher.showConsoleOnError')}>
              <Checkbox
                checked={draft.launcher.showConsoleOnError}
                onCheckedChange={(checked) => setLauncher({ showConsoleOnError: checked === true })}
              />
            </Field>
            <Field label={t('launcher.autoCloseConsole')}>
              <Checkbox
                checked={draft.launcher.autoCloseConsole}
                onCheckedChange={(checked) => setLauncher({ autoCloseConsole: checked === true })}
              />
            </Field>
          </SettingsSection>
          <SettingsSection title={t('launcher.gameTime')}>
            <Field label={t('launcher.recordGameTime')}>
              <Checkbox
                checked={draft.launcher.recordGameTime}
                onCheckedChange={(checked) => setLauncher({ recordGameTime: checked === true })}
              />
            </Field>
          </SettingsSection>
          <SettingsSection title={t('launcher.resources')}>
            <Field label={t('launcher.trackSources')} hint={t('launcher.trackSourcesHint')}>
              <Checkbox
                checked={draft.launcher.trackResourceSources}
                onCheckedChange={(checked) => setLauncher({ trackResourceSources: checked === true })}
              />
            </Field>
            <Field label={t('launcher.updateChannel')} hint={t('launcher.updateChannelHint')}>
              <Select
                value={draft.launcher.updateChannel}
                onValueChange={(channel) => setLauncher({ updateChannel: channel as UpdateChannel })}
              >
                <SelectTrigger className="w-56">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {UPDATE_CHANNELS.map((channel) => (
                    <SelectItem key={channel} value={channel}>
                      {t(`launcher.channel.${channel}`)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </Field>
            <Field label={t('launcher.sourceLookupOrder')} hint={t('launcher.sourceLookupOrderHint')}>
              <Select
                value={draft.launcher.sourceLookupOrder}
                onValueChange={(order) => setLauncher({ sourceLookupOrder: order as SourceLookupOrder })}
              >
                <SelectTrigger className="w-56">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {SOURCE_LOOKUP_ORDERS.map((order) => (
                    <SelectItem key={order} value={order}>
                      {t(`launcher.lookupOrder.${order}`)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </Field>
          </SettingsSection>
          <SettingsSection title={t('launcher.network')}>
            <Field label={t('launcher.concurrentDownloads')}>
              <NumberInput
                value={draft.network.concurrentDownloads}
                min={1}
                max={64}
                onChange={(concurrentDownloads) => setNetwork({ concurrentDownloads })}
              />
            </Field>
            <Field label={t('launcher.requestTimeout')} hint={t('launcher.requestTimeoutHint')}>
              <NumberInput
                value={draft.network.requestTimeoutSecs}
                min={5}
                suffix={t('units.seconds')}
                onChange={(requestTimeoutSecs) => setNetwork({ requestTimeoutSecs })}
              />
            </Field>
            <Field label={t('launcher.proxy')} hint={t('launcher.proxyHint')}>
              <Input
                placeholder={t('launcher.proxyPlaceholder')}
                value={draft.network.proxy ?? ''}
                onChange={(event) => setNetwork({ proxy: event.target.value || null })}
              />
            </Field>
            <Field label={t('launcher.retries')}>
              <NumberInput
                value={draft.network.retries}
                min={0}
                max={10}
                onChange={(retries) => setNetwork({ retries })}
              />
            </Field>
          </SettingsSection>
          <SettingsSection title={t('launcher.apis')}>
            <p className="text-xs text-muted-foreground">{t('launcher.apisRestartHint')}</p>
            <Field label={t('launcher.microsoftClientId')} hint={t('launcher.microsoftClientIdHint')}>
              <Input
                placeholder={t('launcher.builtInPlaceholder')}
                value={draft.apis.microsoftClientId ?? ''}
                onChange={(event) => setApis({ microsoftClientId: event.target.value || null })}
              />
            </Field>
            <Field
              label={t('launcher.modrinthToken')}
              hint={
                <Trans
                  t={t}
                  i18nKey="launcher.modrinthTokenHint"
                  components={{
                    open: (
                      <button
                        type="button"
                        className="text-primary underline-offset-2 hover:underline"
                        onClick={() => void unwrap(commands.openUrl(MODRINTH_TOKEN_DOCS))}
                      />
                    ),
                  }}
                />
              }
            >
              <Input
                type="password"
                placeholder={t('launcher.notUsedPlaceholder')}
                value={draft.apis.modrinthToken ?? ''}
                onChange={(event) => setApis({ modrinthToken: event.target.value || null })}
              />
            </Field>
            <Field label={t('launcher.curseforgeApiKey')} hint={t('launcher.curseforgeApiKeyHint')}>
              <Input
                type="password"
                placeholder={t('launcher.builtInPlaceholder')}
                value={draft.apis.curseforgeApiKey ?? ''}
                onChange={(event) => setApis({ curseforgeApiKey: event.target.value || null })}
              />
            </Field>
            <Field label={t('launcher.modrinthFallback')} hint={t('launcher.modrinthFallbackHint')}>
              <Checkbox
                checked={draft.apis.modrinthFallbackForBlocked}
                onCheckedChange={(checked) => setApis({ modrinthFallbackForBlocked: checked === true })}
              />
            </Field>
            <Field label={t('launcher.technicClientId')} hint={t('launcher.technicClientIdHint')}>
              <Input
                type="password"
                placeholder={t('launcher.notUsedPlaceholder')}
                value={draft.apis.technicClientId ?? ''}
                onChange={(event) => setApis({ technicClientId: event.target.value || null })}
              />
            </Field>
          </SettingsSection>
        </>
      )}
      {tab === 'java' && (
        <SettingsSection>
          <Field label={t('java.installation')}>
            <JavaPicker value={draft.java.path} onChange={(path) => setJava({ path })} />
          </Field>
          <Field label={t('java.autoDownload')} hint={t('java.autoDownloadHint')}>
            <Checkbox
              checked={draft.java.autoDownload}
              onCheckedChange={(checked) => setJava({ autoDownload: checked === true })}
            />
          </Field>
          <Field label={t('java.minMemory')}>
            <NumberInput
              value={draft.java.minMemoryMb}
              min={128}
              step={128}
              suffix={t('units.mib')}
              onChange={(minMemoryMb) => setJava({ minMemoryMb })}
            />
          </Field>
          <Field label={t('java.maxMemory')} hint={memoryHint}>
            <NumberInput
              value={draft.java.maxMemoryMb}
              min={512}
              step={256}
              suffix={t('units.mib')}
              onChange={(maxMemoryMb) => setJava({ maxMemoryMb })}
            />
          </Field>
          <Field label={t('java.jvmArgs')}>
            <Input
              placeholder="-XX:+UseG1GC"
              value={draft.java.jvmArgs}
              onChange={(event) => setJava({ jvmArgs: event.target.value })}
            />
          </Field>
        </SettingsSection>
      )}
      {tab === 'minecraft' && (
        <SettingsSection>
          <Field label={t('minecraft.windowSize')}>
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
          <Field label={t('minecraft.maximized')}>
            <Checkbox
              checked={draft.window.maximized}
              onCheckedChange={(checked) => setWindow({ maximized: checked === true })}
            />
          </Field>
        </SettingsSection>
      )}

      {tab === 'commands' && (
        <SettingsSection>
          <CommandFields
            value={draft.commands}
            onChange={(commands) => setDraft((current) => ({ ...current, commands }))}
          />
        </SettingsSection>
      )}

      {tab === 'tools' && (
        <>
          <SettingsSection title={t('tools.profilers')}>
            <Field label={t('tools.jprofilerPath')} hint={t('tools.jprofilerPathHint')}>
              <PathInput
                directory
                placeholder={t('tools.notSet')}
                value={draft.tools.jprofilerPath}
                onChange={(jprofilerPath) => setTools({ jprofilerPath })}
              />
            </Field>
            <Field label={t('tools.jprofilerPort')}>
              <NumberInput
                value={draft.tools.jprofilerPort}
                min={1024}
                max={65535}
                onChange={(jprofilerPort) => setTools({ jprofilerPort })}
              />
            </Field>
            <Field label={t('tools.visualvm')} hint={t('tools.visualvmHint')}>
              <PathInput
                placeholder={t('tools.visualvmPlaceholder')}
                value={draft.tools.visualvmPath}
                onChange={(visualvmPath) => setTools({ visualvmPath })}
              />
            </Field>
          </SettingsSection>
          <SettingsSection title={t('tools.editor')}>
            <Field label={t('tools.textEditor')} hint={t('tools.textEditorHint')}>
              <Input
                placeholder={t('tools.defaultApp')}
                value={draft.tools.editor ?? ''}
                onChange={(event) => setTools({ editor: event.target.value || null })}
              />
            </Field>
          </SettingsSection>
        </>
      )}

      {draft.java.minMemoryMb > draft.java.maxMemoryMb && (
        <p className="text-sm text-destructive">{t('java.memoryRange')}</p>
      )}
      {update.error && <p className="text-sm text-destructive">{update.error.message}</p>}
      <SaveBar
        isDirty={isDirty}
        canSave={!update.isPending && draft.java.minMemoryMb <= draft.java.maxMemoryMb}
        onSave={() => update.mutate(draft)}
        onReset={() => setDraft(saved)}
      />
    </div>
  )
}
