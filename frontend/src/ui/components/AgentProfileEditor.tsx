import { useMemo, useRef, useState } from 'react'
import { FolderOpen, Plus, Trash2 } from 'lucide-react'
import type { AgentRow } from '../../mocks/demo'
import type { Translate } from '../../i18n'
import { KeyValueControl } from './KeyValueControl'
import { Metric } from './Metric'
import { McpServersControl } from './McpServersControl'

type AgentTab = 'base' | 'skills' | 'mcps' | 'advanced'

interface AgentProfileEditorProps {
  value: AgentRow
  t: Translate
  onChange: (value: AgentRow) => void
}

interface HarnessConfig {
  reasoning?: string[]
  reasoningSummary?: string[]
  temperature?: boolean
  contextLength?: boolean
  credentials?: 'anthropic' | 'openai' | 'custom' | 'none'
}

const harnessConfigs: Record<string, HarnessConfig> = {
  'claude-code': { reasoning: ['low', 'medium', 'high', 'xhigh', 'max'], credentials: 'anthropic' },
  codex: {
    reasoning: ['minimal', 'low', 'medium', 'high'],
    reasoningSummary: ['auto', 'concise', 'detailed', 'none'],
    credentials: 'openai',
  },
  'codex-cli': {
    reasoning: ['minimal', 'low', 'medium', 'high'],
    reasoningSummary: ['auto', 'concise', 'detailed', 'none'],
    credentials: 'openai',
  },
  'qwen-coder': { credentials: 'openai' },
  'custom-harness': { temperature: true, contextLength: true, credentials: 'custom' },
  oracle: { credentials: 'none' },
}

