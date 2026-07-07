import { Plus, Trash2 } from 'lucide-react'

interface NetworkAccessControlProps {
  enabledLabel: string
  hostsLabel: string
  addLabel?: string
  deleteLabel?: string
  value: string
  onChange: (value: string) => void
}

export function NetworkAccessControl({
  enabledLabel,
  hostsLabel,
  addLabel = 'Add',
  deleteLabel = 'Delete',
  value,
  onChange,
}: NetworkAccessControlProps) {
  const normalizedValue = value.trim() || '*'
  const enabled = normalizedValue !== 'none'
  const hosts = enabled ? parseHosts(normalizedValue) : ['*']
  const commitHosts = (nextHosts: string[]) => onChange(formatHosts(nextHosts))

  return (
    <div className="network-access-control field-wide">
      <label className="switch-control network-access-toggle">
        <span>{enabledLabel}</span>
        <input
          aria-label={enabledLabel}
          checked={enabled}
          type="checkbox"
          onChange={(event) => onChange(event.target.checked ? '*' : 'none')}
        />
      </label>
      {enabled && (
        <div className="network-host-list">
          <div className="network-host-list-header">
            <span>{hostsLabel}</span>
            <button className="secondary-button compact-action" type="button" onClick={() => commitHosts([...hosts, ''])}>
              <Plus aria-hidden="true" />
              {addLabel}
            </button>
          </div>
          <div className="network-host-list-rows">
            {hosts.map((host, index) => (
              <div className="network-host-list-row" key={index}>
                <input
                  aria-label={`${hostsLabel} ${index + 1}`}
                  value={host}
                  onChange={(event) => commitHosts(hosts.map((item, itemIndex) => (itemIndex === index ? event.target.value : item)))}
                />
                <button
                  aria-label={`${deleteLabel} ${hostsLabel} ${index + 1}`}
                  className="icon-button"
                  type="button"
                  onClick={() => commitHosts(hosts.filter((_, itemIndex) => itemIndex !== index))}
                >
                  <Trash2 aria-hidden="true" />
                </button>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}

function parseHosts(value: string) {
  const hosts = value.split(',').map((host) => host.trim()).filter(Boolean)
  return hosts.length ? hosts : ['*']
}

function formatHosts(hosts: string[]) {
  const cleanHosts = hosts.map((host) => host.trim()).filter(Boolean)
  return cleanHosts.length ? cleanHosts.join(', ') : '*'
}
