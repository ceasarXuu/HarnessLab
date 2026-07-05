import { Plus, Trash2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { KeyValueControl } from './KeyValueControl'

type McpTransport = 'streamable-http' | 'sse' | 'stdio'

export interface McpServerLabels {
  addServer: string
  addItem: string
  args: string
  command: string
  description: string
  enabled: string
  env: string
  name: string
  transport: string
  url: string
}

interface McpServer {
  args?: string[]
  command?: string
  enabled: boolean
  env?: string
  name: string
  transport: McpTransport
  url?: string
}

interface McpServersControlProps {
  labels: McpServerLabels
  value: string
  onChange: (value: string) => void
}

const emptyServer: McpServer = {
  enabled: true,
  name: 'mcp-server',
  transport: 'streamable-http',
  url: 'http://mcp-server:8000/mcp',
}

export function McpServersControl({ labels, value, onChange }: McpServersControlProps) {
  const [servers, setServers] = useState(() => parseMcpServers(value))

  useEffect(() => {
    setServers(parseMcpServers(value))
  }, [value])

  const commit = (nextServers: McpServer[]) => {
    setServers(nextServers)
    onChange(formatMcpServers(nextServers))
  }

  const updateServer = (index: number, patch: Partial<McpServer>) => {
    commit(servers.map((server, rowIndex) => rowIndex === index ? normalizeServer({ ...server, ...patch }) : server))
  }

  return (
    <div className="mcp-server-control">
      <p className="field-hint">{labels.description}</p>
      <div className="rule-list-header">
        <span>{labels.name}</span>
        <button
          className="secondary-button compact-action"
          type="button"
          onClick={() => commit([...servers, { ...emptyServer, name: uniqueServerName(servers) }])}
        >
          <Plus aria-hidden="true" />
          {labels.addServer}
        </button>
      </div>
      <div className="mcp-server-list">
        {servers.map((server, index) => (
          <section className="mcp-server-card" key={`${server.name}-${index}`}>
            <div className="mcp-server-header">
              <label className="switch-control">
                <span>{labels.enabled}</span>
                <input
                  aria-label={`${labels.enabled} ${server.name}`}
                  checked={server.enabled}
                  type="checkbox"
                  onChange={(event) => updateServer(index, { enabled: event.target.checked })}
                />
              </label>
              <button
                aria-label={`Delete MCP server ${server.name || index + 1}`}
                className="icon-button"
                type="button"
                onClick={() => commit(servers.filter((_, rowIndex) => rowIndex !== index))}
              >
                <Trash2 aria-hidden="true" />
              </button>
            </div>
            <div className="agent-form-grid">
              <label>
                {labels.name}
                <input value={server.name} onChange={(event) => updateServer(index, { name: event.target.value })} />
              </label>
              <label>
                {labels.transport}
                <select
                  value={server.transport}
                  onChange={(event) => updateServer(index, { transport: event.target.value as McpTransport })}
                >
                  <option value="streamable-http">streamable-http</option>
                  <option value="sse">sse</option>
                  <option value="stdio">stdio</option>
                </select>
              </label>
          {server.transport === 'stdio' ? (
                <>
                  <label className="field-wide">
                    {labels.command}
                    <input value={server.command ?? ''} onChange={(event) => updateServer(index, { command: event.target.value })} />
                  </label>
                  <ArgumentList
                    addLabel={labels.addItem}
                    label={labels.args}
                    value={server.args ?? ['']}
                    onChange={(args) => updateServer(index, { args })}
                  />
                  <div className="field-wide">
                    <KeyValueControl
                      compact
                      label={labels.env}
                      value={server.env ?? 'none'}
                      onChange={(env) => updateServer(index, { env })}
                    />
                  </div>
                </>
              ) : (
                <label className="field-wide">
                  {labels.url}
                  <input value={server.url ?? ''} onChange={(event) => updateServer(index, { url: event.target.value })} />
                </label>
              )}
            </div>
          </section>
        ))}
      </div>
    </div>
  )
}

function ArgumentList({
  addLabel,
  label,
  value,
  onChange,
}: {
  addLabel: string
  label: string
  value: string[]
  onChange: (value: string[]) => void
}) {
  const args = value.length ? value : ['']
  const commit = (nextArgs: string[]) => onChange(nextArgs.length ? nextArgs : [''])

  return (
    <div className="argument-list-control field-wide">
      <div className="rule-list-header">
        <span>{label}</span>
        <button className="secondary-button compact-action" type="button" onClick={() => commit([...args, ''])}>
          <Plus aria-hidden="true" />
          {addLabel}
        </button>
      </div>
      <div className="rule-list-rows">
        {args.map((arg, index) => (
          <div className="rule-list-row" key={index}>
            <input
              aria-label={label}
              value={arg}
              onChange={(event) => commit(args.map((item, rowIndex) => rowIndex === index ? event.target.value : item))}
            />
            <button
              aria-label={`Delete ${label} ${index + 1}`}
              className="icon-button"
              type="button"
              onClick={() => commit(args.filter((_, rowIndex) => rowIndex !== index))}
            >
              <Trash2 aria-hidden="true" />
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}

function parseMcpServers(value: string): McpServer[] {
  if (!value || value === 'none') return []
  try {
    const parsed = JSON.parse(value) as McpServer[]
    return Array.isArray(parsed) ? parsed.map(normalizeServer) : []
  } catch {
    return [{
      enabled: true,
      name: 'legacy-mcp-config',
      transport: 'stdio',
      command: value,
      args: [],
      env: 'none',
    }]
  }
}

function formatMcpServers(servers: McpServer[]) {
  const cleaned = servers
    .map(normalizeServer)
    .filter((server) => server.name.trim())
  return cleaned.length ? JSON.stringify(cleaned, null, 2) : 'none'
}

function normalizeServer(server: McpServer): McpServer {
  if (server.transport === 'stdio') {
    return {
      args: (server.args ?? []).map((arg) => arg.trim()).filter(Boolean),
      command: server.command ?? '',
      enabled: server.enabled,
      env: server.env ?? 'none',
      name: server.name,
      transport: 'stdio',
    }
  }
  return {
    enabled: server.enabled,
    name: server.name,
    transport: server.transport,
    url: server.url ?? '',
  }
}

function uniqueServerName(servers: McpServer[]) {
  const existing = new Set(servers.map((server) => server.name))
  let index = servers.length + 1
  while (existing.has(`mcp-server-${index}`)) index += 1
  return `mcp-server-${index}`
}
