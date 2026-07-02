import { ChevronDown } from 'lucide-react'
import { type ReactNode, useState } from 'react'

interface SelectOption {
  label: string
  value: string
}

interface CustomSelectProps {
  ariaLabel: string
  options: SelectOption[]
  value: string
  className?: string
  leadingIcon?: ReactNode
  visibleLabel?: string
  onChange: (value: string) => void
}

export function CustomSelect({
  ariaLabel,
  className,
  leadingIcon,
  options,
  value,
  visibleLabel,
  onChange,
}: CustomSelectProps) {
  const [open, setOpen] = useState(false)
  const selected = options.find((option) => option.value === value) ?? options[0]

  return (
    <div
      className={`custom-select ${open ? 'open' : ''} ${className ?? ''}`}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget)) {
          setOpen(false)
        }
      }}
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          setOpen(false)
        }
      }}
    >
      {leadingIcon}
      {visibleLabel && <span>{visibleLabel}</span>}
      <button
        type="button"
        className="select-trigger"
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-label={ariaLabel}
        onClick={() => setOpen((current) => !current)}
      >
        <span>{selected?.label ?? value}</span>
        <ChevronDown aria-hidden="true" />
      </button>
      {open && (
        <div className="select-menu" role="listbox" aria-label={`${ariaLabel} options`}>
          {options.map((option) => (
            <button
              type="button"
              className={option.value === value ? 'active' : undefined}
              key={option.value}
              role="option"
              aria-selected={option.value === value}
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => {
                onChange(option.value)
                setOpen(false)
              }}
            >
              {option.label}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