export function AgentProfileEditor({ value, t, onChange }: AgentProfileEditorProps) {
  const [activeTab, setActiveTab] = useState<AgentTab>('base')
  const config = useMemo(() => harnessConfigs[value.harness] ?? harnessConfigs['custom-harness'], [value.harness])
  const setField = (field: keyof AgentRow, nextValue: string) => onChange({ ...value, [field]: nextValue })

  return (
    <div className="agent-profile-editor">
      <div className="run-tabs agent-detail-tabs" role="tablist" aria-label={t('selectedAgent')}>
        {[
          { key: 'base' as const, label: t('runTabCore') },
          { key: 'skills' as const, label: 'Skills' },
          { key: 'mcps' as const, label: 'MCPs' },
          { key: 'advanced' as const, label: t('agentAdvancedTab') },
        ].map((tab) => (
          <button
            key={tab.key}
            type="button"
            role="tab"
            aria-selected={activeTab === tab.key}
            className={activeTab === tab.key ? 'active' : undefined}
            onClick={() => setActiveTab(tab.key)}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {activeTab === 'base' && (
        <>
          <section className="surface rail-card">
            <SectionTitle>{t('modelSettings')}</SectionTitle>
            <div className="agent-form-grid">
              <ModelListControl label={t('supportedModels')} value={value.models} onChange={(nextValue) => setField('models', nextValue)} />
              {config.reasoning && (
                <TagGroupControl label={t('reasoningEffort')} options={config.reasoning} value={value.reasoningEffort ?? ''} onChange={(nextValue) => setField('reasoningEffort', nextValue)} />
              )}
              {config.reasoningSummary && (
                <label>
                  {t('reasoningSummary')}
                  <select value={value.reasoningSummary ?? ''} onChange={(event) => setField('reasoningSummary', event.target.value)}>
                    <option value="">-</option>
                    {config.reasoningSummary.map((option) => <option key={option} value={option}>{option}</option>)}
                  </select>
                </label>
              )}
              {config.temperature && (
                <label>{t('temperature')}<input inputMode="decimal" value={value.temperature ?? ''} onChange={(event) => setField('temperature', event.target.value)} /></label>
              )}
              {config.contextLength && (
                <label>{t('contextLength')}<input inputMode="numeric" value={value.contextLength ?? ''} onChange={(event) => setField('contextLength', event.target.value)} /></label>
              )}
            </div>
          </section>

          <section className="surface rail-card">
            <SectionTitle>{t('credentialsAndParams')}</SectionTitle>
            <div className="agent-form-grid">
              {config.credentials !== 'none' && (
                <>
                  <label>{t('apiKeyEnv')}<input value={value.apiKeyEnv ?? ''} onChange={(event) => setField('apiKeyEnv', event.target.value)} /></label>
                  <label>{t('baseUrlEnv')}<input value={value.baseUrlEnv ?? ''} onChange={(event) => setField('baseUrlEnv', event.target.value)} /></label>
                </>
              )}
              <div className="field-wide">
                <KeyValueControl compact label={t('genericAgentEnv')} value={value.env ?? 'none'} onChange={(nextValue) => setField('env', nextValue)} />
              </div>
            </div>
          </section>

          <section className="surface rail-card">
            <SectionTitle>{t('networkAccess')}</SectionTitle>
            <NetworkAccessControl enabledLabel={t('networkAccessToggle')} hostsLabel={t('agentAllowedHosts')} value={value.allowedHosts ?? '*'} onChange={(nextValue) => setField('allowedHosts', nextValue)} />
          </section>
        </>
      )}

      {activeTab === 'skills' && (
        <section className="surface rail-card">
          <SectionTitle>{t('skillsConfig')}</SectionTitle>
          <DirectoryListControl chooseLabel={t('chooseFolder')} description={t('skillsConfigDescription')} label={t('skills')} value={value.skills ?? 'none'} onChange={(nextValue) => setField('skills', nextValue)} />
        </section>
      )}

      {activeTab === 'mcps' && (
        <section className="surface rail-card">
          <SectionTitle>{t('mcpConfigSection')}</SectionTitle>
          <McpServersControl labels={mcpLabels(t)} value={value.mcp ?? 'none'} onChange={(nextValue) => setField('mcp', nextValue)} />
        </section>
      )}

      {activeTab === 'advanced' && (
        <section className="surface rail-card">
          <SectionTitle>{t('advancedAgentParams')}</SectionTitle>
          <div className="agent-form-grid">
            <KeyValueControl label={t('genericAgentKwargs')} value={value.kwargs ?? 'none'} onChange={(nextValue) => setField('kwargs', nextValue)} />
            <Metric label={t('runtimeDefaults')} value={value.runtime ?? '-'} />
            <Metric label={t('setupTimeout')} value={value.setupTimeout ?? '-'} />
            <Metric label={t('maxTimeout')} value={value.maxTimeout ?? '-'} />
            <Metric label={t('sourceRef')} value={value.source} />
            <Metric label={t('updated')} value={value.updated} />
          </div>
        </section>
      )}
    </div>
  )
}

export function AgentIdentityEditor({ value, t, onChange }: AgentProfileEditorProps) {
  const isCustomAgent = value.type === 'custom' || value.harness === 'custom-harness'
  const setField = (field: keyof AgentRow, nextValue: string) => onChange({ ...value, [field]: nextValue })

  return (
    <div className="agent-form-grid">
      <label>
        {t('agentName')}
        <input value={value.agentName} onChange={(event) => setField('agentName', event.target.value)} />
      </label>
      <label>
        {t('harness')}
        <select value={value.harness} onChange={(event) => setField('harness', event.target.value)}>
          {Object.keys(harnessConfigs).map((harness) => (
            <option key={harness} value={harness}>{harness}</option>
          ))}
        </select>
      </label>
      <label>
        {t('agentType')}
        <input readOnly value={value.type} />
      </label>
      {isCustomAgent && (
        <label>
          {t('customImportPath')}
          <input value={value.adapter} onChange={(event) => setField('adapter', event.target.value)} />
        </label>
      )}
    </div>
  )
}

function NetworkAccessControl({ enabledLabel, hostsLabel, value, onChange }: { enabledLabel: string; hostsLabel: string; value: string; onChange: (value: string) => void }) {
  const normalizedValue = value.trim() || '*'
  const enabled = normalizedValue !== 'none'
  const hostsValue = enabled ? normalizedValue : '*'

  return (
    <div className="network-access-control">
      <label className="switch-control network-access-toggle">
        <span>{enabledLabel}</span>
        <input aria-label={enabledLabel} checked={enabled} type="checkbox" onChange={(event) => onChange(event.target.checked ? '*' : 'none')} />
      </label>
      {enabled && (
        <label className="field-wide">
          {hostsLabel}
          <textarea value={hostsValue} onChange={(event) => onChange(event.target.value.trim() || '*')} />
        </label>
      )}
    </div>
  )
}

function mcpLabels(t: Translate) {
  return {
    addItem: t('add'), addServer: t('addMcpServer'), args: t('mcpArgs'), composeSidecar: t('mcpComposeSidecar'),
    composeYaml: t('mcpComposeYaml'), command: t('mcpCommand'), deployment: t('mcpDeployment'), description: t('mcpConfigDescription'),
    enabled: t('enabled'), endpointPath: t('mcpEndpointPath'), env: t('mcpEnv'), externalService: t('mcpExternalService'),
    generatedUrl: t('mcpGeneratedUrl'), name: t('mcpServerName'), port: t('mcpPort'), serviceName: t('mcpServiceName'),
    stdio: t('mcpStdio'), transport: t('mcpTransport'), url: t('mcpUrl'),
  }
}

function DirectoryListControl({ chooseLabel, description, label, value, onChange }: { chooseLabel: string; description: string; label: string; value: string; onChange: (value: string) => void }) {
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [paths, setPaths] = useState(() => parseDirectoryPaths(value))
  const commit = (nextPaths: string[]) => {
    setPaths(nextPaths.length ? nextPaths : [''])
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
      <div className="rule-list-header">
        <span>{label}</span>
        <div className="directory-list-actions">
          <button className="secondary-button compact-action" type="button" onClick={() => commit([...paths, ''])}><Plus aria-hidden="true" />Add</button>
          <button className="secondary-button compact-action" type="button" onClick={() => fileInputRef.current?.click()}><FolderOpen aria-hidden="true" />{chooseLabel}</button>
        </div>
      </div>
      <div className="rule-list-rows">
        {paths.map((path, index) => (
          <div className="rule-list-row" key={index}>
            <input aria-label={label} value={path} onChange={(event) => commit(paths.map((item, rowIndex) => rowIndex === index ? event.target.value : item))} />
            <button aria-label={`Delete ${label} ${path || index + 1}`} className="icon-button" type="button" onClick={() => commit(paths.filter((_, rowIndex) => rowIndex !== index))}><Trash2 aria-hidden="true" /></button>
          </div>
        ))}
      </div>
      <input ref={fileInputRef} aria-label={`${chooseLabel}: ${label}`} className="visually-hidden" type="file" onChange={(event) => updateFromFiles(event.target.files)} {...{ directory: '', webkitdirectory: '' }} />
    </div>
  )
}

function SectionTitle({ children }: { children: string }) {
  return <div className="rail-title"><h3>{children}</h3></div>
}

function ModelListControl({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  const [models, setModels] = useState(() => parseModelNames(value))
  const commit = (nextModels: string[]) => {
    setModels(nextModels.length ? nextModels : [''])
    onChange(formatModelNames(nextModels))
  }
  return (
    <div className="model-list-control field-wide">
      <div className="rule-list-header"><span>{label}</span><button className="secondary-button compact-action" type="button" onClick={() => commit([...models, ''])}><Plus aria-hidden="true" />Add</button></div>
      <div className="rule-list-rows">
        {models.map((modelName, index) => (
          <div className="rule-list-row" key={index}>
            <input aria-label="Model name" value={modelName} onChange={(event) => commit(models.map((item, rowIndex) => rowIndex === index ? event.target.value : item))} />
            <button aria-label={`Delete model ${modelName || index + 1}`} className="icon-button" type="button" onClick={() => commit(models.filter((_, rowIndex) => rowIndex !== index))}><Trash2 aria-hidden="true" /></button>
          </div>
        ))}
      </div>
    </div>
  )
}

function TagGroupControl({ label, options, value, onChange }: { label: string; options: string[]; value: string; onChange: (value: string) => void }) {
  const selected = new Set(parseList(value))
  const toggle = (option: string) => {
    const next = new Set(selected)
    if (next.has(option)) {
      next.delete(option)
    } else {
      next.add(option)
    }
    onChange(options.filter((item) => next.has(item)).join(', '))
  }
  return (
    <div className="tag-group-control field-wide">
      <span>{label}</span>
      <div className="tag-group" role="group" aria-label={label}>
        {options.map((option) => <button aria-pressed={selected.has(option)} className="tag-option" key={option} type="button" onClick={() => toggle(option)}>{option}</button>)}
      </div>
    </div>
  )
}

function parseModelNames(value: string) {
  const names = parseList(value)
  return names.length ? names : ['']
}

function parseList(value: string) {
  return value.split(',').map((item) => item.trim()).filter(Boolean)
}

function formatModelNames(models: string[]) {
  const names = models.map((item) => item.trim()).filter(Boolean)
  return names.join(', ')
}

function parseDirectoryPaths(value: string) {
  if (!value || value === 'none') return ['']
  const paths = value.split(/\n|,/).map((item) => item.trim()).filter(Boolean)
  return paths.length ? paths : ['']
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

export function getAgentStatusLabel(status: AgentRow['status'], t: Translate) {
  if (status === 'needs-token') return t('agentStatusNeedsToken')
  if (status === 'configured') return t('agentStatusConfigured')
  return t('agentStatusAvailable')
}
