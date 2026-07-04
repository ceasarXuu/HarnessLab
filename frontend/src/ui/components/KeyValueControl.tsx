import { useState } from 'react'
import { Plus, Trash2 } from 'lucide-react'

interface KeyValueControlProps {
  label: string
  value: string
  onChange: (value: string) => void
  compact?: boolean
}

interface KeyValueRow {
  key: string
  value: string
}

export function KeyValueControl({ label, value, onChange, compact = false }: KeyValueControlProps) {
  const [rows, setRows] = useState(() => parseRows(value))
  const commit = (nextRows: KeyValueRow[]) => {
    setRows(nextRows.length ? nextRows : [{ key: '', value: '' }])
    onChange(formatRows(nextRows))
  }
  const className = compact ? 'key-value-control compact-key-value field-wide' : 'key-value-control field-wide'

  return (
    <div className={className}>
      <div className="key-value-header">
        <span>{label}</span>
        <button
          aria-label={compact ? `Add ${label}` : undefined}
          className="secondary-button compact-action"
          type="button"
          onClick={() => commit([...rows, { key: '', value: '' }])}
        >
          <Plus aria-hidden="true" />
          Add
        </button>
      </div>
      <div className="key-value-list">
        {rows.map((row, index) => (
          <div className="key-value-row" key={index}>
            <label>
              <span className={compact ? 'visually-hidden' : undefined}>Key</span>
              <input
                aria-label={compact ? 'Env key' : undefined}
                value={row.key}
                onChange={(event) => commit(rows.map((item, rowIndex) => rowIndex === index ? { ...item, key: event.target.value } : item))}
              />
            </label>
            <label>
              <span className={compact ? 'visually-hidden' : undefined}>Value</span>
              <input
                aria-label={compact ? 'Env value' : undefined}
                value={row.value}
                onChange={(event) => commit(rows.map((item, rowIndex) => rowIndex === index ? { ...item, value: event.target.value } : item))}
              />
            </label>
            <button
              aria-label={`Delete env ${row.key || index + 1}`}
              className={compact ? 'icon-button' : 'secondary-button compact-action'}
              type="button"
              onClick={() => commit(rows.filter((_, rowIndex) => rowIndex !== index))}
            >
              <Trash2 aria-hidden="true" />
              {!compact && 'Delete'}
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}

function parseRows(value: string): KeyValueRow[] {
  if (!value || value === 'none') return [{ key: '', value: '' }]
  return value.split('\n').map((line) => {
    const [key, ...rest] = line.split('=')
    return { key: key.trim(), value: rest.join('=').trim() }
  })
}

function formatRows(rows: KeyValueRow[]) {
  const formatted = rows
    .filter((row) => row.key.trim() || row.value.trim())
    .map((row) => `${row.key.trim()}=${row.value.trim()}`)
  return formatted.length ? formatted.join('\n') : 'none'
}
