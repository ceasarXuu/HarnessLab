import { useEffect, useState } from 'react'
import { Plus, Trash2 } from 'lucide-react'

interface KeyValueControlProps {
  label: string
  value: string
  onChange: (value: string) => void
  className?: string
  compact?: boolean
  readOnly?: boolean
  allowInherited?: boolean
  labels: {
    add: string
    delete: string
    key: string
    value: string
    source?: string
    inherited?: string
    literal?: string
  }
}

interface KeyValueRow {
  key: string
  value: string
  inherited: boolean
}

export function KeyValueControl({ label, value, onChange, className, compact = false, readOnly = false, allowInherited = false, labels }: KeyValueControlProps) {
  const [rows, setRows] = useState(() => parseRows(value))
  useEffect(() => {
    setRows(parseRows(value))
  }, [value])

  const commit = (nextRows: KeyValueRow[]) => {
    setRows(nextRows)
    onChange(formatRows(nextRows))
  }
  const rootClassName = compact ? 'key-value-control compact-key-value field-wide' : `key-value-control field-wide ${className ?? ''}`

  return (
    <div className={rootClassName}>
      <div className="key-value-header">
        <span>{label}</span>
        {!readOnly && (
          <button
            aria-label={compact ? `${labels.add} ${label}` : undefined}
            className="secondary-button compact-action"
            type="button"
            onClick={() => setRows([...rows.filter((row) => row.key.trim() || row.value.trim()), { key: '', value: '', inherited: allowInherited }])}
          >
            <Plus aria-hidden="true" />
            {labels.add}
          </button>
        )}
      </div>
      <div className="key-value-list">
        {rows.map((row, index) => (
          <div
            className={`key-value-row${allowInherited ? ' key-value-row--with-source' : ''}`}
            key={index}
            onBlur={(event) => {
              if (event.currentTarget.contains(event.relatedTarget)) return
              if (!row.key.trim() && !row.value.trim()) {
                setRows(rows.filter((_, rowIndex) => rowIndex !== index))
              }
            }}
          >
            <label>
              <span className={compact ? 'visually-hidden' : undefined}>{labels.key}</span>
              <input
                aria-label={compact ? labels.key : undefined}
                autoFocus={!readOnly && !row.key && !row.value}
                readOnly={readOnly}
                value={row.key}
                onChange={(event) => commit(rows.map((item, rowIndex) => rowIndex === index ? { ...item, key: event.target.value } : item))}
              />
            </label>
            {allowInherited && (
              <label>
                <span className={compact ? 'visually-hidden' : undefined}>{labels.source}</span>
                <select
                  aria-label={compact ? labels.source : undefined}
                  disabled={readOnly}
                  value={row.inherited ? 'inherited' : 'literal'}
                  onChange={(event) => commit(rows.map((item, rowIndex) => rowIndex === index
                    ? { ...item, inherited: event.target.value === 'inherited' }
                    : item))}
                >
                  <option value="inherited">{labels.inherited}</option>
                  <option value="literal">{labels.literal}</option>
                </select>
              </label>
            )}
            <label>
              <span className={compact ? 'visually-hidden' : undefined}>{labels.value}</span>
              <input
                aria-label={compact ? labels.value : undefined}
                readOnly={readOnly}
                disabled={row.inherited}
                value={row.value}
                onChange={(event) => commit(rows.map((item, rowIndex) => rowIndex === index ? { ...item, value: event.target.value } : item))}
              />
            </label>
            {!readOnly && (
              <button
                aria-label={`${labels.delete} ${label} ${row.key || index + 1}`}
                className={compact ? 'icon-button' : 'secondary-button compact-action'}
                type="button"
                onClick={() => commit(rows.filter((_, rowIndex) => rowIndex !== index))}
              >
                <Trash2 aria-hidden="true" />
                {!compact && labels.delete}
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}

function parseRows(value: string): KeyValueRow[] {
  if (!value || value === 'none') return []
  return value.split('\n').map((line) => {
    const [key, ...rest] = line.split('=')
    return { key: key.trim(), value: rest.join('=').trim(), inherited: !line.includes('=') }
  })
}

function formatRows(rows: KeyValueRow[]) {
  const formatted = rows
    .filter((row) => row.key.trim() || row.value.trim())
    .map((row) => row.inherited ? row.key.trim() : `${row.key.trim()}=${row.value.trim()}`)
  return formatted.length ? formatted.join('\n') : 'none'
}
