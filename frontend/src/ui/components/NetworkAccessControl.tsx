interface NetworkAccessControlProps {
  enabledLabel: string
  hostsLabel: string
  value: string
  onChange: (value: string) => void
}

export function NetworkAccessControl({ enabledLabel, hostsLabel, value, onChange }: NetworkAccessControlProps) {
  const normalizedValue = value.trim() || '*'
  const enabled = normalizedValue !== 'none'
  const hostsValue = enabled ? normalizedValue : '*'

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
        <label className="field-wide">
          {hostsLabel}
          <textarea value={hostsValue} onChange={(event) => onChange(event.target.value.trim() || '*')} />
        </label>
      )}
    </div>
  )
}
