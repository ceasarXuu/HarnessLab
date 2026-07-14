import type { ReactNode } from 'react'
import { Plus, Trash2 } from 'lucide-react'

interface EditableStringListProps {
  addLabel: string
  className?: string
  deleteLabel: string
  extraActions?: ReactNode
  itemAriaLabel?: (value: string, index: number) => string
  label: string
  onChange: (values: string[]) => void
  placeholder?: string
  readOnly?: boolean
  values: string[]
}

export function EditableStringList({
  addLabel,
  className,
  deleteLabel,
  extraActions,
  itemAriaLabel,
  label,
  onChange,
  placeholder,
  readOnly = false,
  values,
}: EditableStringListProps) {
  const rows = values.length ? values : ['']
  const setRow = (index: number, value: string) => {
    onChange(rows.map((item, rowIndex) => (rowIndex === index ? value : item)))
  }
  const removeRow = (index: number) => {
    const nextValues = rows.filter((_, rowIndex) => rowIndex !== index)
    onChange(nextValues.length ? nextValues : [''])
  }

  return (
    <div className={`editable-string-list ${className ?? ''}`}>
      <div className="rule-list-header">
        <span>{label}</span>
        {!readOnly && (
          <div className="editable-string-list-actions">
            <button className="secondary-button compact-action" type="button" onClick={() => onChange([...rows, ''])}>
              <Plus aria-hidden="true" />
              {addLabel}
            </button>
            {extraActions}
          </div>
        )}
      </div>
      <div className="rule-list-rows">
        {rows.map((value, index) => (
          <div className="rule-list-row" key={index}>
            <input
              aria-label={itemAriaLabel ? itemAriaLabel(value, index) : `${label} ${index + 1}`}
              readOnly={readOnly}
              placeholder={placeholder}
              value={value}
              onChange={(event) => setRow(index, event.target.value)}
            />
            {!readOnly && (
              <button
                aria-label={deleteLabel}
                className="icon-button"
                title={`${deleteLabel} ${label} ${value || index + 1}`}
                type="button"
                onClick={() => removeRow(index)}
              >
                <Trash2 aria-hidden="true" />
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}
