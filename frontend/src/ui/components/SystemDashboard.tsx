import type { SystemRow } from '../../domain/harbor'
import type { MessageKey, Translate } from '../../i18n'

interface SystemDashboardProps {
  disabled: boolean
  dockerCommand: string
  dockerCommandError: string | null
  dockerCommandRunning: boolean
  rows: SystemRow[]
  t: Translate
  onCheckUpdate: () => void
  onCleanDockerCache: () => void
  onCleanStorageCache: () => void
  onDockerCommandBlur: () => void
  onDockerCommandChange: (command: string) => void
  onRunDockerCommand: () => void
  onRestartService: () => void
}

type SystemGroup = 'services' | 'storage' | 'resources'

const groupKinds: Record<SystemGroup, SystemRow['kind'][]> = {
  services: ['ornnlab-service', 'harbor-cli', 'docker'],
  storage: ['storage', 'resource-storage'],
  resources: ['resource-cpu', 'resource-gpu'],
}

export function SystemDashboard(props: SystemDashboardProps) {
  const { rows, t } = props
  if (rows.length === 0) return <div className="system-dashboard-empty">{t('noSystemComponents')}</div>

  return (
    <div className="system-dashboard">
      <SystemGroupSection {...props} group="services" title={t('systemServicesDependencies')} />
      <SystemGroupSection {...props} group="storage" title={t('systemStorageGroup')} />
      <SystemGroupSection {...props} group="resources" title={t('systemHostResources')} />
    </div>
  )
}

function SystemGroupSection({ group, rows, title, ...props }: SystemDashboardProps & { group: SystemGroup; title: string }) {
  const groupedRows = groupKinds[group].flatMap((kind) => rows.filter((row) => row.kind === kind))
  if (groupedRows.length === 0) return null
  return (
    <section className="system-dashboard-group" aria-labelledby={`system-${group}`}>
      <h2 id={`system-${group}`}>{title}</h2>
      <div className={`system-card-grid system-card-grid-${group}`}>
        {groupedRows.map((row) => <SystemCard key={row.kind} row={row} {...props} />)}
      </div>
    </section>
  )
}

function SystemCard({ row, t, disabled, dockerCommand, dockerCommandError, dockerCommandRunning, onCheckUpdate, onCleanDockerCache, onCleanStorageCache, onDockerCommandBlur, onDockerCommandChange, onRestartService, onRunDockerCommand }: Omit<SystemDashboardProps, 'rows'> & { row: SystemRow }) {
  const title = t(componentTitleKey(row.kind))
  const meter = resourceMeter(row)
  const message = componentMessage(row, t)
  return (
    <article className={`system-card system-card-${row.kind}`} aria-label={title}>
      <header className="system-card-header">
        <h3>{title}</h3>
        <SystemState state={row.state} t={t} />
      </header>
      {meter && <ResourceMeter value={meter.value} label={meter.label} />}
      <SystemCardDetails row={row} t={t} />
      {message && <p className={row.kind === 'docker' && row.state === 'not-running' ? 'system-card-notice' : 'system-card-error'}>{message}</p>}
      {row.kind === 'docker' && (
        <div className="system-docker-command">
          <label htmlFor="docker-start-command">{t('dockerStartCommand')}</label>
          <div>
            <input
              disabled={disabled || dockerCommandRunning}
              id="docker-start-command"
              maxLength={500}
              spellCheck={false}
              value={dockerCommand}
              onBlur={onDockerCommandBlur}
              onChange={(event) => onDockerCommandChange(event.target.value)}
            />
            <button
              className="secondary-button compact-action"
              disabled={disabled || dockerCommandRunning || !dockerCommand.trim()}
              onClick={onRunDockerCommand}
            >
              {dockerCommandRunning ? t('runningCommand') : t('runCommand')}
            </button>
          </div>
          {dockerCommandError && <p className="system-card-error" role="alert">{dockerCommandError}</p>}
        </div>
      )}
      {row.actions.length > 0 && (
        <div className="system-card-actions">
          {row.actions.includes('check-update') && <button className="secondary-button compact-action" disabled={disabled} onClick={onCheckUpdate}>{t('checkUpdate')}</button>}
          {row.actions.includes('restart-service') && <button className="secondary-button compact-action" disabled={disabled} onClick={onRestartService}>{t('restart')}</button>}
          {row.actions.includes('clean-docker-cache') && <button className="secondary-button compact-action" disabled={disabled} onClick={onCleanDockerCache}>{t('cleanCache')}</button>}
          {row.actions.includes('clean-storage-cache') && <button className="secondary-button compact-action" disabled={disabled} onClick={onCleanStorageCache}>{t('cleanCache')}</button>}
        </div>
      )}
    </article>
  )
}

