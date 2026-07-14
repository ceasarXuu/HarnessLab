import { useRef, useState } from 'react'
import { FolderOpen } from 'lucide-react'
import { agentCapabilitiesForHarness, supportsAgentField } from '../../domain/agentCapabilities'
import type { AgentCapabilities, AgentCapabilityField, AgentRow } from '../../domain/harbor'
import type { Translate } from '../../i18n'
import { AgentHarnessParameters } from './AgentHarnessParameters'
import { EditableStringList } from './EditableStringList'
import { KeyValueControl } from './KeyValueControl'
import { Metric } from './Metric'
import { McpServersControl } from './McpServersControl'
import { ReadonlyKeyValueList, ReadonlyMcpServers, ReadonlyStringList } from './ReadonlyDisplay'

type AgentTab = 'base' | 'skills' | 'mcps' | 'advanced'

interface AgentProfileEditorProps {
  capabilitiesByHarness?: Record<string, AgentCapabilities>
  readOnly?: boolean
  value: AgentRow
  t: Translate
  onChange: (value: AgentRow) => void
}

const harnessOptions = [
  'acp', 'aider', 'antigravity-cli', 'claude-code', 'cline-cli', 'codex', 'copilot-cli',
  'cursor-cli', 'devin', 'gemini-cli', 'goose', 'hermes', 'kimi-cli', 'langgraph', 'mini-swe-agent',
  'nemo-agent', 'nop', 'openclaw', 'opencode', 'openhands', 'openhands-sdk', 'oracle', 'pi',
  'qwen-coder', 'rovodev-cli', 'swe-agent', 'terminus', 'terminus-1', 'terminus-2', 'trae-agent',
  'custom-harness',
]

