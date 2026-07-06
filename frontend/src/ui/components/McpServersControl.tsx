import { Plus, Trash2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { KeyValueControl } from './KeyValueControl'

type McpTransport = 'streamable-http' | 'sse' | 'stdio'
type McpDeployment = 'compose-sidecar' | 'external-service' | 'stdio'

export interface McpServerLabels {
  addServer: string
  addItem: string
  args: string
  composeSidecar: string
  composeYaml: string
  command: string
  deployment: string
  description: string
  enabled: string
  endpointPath: string
  env: string
  externalService: string
  generatedUrl: string
  name: string
  port: string
  serviceName: string
  stdio: string
  transport: string
  url: string
}

interface McpServer {
  args?: string[]
  command?: string
  composeYaml?: string
  deployment: McpDeployment
  enabled: boolean
  endpointPath?: string
  env?: string
  name: string
  port?: string
  serviceName?: string
  transport: McpTransport
  url?: string
}

interface McpServersControlProps {
  labels: McpServerLabels
  value: string
  onChange: (value: string) => void
}

const emptyServer: McpServer = {
  composeYaml: composeTemplate('mcp-server', 'mcp-server:latest', '8000'),
  deployment: 'compose-sidecar',
  endpointPath: '/mcp',
  enabled: true,
  name: 'mcp-server',
  port: '8000',
  serviceName: 'mcp-server',
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
                {labels.deployment}
                <select
                  value={server.deployment}
                  onChange={(event) => updateServer(index, { deployment: event.target.value as McpDeployment })}
                >
                  <option value="compose-sidecar">{labels.composeSidecar}</option>
                  <option value="external-service">{labels.externalService}</option>
                  <option value="stdio">{labels.stdio}</option>
                </select>
              </label>
              <label>
                {labels.transport}
                <select
                  value={server.transport}
                  onChange={(event) => updateServer(index, { transport: event.target.value as McpTransport })}
                  disabled={server.deployment === 'stdio'}
                >
                  <option value="streamable-http">streamable-http</option>
                  <option value="sse">sse</option>
                  {server.deployment === 'stdio' && <option value="stdio">stdio</option>}
                </select>
              </label>
              {server.deployment === 'compose-sidecar' && (
                <>
                  <label>
                    {labels.serviceName}
                    <input value={server.serviceName ?? ''} onChange={(event) => updateServer(index, { serviceName: event.target.value })} />
                  </label>
                  <label>
                    {labels.port}
                    <input inputMode="numeric" value={server.port ?? ''} onChange={(event) => updateServer(index, { port: event.target.value })} />
                  </label>
                  <label>
                    {labels.endpointPath}
                    <input value={server.endpointPath ?? ''} onChange={(event) => updateServer(index, { endpointPath: event.target.value })} />
                  </label>
                  <label>
                    {labels.generatedUrl}
                    <input readOnly value={composeUrl(server)} />
                  </label>
                  <label className="field-wide">
                    {labels.composeYaml}
                    <textarea value={server.composeYaml ?? ''} onChange={(event) => updateServer(index, { composeYaml: event.target.value })} />
                  </label>
                </>
              )}
              {server.deployment === 'external-service' && (
                <label className="field-wide">
                  {labels.url}
                  <input value={server.url ?? ''} onChange={(event) => updateServer(index, { url: event.target.value })} />
                </label>
              )}
              {server.deployment === 'stdio' && (
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
      deployment: 'stdio',
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
  if (server.deployment === 'stdio' || (!server.deployment && server.transport === 'stdio')) {
    return {
      args: (server.args ?? []).map((arg) => arg.trim()).filter(Boolean),
      command: server.command ?? '',
      deployment: 'stdio',
      enabled: server.enabled,
      env: server.env ?? 'none',
      name: server.name,
      transport: 'stdio',
    }
  }
  if (server.deployment === 'external-service') {
    return {
      deployment: 'external-service',
      enabled: server.enabled,
      name: server.name,
      transport: normalizeNetworkTransport(server.transport),
      url: server.url ?? '',
    }
  }
  const serviceName = server.serviceName ?? hostFromUrl(server.url) ?? server.name
  const port = server.port ?? portFromUrl(server.url) ?? '8000'
  const endpointPath = normalizePath(server.endpointPath ?? pathFromUrl(server.url))
  return {
    composeYaml: server.composeYaml ?? composeTemplate(serviceName, `${serviceName}:latest`, port),
    deployment: 'compose-sidecar',
    endpointPath,
    enabled: server.enabled,
    name: server.name,
    port,
    serviceName,
    transport: normalizeNetworkTransport(server.transport),
    url: `http://${serviceName}:${port}${endpointPath}`,
  }
}

function normalizeNetworkTransport(transport: McpTransport) {
  return transport === 'stdio' ? 'streamable-http' : transport
}

function uniqueServerName(servers: McpServer[]) {
  const existing = new Set(servers.map((server) => server.name))
  let index = servers.length + 1
  while (existing.has(`mcp-server-${index}`)) index += 1
  return `mcp-server-${index}`
}

function composeUrl(server: McpServer) {
  const serviceName = server.serviceName?.trim() || server.name || 'mcp-server'
  const port = server.port?.trim() || '8000'
  const endpointPath = normalizePath(server.endpointPath)
  return `http://${serviceName}:${port}${endpointPath}`
}

function composeTemplate(serviceName: string, image: string, port: string) {
  return [
    'services:',
    `  ${serviceName}:`,
    `    image: ${image}`,
    '    expose:',
    `      - "${port}"`,
  ].join('\n')
}

function hostFromUrl(value?: string) {
  try {
    return value ? new URL(value).hostname : undefined
  } catch {
    return undefined
  }
}

function portFromUrl(value?: string) {
  try {
    return value ? new URL(value).port || undefined : undefined
  } catch {
    return undefined
  }
}

function pathFromUrl(value?: string) {
  try {
    return value ? new URL(value).pathname : undefined
  } catch {
    return undefined
  }
}

function normalizePath(value?: string) {
  const path = value?.trim() || '/mcp'
  return path.startsWith('/') ? path : `/${path}`
}
