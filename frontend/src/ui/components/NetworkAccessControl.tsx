import { EditableStringList } from './EditableStringList'

interface NetworkAccessControlProps {
  enabledLabel: string
  hostsLabel: string
  addLabel: string
  deleteLabel: string
  value: string
  onChange: (value: string) => void
}

export function NetworkAccessControl({
  enabledLabel,
  hostsLabel,
  addLabel,
  deleteLabel,
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
        <EditableStringList
          addLabel={addLabel}
          className="network-host-list"
          deleteLabel={deleteLabel}
          itemAriaLabel={(_, index) => `${hostsLabel} ${index + 1}`}
          label={hostsLabel}
          values={hosts}
          onChange={commitHosts}
        />
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