export function AgentProfileEditor({
  capabilitiesByHarness,
  readOnly = false,
  value,
  t,
  onChange,
}: AgentProfileEditorProps) {
  const [activeTab, setActiveTab] = useState<AgentTab>('base')
  const capabilities = value.capabilities ?? agentCapabilitiesForHarness(value.harness, capabilitiesByHarness)
  const visibleFields = agentVisibleFields(value, capabilities, readOnly)
  const supports = (field: AgentCapabilityField) => visibleFields[field]
  const tabs = buildAgentTabs(visibleFields, t)
  const resolvedActiveTab = tabs.some((tab) => tab.key === activeTab) ? activeTab : tabs[0]?.key ?? 'base'
  const setField = (field: keyof AgentRow, nextValue: string) => onChange({ ...value, [field]: nextValue })

  if (!tabs.length) return null

  return (
    <div className="agent-profile-editor">
      <div className="run-tabs agent-detail-tabs" role="tablist" aria-label={t('selectedAgent')}>
        {tabs.map((tab) => (
          <button
            key={tab.key}
            type="button"
            role="tab"
            aria-selected={resolvedActiveTab === tab.key}
            className={resolvedActiveTab === tab.key ? 'active' : undefined}
            onClick={() => setActiveTab(tab.key)}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {resolvedActiveTab === 'base' && (
        <>
          {supports('modelName') && (
            <section className="surface rail-card">
              <SectionTitle>{t('modelSettings')}</SectionTitle>
              <div className="agent-form-grid">
                {readOnly ? (
                  <Metric label={t('supportedModels')} value={displayReadOnlyValue(value.models, t('configuredAtJobRun'))} />
                ) : (
                  <ModelListControl
                    addLabel={t('add')}
                    deleteLabel={t('delete')}
                    itemLabel={t('modelName')}
                    label={t('supportedModels')}
                    placeholder={t('modelNamePlaceholder')}
                    value={value.models}
                    onChange={(nextValue) => setField('models', nextValue)}
                  />
                )}
              </div>
            </section>
          )}

          {supports('env') && (
            <section className="surface rail-card">
              <SectionTitle>{t('credentialsAndParams')}</SectionTitle>
              <div className="agent-form-grid">
                {readOnly ? (
                  <ReadonlyKeyValueList label={t('genericAgentEnv')} value={value.env} emptyLabel={t('supportedByHarness')} />
                ) : (
                  <div className="field-wide">
                    <KeyValueControl
                      compact
                      readOnly={readOnly}
                      label={t('genericAgentEnv')}
                      labels={envKeyValueLabels(t)}
                      value={value.env ?? 'none'}
                      onChange={(nextValue) => setField('env', nextValue)}
                    />
                  </div>
                )}
              </div>
            </section>
          )}
        </>
      )}

      {resolvedActiveTab === 'skills' && supports('skills') && (
        <section className="surface rail-card">
          <SectionTitle>{t('skillsConfig')}</SectionTitle>
          {readOnly ? (
            <ReadonlyStringList label={t('skills')} value={value.skills} emptyLabel={t('supportedByHarness')} />
          ) : (
            <DirectoryListControl
              addLabel={t('add')}
              chooseLabel={t('chooseFolder')}
              deleteLabel={t('delete')}
              description={t('skillsConfigDescription')}
              label={t('skills')}
              readOnly={readOnly}
              value={value.skills ?? 'none'}
              onChange={(nextValue) => setField('skills', nextValue)}
            />
          )}
        </section>
      )}

      {resolvedActiveTab === 'mcps' && supports('mcpServers') && (
        <section className="surface rail-card">
          <SectionTitle>{t('mcpConfigSection')}</SectionTitle>
          {readOnly ? (
            <ReadonlyMcpServers label={t('mcpConfigSection')} labels={mcpLabels(t)} value={value.mcp} emptyLabel={t('supportedByHarness')} />
          ) : (
            <McpServersControl
              labels={mcpLabels(t)}
              readOnly={readOnly}
              value={value.mcp ?? 'none'}
              onChange={(nextValue) => setField('mcp', nextValue)}
            />
          )}
        </section>
      )}

      {resolvedActiveTab === 'advanced' && (
        <section className="surface rail-card">
          <SectionTitle>{t('advancedAgentParams')}</SectionTitle>
          <div className="agent-form-grid">
            {supports('timeouts') && (
              readOnly ? (
                <>
                  <Metric label={t('agentExecutionTimeout')} value={t('configuredAtJobRun')} />
                  <Metric label={t('setupTimeout')} value={t('configuredAtJobRun')} />
                  <Metric label={t('maxTimeout')} value={t('configuredAtJobRun')} />
                </>
              ) : (
                <>
                  <label>
                    {t('agentExecutionTimeout')}
                    <input
                      min="1"
                      type="number"
                      value={secondsInput(value.timeout)}
                      onChange={(event) => setField('timeout', withSeconds(event.target.value))}
                    />
                  </label>
                  <label>
                    {t('setupTimeout')}
                    <input
                      min="1"
                      type="number"
                      value={secondsInput(value.setupTimeout)}
                      onChange={(event) => setField('setupTimeout', withSeconds(event.target.value))}
                    />
                  </label>
                  <label>
                    {t('maxTimeout')}
                    <input
                      min="1"
                      type="number"
                      value={secondsInput(value.maxTimeout)}
                      onChange={(event) => setField('maxTimeout', withSeconds(event.target.value))}
                    />
                  </label>
                </>
              )
            )}
            {supports('customKwargs') && (
              readOnly ? (
                <ReadonlyKeyValueList label={t('genericAgentKwargs')} value={value.kwargs} emptyLabel={t('notConfigured')} />
              ) : (
                <KeyValueControl
                  label={t('genericAgentKwargs')}
                  labels={defaultKeyValueLabels(t)}
                  value={value.kwargs ?? 'none'}
                  onChange={(nextValue) => setField('kwargs', nextValue)}
                />
              )
            )}
            {supports('harnessParameters') && <AgentHarnessParameters readOnly={readOnly} t={t} value={value} onChange={onChange} />}
          </div>
        </section>
      )}
    </div>
  )
}

export function AgentIdentityEditor({
  capabilitiesByHarness,
  lockHarness = false,
  value,
  t,
  onChange,
}: AgentProfileEditorProps & { lockHarness?: boolean }) {
  const usesCustomHarness = value.harness === 'custom-harness'
  const setField = (field: keyof AgentRow, nextValue: string) => onChange({ ...value, [field]: nextValue })
  const setHarness = (harness: string) => onChange({
    ...value,
    adapter: harness === 'custom-harness' ? value.adapter || 'agents.custom:Agent' : 'none',
    capabilities: agentCapabilitiesForHarness(harness, capabilitiesByHarness),
    harness,
  })

  return (
    <div className="agent-form-grid">
      <label>
        {t('agentName')}
        <input value={value.agentName} onChange={(event) => setField('agentName', event.target.value)} />
      </label>
      {lockHarness ? (
        <Metric label={t('harness')} value={value.harness} variant="plain" />
      ) : (
        <label>
          {t('harness')}
          <select value={value.harness} onChange={(event) => setHarness(event.target.value)}>
            {harnessOptions.map((harness) => (
              <option key={harness} value={harness}>{harness}</option>
            ))}
          </select>
        </label>
      )}
      <Metric
        label={t('agentResourceType')}
        value={value.type === 'built-in' ? t('harborBuiltInHarness') : t('customHarness')}
        variant="plain"
      />
      {usesCustomHarness && (
        <label>
          {t('customImportPath')}
          <input value={value.adapter} onChange={(event) => setField('adapter', event.target.value)} />
        </label>
      )}
    </div>
  )
}

function mcpLabels(t: Translate) {
  return {
    addServer: t('addMcpServer'), args: t('mcpArgs'), command: t('mcpCommand'),
    deleteItem: t('deleteItem'), deleteServer: t('deleteMcpServer'),
    name: t('mcpServerName'), transport: t('mcpTransport'), url: t('mcpUrl'),
  }
}

type AgentVisibleFields = Record<AgentCapabilityField, boolean>

function agentVisibleFields(agent: AgentRow, capabilities: AgentCapabilities, readOnly: boolean): AgentVisibleFields {
  return {
    customKwargs: fieldVisible(agent, capabilities, 'customKwargs', readOnly),
    env: fieldVisible(agent, capabilities, 'env', readOnly),
    harnessParameters: fieldVisible(agent, capabilities, 'harnessParameters', readOnly),
    mcpServers: fieldVisible(agent, capabilities, 'mcpServers', readOnly),
    modelName: fieldVisible(agent, capabilities, 'modelName', readOnly),
    skills: fieldVisible(agent, capabilities, 'skills', readOnly),
    timeouts: fieldVisible(agent, capabilities, 'timeouts', readOnly),
  }
}

function fieldVisible(
  agent: AgentRow,
  capabilities: AgentCapabilities,
  field: AgentCapabilityField,
  readOnly: boolean,
) {
  if (!supportsAgentField(capabilities, field)) return false
  if (!readOnly) return true
  if (field === 'modelName') return hasConfiguredValue(agent.models)
  if (field === 'env') return hasConfiguredValue(agent.env)
  if (field === 'skills') return hasConfiguredValue(agent.skills)
  if (field === 'mcpServers') return hasConfiguredValue(agent.mcp)
  if (field === 'customKwargs') return hasConfiguredValue(agent.kwargs)
  if (field === 'timeouts') return Boolean(
    hasConfiguredValue(agent.timeout)
    || hasConfiguredValue(agent.setupTimeout)
    || hasConfiguredValue(agent.maxTimeout),
  )
  return hasConfiguredHarnessParameters(agent, capabilities)
}

function hasConfiguredHarnessParameters(agent: AgentRow, capabilities: AgentCapabilities) {
  return capabilities.parameters.some((parameter) => {
    const raw = parameter.source === 'env' ? agent.env : agent.kwargs
    return keyValueHasKey(raw, parameter.key)
  })
}

function keyValueHasKey(value: string | undefined, key: string) {
  if (!hasConfiguredValue(value)) return false
  return value?.split('\n').some((line) => line.split('=')[0]?.trim() === key) ?? false
}

function buildAgentTabs(visibleFields: AgentVisibleFields, t: Translate) {
  const hasBase = visibleFields.modelName || visibleFields.env
  const hasAdvanced = visibleFields.timeouts
    || visibleFields.customKwargs
    || visibleFields.harnessParameters
  return [
    hasBase ? { key: 'base' as const, label: t('runTabCore') } : null,
    visibleFields.skills ? { key: 'skills' as const, label: t('skillsTab') } : null,
    visibleFields.mcpServers ? { key: 'mcps' as const, label: t('mcpsTab') } : null,
    hasAdvanced ? { key: 'advanced' as const, label: t('agentAdvancedTab') } : null,
  ].filter((tab): tab is { key: AgentTab; label: string } => tab !== null)
}

function displayReadOnlyValue(value: string | undefined, fallback: string) {
  if (!hasConfiguredValue(value) || value === '-') return fallback
  return value ?? fallback
}

function hasConfiguredValue(value: string | undefined) {
  return Boolean(value && value !== 'none' && value !== '-')
}

function defaultKeyValueLabels(t: Translate) {
  return { add: t('add'), delete: t('delete'), key: t('formKey'), value: t('value') }
}

function envKeyValueLabels(t: Translate) {
  return { add: t('add'), delete: t('delete'), key: t('envKey'), value: t('envValue') }
}

function DirectoryListControl({
  addLabel,
  chooseLabel,
  deleteLabel,
  description,
  label,
  readOnly = false,
  value,
  onChange,
}: {
  addLabel: string
  chooseLabel: string
  deleteLabel: string
  description: string
  label: string
  readOnly?: boolean
  value: string
  onChange: (value: string) => void
}) {
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [paths, setPaths] = useState(() => parseDirectoryPaths(value))
  const commit = (nextPaths: string[]) => {
    setPaths(nextPaths)
    onChange(formatDirectoryPaths(nextPaths))
  }
  const updateFromFiles = (files: FileList | null) => {
    const firstFile = files?.item(0)
    const selectedPath = firstFile ? getSelectedDirectoryPath(firstFile) : ''
    if (!selectedPath) return
    commit([...paths.filter((path) => path.trim()), selectedPath])
    if (fileInputRef.current) fileInputRef.current.value = ''
  }

  return (
    <div className="directory-list-control field-wide">
      <p className="field-hint">{description}</p>
      <EditableStringList
        addLabel={addLabel}
        className="directory-string-list"
        deleteLabel={deleteLabel}
        itemAriaLabel={() => label}
        label={label}
        readOnly={readOnly}
        values={paths}
        onChange={commit}
        extraActions={(
          <button className="secondary-button compact-action" type="button" onClick={() => fileInputRef.current?.click()}><FolderOpen aria-hidden="true" />{chooseLabel}</button>
        )}
      />
      <input ref={fileInputRef} aria-label={`${chooseLabel}: ${label}`} className="visually-hidden" type="file" onChange={(event) => updateFromFiles(event.target.files)} {...{ directory: '', webkitdirectory: '' }} />
    </div>
  )
}

function SectionTitle({ children }: { children: string }) {
  return <div className="rail-title"><h3>{children}</h3></div>
}

function ModelListControl({
  addLabel,
  deleteLabel,
  itemLabel,
  label,
  placeholder,
  readOnly = false,
  value,
  onChange,
}: {
  addLabel: string
  deleteLabel: string
  itemLabel: string
  label: string
  placeholder: string
  readOnly?: boolean
  value: string
  onChange: (value: string) => void
}) {
  const [models, setModels] = useState(() => parseModelNames(value))
  const commit = (nextModels: string[]) => {
    setModels(nextModels)
    onChange(formatModelNames(nextModels))
  }
  return (
    <EditableStringList
      addLabel={addLabel}
      className="model-list-control field-wide"
      deleteLabel={deleteLabel}
      itemAriaLabel={() => itemLabel}
      label={label}
      placeholder={placeholder}
      readOnly={readOnly}
      values={models}
      onChange={commit}
    />
  )
}

function parseModelNames(value: string) {
  if (!value || value === '-') return []
  const names = parseList(value)
  return names
}

function parseList(value: string) {
  return value.split(',').map((item) => item.trim()).filter(Boolean)
}

function formatModelNames(models: string[]) {
  const names = models.map((item) => item.trim()).filter(Boolean)
  return names.join(', ')
}

function parseDirectoryPaths(value: string) {
  if (!value || value === 'none') return []
  const paths = value.split(/\n|,/).map((item) => item.trim()).filter(Boolean)
  return paths
}

function formatDirectoryPaths(paths: string[]) {
  const cleanPaths = paths.map((item) => item.trim()).filter(Boolean)
  return cleanPaths.length ? cleanPaths.join('\n') : 'none'
}

function getSelectedDirectoryPath(file: File) {
  const relativePath = file.webkitRelativePath
  const folderName = relativePath?.split('/').filter(Boolean)[0]
  const fullPath = (file as File & { path?: string }).path
  if (fullPath && relativePath && folderName && fullPath.endsWith(relativePath)) return `${fullPath.slice(0, -relativePath.length)}${folderName}`
  return folderName ?? fullPath ?? ''
}

function secondsInput(value: string | undefined) {
  return value?.replace(/s$/, '') ?? ''
}

function withSeconds(value: string) {
  return value ? `${value}s` : ''
}

export function getAgentStatusLabel(status: AgentRow['status'], t: Translate) {
  if (status === 'needs-token') return t('agentStatusNeedsToken')
  if (status === 'configured') return t('agentStatusConfigured')
  return t('agentStatusAvailable')
}