function componentMessage(row: SystemRow, t: Translate): string | null {
  if (row.kind === 'docker') {
    if (row.state === 'not-running') return t('dockerNotRunningHelp')
    if (row.state === 'error') return t('dockerConnectionErrorHelp')
  }
  return 'error' in row ? row.error : null
}

function SystemCardDetails({ row, t }: { row: SystemRow; t: Translate }) {
  switch (row.kind) {
    case 'ornnlab-service':
      return <DetailList items={[[t('endpoint'), row.endpoint ?? '—'], [t('logsPath'), row.logsPath]]} codeIndexes={[1]} />
    case 'harbor-cli':
      return <DetailList items={[[t('version'), row.version ?? '—'], [t('executablePath'), row.executablePath]]} codeIndexes={[1]} />
    case 'docker':
      return <DetailList items={[
        [t('dockerContext'), row.context ?? '—'],
        [t('clientVersion'), row.clientVersion ?? '—'],
        [t('serverVersion'), row.serverVersion ?? '—'],
        [t('executablePath'), row.executablePath],
      ]} codeIndexes={[3]} />
    case 'storage':
      return <DetailList items={[[t('cacheSize'), formatBytes(row.sizeBytes)], [t('path'), row.path]]} codeIndexes={[1]} />
    case 'resource-cpu':
      return <DetailList items={[[t('logicalCores'), row.logicalCores?.toString() ?? '—']]} />
    case 'resource-gpu':
      return <DetailList items={[[t('gpuDevices'), row.deviceCount.toString()]]} />
    case 'resource-storage': {
      const available = formatBytes(row.availableBytes)
      const total = formatBytes(row.totalBytes)
      return <DetailList items={[[t('value'), interpolate(t('availableOfTotal'), { available, total })], [t('path'), row.path]]} codeIndexes={[1]} />
    }
  }
}

function DetailList({ items, codeIndexes = [] }: { items: [string, string][]; codeIndexes?: number[] }) {
  return (
    <dl className="system-card-details">
      {items.map(([label, value], index) => (
        <div key={label}>
          <dt>{label}</dt>
          <dd>{codeIndexes.includes(index) ? <code>{value}</code> : value}</dd>
        </div>
      ))}
    </dl>
  )
}

function SystemState({ state, t }: { state: SystemRow['state']; t: Translate }) {
  return <span className={`system-state system-state-${stateTone(state)}`}>{t(stateKey(state))}</span>
}

function ResourceMeter({ value, label }: { value: number; label: string }) {
  return (
    <div className="system-resource-summary">
      <strong>{label}</strong>
      <div className="system-resource-meter" role="meter" aria-label={label} aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(value)}>
        <span style={{ width: `${Math.max(0, Math.min(value, 100))}%` }} />
      </div>
    </div>
  )
}

function resourceMeter(row: SystemRow): { value: number; label: string } | null {
  if (row.kind === 'resource-cpu' || row.kind === 'resource-gpu') {
    return row.usagePercent === null ? null : { value: row.usagePercent, label: `${row.usagePercent}%` }
  }
  if (row.kind === 'resource-storage' && row.availableBytes !== null && row.totalBytes) {
    const availablePercent = (row.availableBytes / row.totalBytes) * 100
    return { value: availablePercent, label: `${Math.round(availablePercent)}%` }
  }
  return null
}

function componentTitleKey(kind: SystemRow['kind']): MessageKey {
  return {
    'ornnlab-service': 'ornnlabService', 'harbor-cli': 'harborCli', docker: 'docker', storage: 'localCache',
    'resource-cpu': 'cpuUsage', 'resource-gpu': 'gpuUsage', 'resource-storage': 'availableStorage',
  }[kind] as MessageKey
}

function stateKey(state: SystemRow['state']): MessageKey {
  const key = state.replace(/-([a-z])/g, (_, letter: string) => letter.toUpperCase())
  return `state${key.charAt(0).toUpperCase()}${key.slice(1)}` as MessageKey
}

function stateTone(state: SystemRow['state']) {
  if (['running', 'installed', 'available', 'normal'].includes(state)) return 'positive'
  if (['starting', 'restarting', 'not-running', 'elevated', 'low'].includes(state)) return 'warning'
  if (['degraded', 'error', 'high', 'critical'].includes(state)) return 'negative'
  return 'neutral'
}

function formatBytes(value: number | null): string {
  if (value === null) return '—'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let amount = value
  let unitIndex = 0
  while (amount >= 1024 && unitIndex < units.length - 1) {
    amount /= 1024
    unitIndex += 1
  }
  return `${amount.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`
}

function interpolate(template: string, values: Record<string, string>) {
  return Object.entries(values).reduce((result, [key, value]) => result.replace(`{${key}}`, value), template)
}
