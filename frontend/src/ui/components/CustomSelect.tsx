import { ChevronDown } from 'lucide-react'
import { type ReactNode, useState } from 'react'

export interface SelectOption {
  badge?: {
    label: string
    tone: 'neutral' | 'success' | 'warning'
  }
  label: string
  value: string
}

interface CustomSelectProps {
  ariaLabel: string
  options: SelectOption[]
  value: string
  busy?: boolean
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
  busy = false,
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
  const isSearchable = searchable || options.length > 10
  const selected = options.find((option) => option.value === value)
  const activeSearch = searchValue ?? internalSearch
  const visibleOptions = isSearchable && !onSearchChange
    ? options.filter((option) => option.label.toLowerCase().includes(activeSearch.toLowerCase()))
    : options

  const updateSearch = (next: string) => {
    if (onSearchChange) onSearchChange(next)
    else setInternalSearch(next)
  }

  const closeMenu = () => {
    setOpen(false)
    if (!onSearchChange) setInternalSearch('')
  }

  const optionContent = (option: SelectOption) => (
    <span className="select-option-content">
      <span className="select-option-label">{option.label}</span>
      {option.badge && (
        <span className={`select-option-badge ${option.badge.tone}`}>{option.badge.label}</span>
      )}
    </span>
  )

  return (
    <div
      className={`custom-select ${open ? 'open' : ''} ${disabled ? 'disabled' : ''} ${className ?? ''}`}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget)) {
          closeMenu()
        }
      }}
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          closeMenu()
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
        aria-busy={busy}
        disabled={disabled}
        onClick={() => setOpen((current) => !current)}
      >
        {selected ? optionContent(selected) : <span>{value || placeholder || ''}</span>}
        <ChevronDown aria-hidden="true" />
      </button>
      {open && !disabled && (
        <div className="select-menu" role="listbox" aria-label={`${ariaLabel} options`}>
          {isSearchable && (
            <input
              aria-label={searchAriaLabel ?? searchPlaceholder ?? `Search ${ariaLabel}`}
              aria-busy={busy}
              autoFocus
              className="select-menu-search"
              placeholder={searchPlaceholder ?? `Search ${ariaLabel}`}
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
              aria-label={option.badge ? `${option.label} ${option.badge.label}` : undefined}
              aria-selected={option.value === value}
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => {
                onChange(option.value)
                closeMenu()
              }}
            >
              {optionContent(option)}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
