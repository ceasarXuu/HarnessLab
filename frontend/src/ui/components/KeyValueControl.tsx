import { Plus, Trash2 } from 'lucide-react'

interface KeyValueControlProps {
  label: string
  value: string
  onChange: (value: string) => void
}

interface KeyValueRow {
  key: string
  value: string
}

export function KeyValueControl({ label, value, onChange }: KeyValueControlProps) {
  const rows = parseRows(value)
  const commit = (nextRows: KeyValueRow[]) => onChange(formatRows(nextRows))

  return (
    <div className="key-value-control field-wide">
      <div className="key-value-header">
        <span>{label}</span>
        <button className="secondary-button compact-action" type="button" onClick={() => commit([...rows, { key: '', value: '' }])}>
          <Plus aria-hidden="true" />
          Add
        </button>
      </div>
      <div className="key-value-list">
        {rows.map((row, index) => (
          <div className="key-value-row" key={index}>
            <label>
              Key
              <input
                value={row.key}
                onChange={(event) => commit(rows.map((item, rowIndex) => rowIndex === index ? { ...item, key: event.target.value } : item))}
              />
            </label>
            <label>
              Value
              <input
                value={row.value}
                onChange={(event) => commit(rows.map((item, rowIndex) => rowIndex === index ? { ...item, value: event.target.value } : item))}
              />
            </label>
            <button className="secondary-button compact-action" type="button" onClick={() => commit(rows.filter((_, rowIndex) => rowIndex !== index))}>
              <Trash2 aria-hidden="true" />
              Delete
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
