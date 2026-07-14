interface SwitchControlProps {
  checked: boolean
  label: string
  className?: string
  disabled?: boolean
  onChange: (checked: boolean) => void
}

export function SwitchControl({ checked, className, disabled = false, label, onChange }: SwitchControlProps) {
  return (
    <label className={`switch-control${className ? ` ${className}` : ''}`}>
      <span>{label}</span>
      <input
        aria-label={label}
        checked={checked}
        disabled={disabled}
        role="switch"
        type="checkbox"
        onChange={(event) => onChange(event.target.checked)}
      />
    </label>
  )
}
