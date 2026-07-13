import type { McpServerLabels } from './McpServersControl'
import { Metric } from './Metric'

interface ReadonlyDisplayProps {
  emptyLabel: string
  label: string
  value?: string
}

interface ReadonlyMcpServersProps {
  emptyLabel: string
  label: string
  labels: McpServerLabels
  value?: string
}

export function ReadonlyKeyValueList({ emptyLabel, label, value }: ReadonlyDisplayProps) {
  const rows = parseKeyValues(value)
  if (!rows.length) return <Metric label={label} value={emptyLabel} />
  return (
    <div className="readonly-display field-wide" aria-label={label}>
      <span className="readonly-display-title">{label}</span>
      <div className="readonly-display-list">
        {rows.map((row) => (
          <div className="readonly-display-row" key={row.key}>
            <span>{row.key}</span>
            <strong>{row.value || emptyLabel}</strong>
          </div>
        ))}
      </div>
    </div>
  )
}

export function ReadonlyStringList({ emptyLabel, label, value }: ReadonlyDisplayProps) {
  const rows = parseStringList(value)
  if (!rows.length) return <Metric label={label} value={emptyLabel} />
  return (
    <div className="readonly-display field-wide" aria-label={label}>
      <span className="readonly-display-title">{label}</span>
      <div className="readonly-display-list">
        {rows.map((row) => (
          <div className="readonly-display-row" key={row}>
            <strong>{row}</strong>
          </div>
        ))}
      </div>
    </div>
  )
}

export function ReadonlyMcpServers({ emptyLabel, label, labels, value }: ReadonlyMcpServersProps) {
  const servers = parseMcpServers(value)
  if (!servers.length) return <Metric label={label} value={emptyLabel} />
  return (
    <div className="readonly-display field-wide" aria-label={label}>
      <span className="readonly-display-title">{label}</span>
      <div className="readonly-display-list">
        {servers.map((server, index) => (
          <div className="readonly-display-row" key={`${server.name}-${index}`}>
            <span>{server.transport}</span>
            <strong>{server.name}</strong>
            {server.endpoint && <small>{server.endpoint}</small>}
            {server.args.length > 0 && <small>{`${labels.args}: ${server.args.join(' ')}`}</small>}
          </div>
        ))}
      </div>
    </div>
  )
}

function parseKeyValues(value?: string) {
  if (!isConfigured(value)) return []
  const raw = value ?? ''
  return raw.split('\n').flatMap((line) => {
    const [key, ...rest] = line.split('=')
    const cleanKey = key.trim()
    if (!cleanKey) return []
    return [{ key: cleanKey, value: rest.join('=').trim() }]
  })
}

function parseStringList(value?: string) {
  if (!isConfigured(value)) return []
  const raw = value ?? ''
  return raw.split(/\n|,/).map((item) => item.trim()).filter(Boolean)
}

function parseMcpServers(value?: string) {
  if (!isConfigured(value)) return []
  try {
    const decoded = JSON.parse(value ?? '')
    if (!Array.isArray(decoded)) return []
    return decoded.flatMap((item, index) => {
      if (!item || typeof item !== 'object') return []
      const server = item as Record<string, unknown>
      const name = typeof server.name === 'string' && server.name.trim() ? server.name.trim() : `mcp-server-${index + 1}`
      const transport = typeof server.transport === 'string' ? server.transport : ''
      const command = typeof server.command === 'string' ? server.command : ''
      const url = typeof server.url === 'string' ? server.url : ''
      const args = Array.isArray(server.args) ? server.args.filter((arg): arg is string => typeof arg === 'string') : []
      if (!transport) return []
      return [{ args, endpoint: command || url, name, transport }]
    })
  } catch {
    return []
  }
}

function isConfigured(value?: string) {
  return Boolean(value && value !== 'none' && value !== '-')
}
