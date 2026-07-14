import { Plus, Trash2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { EditableStringList } from './EditableStringList'

type McpTransport = 'streamable-http' | 'sse' | 'stdio'

export interface McpServerLabels {
  addServer: string
  args: string
  command: string
  deleteItem: string
  deleteServer: string
  name: string
  transport: string
  url: string
}

interface McpServer {
  args: string[]
  command?: string
  name: string
  transport: McpTransport
  url?: string
}

interface McpServersControlProps {
  labels: McpServerLabels
  readOnly?: boolean
  value: string
  onChange: (value: string) => void
}

const emptyServer: McpServer = { args: [], command: '', name: 'mcp-server', transport: 'stdio', url: '' }

export function McpServersControl({ labels, readOnly = false, value, onChange }: McpServersControlProps) {
  const [servers, setServers] = useState(() => parseMcpServers(value))
  useEffect(() => setServers(parseMcpServers(value)), [value])
  const commit = (nextServers: McpServer[]) => {
    setServers(nextServers)
    onChange(formatMcpServers(nextServers))
  }
  const update = (index: number, patch: Partial<McpServer>) => {
    commit(servers.map((server, current) => current === index ? { ...server, ...patch } : server))
  }

  return (
    <div className="mcp-server-control">
      <div className="rule-list-header">
        <span>{labels.name}</span>
        {!readOnly && (
          <button className="secondary-button compact-action" type="button" onClick={() => commit([...servers, { ...emptyServer, name: uniqueServerName(servers) }])}>
            <Plus aria-hidden="true" />
            {labels.addServer}
          </button>
        )}
      </div>
      <div className="mcp-server-list">
        {servers.map((server, index) => (
          <section className="mcp-server-card" key={`${server.name}-${index}`}>
            <div className="mcp-server-header">
              <strong>{server.name || labels.name}</strong>
              {!readOnly && (
                <button aria-label={`${labels.deleteServer}: ${server.name}`} className="icon-button" type="button" onClick={() => commit(servers.filter((_, current) => current !== index))}><Trash2 aria-hidden="true" /></button>
              )}
            </div>
            <div className="agent-form-grid">
              <label>{labels.name}<input readOnly={readOnly} value={server.name} onChange={(event) => update(index, { name: event.target.value })} /></label>
              <label>
                {labels.transport}
                <select disabled={readOnly} value={server.transport} onChange={(event) => update(index, { transport: event.target.value as McpTransport })}>
                  <option value="stdio">stdio</option>
                  <option value="sse">SSE</option>
                  <option value="streamable-http">Streamable HTTP</option>
                </select>
              </label>
              {server.transport === 'stdio' ? (
                <label className="field-wide">{labels.command}<input readOnly={readOnly} value={server.command ?? ''} onChange={(event) => update(index, { command: event.target.value, url: undefined })} /></label>
              ) : (
                <label className="field-wide">{labels.url}<input readOnly={readOnly} type="url" value={server.url ?? ''} onChange={(event) => update(index, { command: undefined, url: event.target.value })} /></label>
              )}
              <EditableStringList
                addLabel={labels.addServer}
                className="field-wide"
                deleteLabel={labels.deleteItem}
                itemAriaLabel={() => labels.args}
                label={labels.args}
                readOnly={readOnly}
                values={server.args}
                onChange={(args) => update(index, { args: args.map((item) => item.trim()).filter(Boolean) })}
              />
            </div>
          </section>
        ))}
      </div>
    </div>
  )
}

function parseMcpServers(value: string): McpServer[] {
  if (!value || value === 'none') return []
  try {
    const decoded = JSON.parse(value)
    if (!Array.isArray(decoded)) return []
    return decoded.flatMap((item): McpServer[] => {
      if (!item || typeof item !== 'object') return []
      const value = item as Record<string, unknown>
      const transport = value.transport
      if (transport !== 'stdio' && transport !== 'sse' && transport !== 'streamable-http') return []
      return [{
        args: Array.isArray(value.args) ? value.args.filter((arg): arg is string => typeof arg === 'string') : [],
        command: typeof value.command === 'string' ? value.command : undefined,
        name: typeof value.name === 'string' ? value.name : '',
        transport,
        url: typeof value.url === 'string' ? value.url : undefined,
      }]
    })
  } catch {
    return []
  }
}

function formatMcpServers(servers: McpServer[]) {
  return servers.length ? JSON.stringify(servers.map((server) => ({
    args: server.args,
    command: server.transport === 'stdio' ? server.command?.trim() || undefined : undefined,
    name: server.name.trim(),
    transport: server.transport,
    url: server.transport === 'stdio' ? undefined : server.url?.trim() || undefined,
  })).filter((server) => server.name)) : 'none'
}

function uniqueServerName(servers: McpServer[]) {
  const names = new Set(servers.map((server) => server.name))
  let index = 1
  let name = `mcp-server-${index}`
  while (names.has(name)) {
    index += 1
    name = `mcp-server-${index}`
  }
  return name
}
