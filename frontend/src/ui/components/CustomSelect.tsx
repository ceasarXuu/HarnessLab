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
  disabled?: boolean
  leadingIcon?: ReactNode
  placeholder?: string
  searchPlaceholder?: string
  searchAriaLabel?: string
  searchable?: boolean
  searchValue?: string
  visibleLabel?: string
  onChange: (value: string) => void
  onSearchChange?: (value: string) => void
}

export function CustomSelect({
  ariaLabel,
  className,
  disabled = false,
  leadingIcon,
  options,
  placeholder,
  searchPlaceholder,
  searchAriaLabel,
  searchable = false,
  searchValue,
  value,
  visibleLabel,
  onChange,
  onSearchChange,
}: CustomSelectProps) {
  const [open, setOpen] = useState(false)
  const [internalSearch, setInternalSearch] = useState('')
  const selected = options.find((option) => option.value === value)
  const activeSearch = searchValue ?? internalSearch
  const visibleOptions = searchable && !onSearchChange
    ? options.filter((option) => option.label.toLowerCase().includes(activeSearch.toLowerCase()))
    : options

  const updateSearch = (next: string) => {
    if (onSearchChange) onSearchChange(next)
    else setInternalSearch(next)
  }

  return (
    <div
      className={`custom-select ${open ? 'open' : ''} ${disabled ? 'disabled' : ''} ${className ?? ''}`}
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
        disabled={disabled}
        onClick={() => setOpen((current) => !current)}
      >
        <span>{selected?.label ?? (value || placeholder || '')}</span>
        <ChevronDown aria-hidden="true" />
      </button>
      {open && !disabled && (
        <div className="select-menu" role="listbox" aria-label={`${ariaLabel} options`}>
          {searchable && (
            <input
              aria-label={searchAriaLabel ?? searchPlaceholder ?? `Search ${ariaLabel}`}
              autoFocus
              className="select-menu-search"
              value={activeSearch}
              onChange={(event) => updateSearch(event.target.value)}
            />
          )}
          {visibleOptions.map((option) => (
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
