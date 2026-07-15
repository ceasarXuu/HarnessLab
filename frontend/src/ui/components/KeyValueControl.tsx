import { useEffect, useState } from 'react'
import { Plus, Trash2 } from 'lucide-react'
import { CustomSelect } from './CustomSelect'

interface KeyValueControlProps {
  label: string
  value: string
  onChange: (value: string) => void
  className?: string
  compact?: boolean
  readOnly?: boolean
  allowInherited?: boolean
  keyOptions?: string[]
  labels: {
    add: string
    customKey?: string
    delete: string
    key: string
    searchKeys?: string
    value: string
    source?: string
    inherited?: string
    literal?: string
  }
}

interface KeyValueRow {
  customKey: boolean
  key: string
  value: string
  inherited: boolean
}

const customKeyValue = '__ornnlab_custom_environment_variable__'
const noKeyOptions: string[] = []

export function KeyValueControl({ label, value, onChange, className, compact = false, readOnly = false, allowInherited = false, keyOptions = noKeyOptions, labels }: KeyValueControlProps) {
  const keyOptionsSignature = keyOptions.join('\n')
  const [rows, setRows] = useState(() => parseRows(value, keyOptions))
  useEffect(() => {
    setRows(parseRows(value, keyOptionsSignature ? keyOptionsSignature.split('\n') : noKeyOptions))
  }, [keyOptionsSignature, value])

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
            onClick={() => setRows([
            ...rows.filter((row) => row.key.trim() || row.value.trim()),
              { customKey: keyOptions.length === 0, key: '', value: '', inherited: false },
            ])}
          >
            <Plus aria-hidden="true" />
            {labels.add}
          </button>
        )}
      </div>
      <div className="key-value-list">
        {rows.map((row, index) => (
          <div
            className={`key-value-row${allowInherited ? ' key-value-row--with-source' : ''}${allowInherited && row.inherited ? ' key-value-row--inherited' : ''}`}
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
              {keyOptions.length > 0 && !row.customKey ? (
                <CustomSelect
                  ariaLabel={labels.key}
                  disabled={readOnly}
                  options={[
                    ...availableKeyOptions(keyOptions, rows, index).map((key) => ({ label: key, value: key })),
                    { label: labels.customKey ?? labels.key, value: customKeyValue },
                  ]}
                  placeholder={labels.key}
                  searchAriaLabel={labels.searchKeys}
                  searchPlaceholder={labels.searchKeys}
                  value={row.key}
                  onChange={(nextKey) => {
                    if (nextKey === customKeyValue) {
                      setRows(rows.map((item, rowIndex) => rowIndex === index
                        ? { ...item, customKey: true, key: '' }
                        : item))
                      return
                    }
                    commit(rows.map((item, rowIndex) => rowIndex === index
                      ? { ...item, customKey: false, key: nextKey }
                      : item))
                  }}
                />
              ) : (
                <input
                  aria-label={compact ? labels.key : undefined}
                  autoFocus={!readOnly && !row.key && !row.value}
                  readOnly={readOnly}
                  value={row.key}
                  onChange={(event) => commit(rows.map((item, rowIndex) => rowIndex === index
                    ? { ...item, key: event.target.value }
                    : item))}
                />
              )}
            </label>
            {allowInherited && (
              <label>
                <span className={compact ? 'visually-hidden' : undefined}>{labels.source}</span>
                <CustomSelect
                  ariaLabel={labels.source ?? ''}
                  disabled={readOnly}
                  options={[
                    { label: labels.literal ?? '', value: 'literal' },
                    { label: labels.inherited ?? '', value: 'inherited' },
                  ]}
                  value={row.inherited ? 'inherited' : 'literal'}
                  onChange={(nextValue) => commit(rows.map((item, rowIndex) => rowIndex === index
                    ? { ...item, inherited: nextValue === 'inherited' }
                    : item))}
                />
              </label>
            )}
            {!row.inherited && (
              <label>
                <span className={compact ? 'visually-hidden' : undefined}>{labels.value}</span>
                <input
                  aria-label={compact ? labels.value : undefined}
                  readOnly={readOnly}
                  value={row.value}
                  onChange={(event) => commit(rows.map((item, rowIndex) => rowIndex === index ? { ...item, value: event.target.value } : item))}
                />
              </label>
            )}
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

function parseRows(value: string, keyOptions: string[]): KeyValueRow[] {
  if (!value || value === 'none') return []
  return value.split('\n').map((line) => {
    const [key, ...rest] = line.split('=')
    return {
      customKey: keyOptions.length > 0 && !keyOptions.includes(key.trim()),
      key: key.trim(),
      value: rest.join('=').trim(),
      inherited: !line.includes('='),
    }
  })
}

function availableKeyOptions(keyOptions: string[], rows: KeyValueRow[], currentIndex: number) {
  const selected = new Set(
    rows.filter((_, index) => index !== currentIndex).map((row) => row.key).filter(Boolean),
  )
  return keyOptions.filter((key) => !selected.has(key))
}

function formatRows(rows: KeyValueRow[]) {
  const formatted = rows
    .filter((row) => row.key.trim() || row.value.trim())
    .map((row) => row.inherited ? row.key.trim() : `${row.key.trim()}=${row.value.trim()}`)
  return formatted.length ? formatted.join('\n') : 'none'
}
