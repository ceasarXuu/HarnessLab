interface MetricProps {
  label: string
  value: string
  variant?: 'card' | 'plain'
}

export function Metric({ label, value, variant = 'card' }: MetricProps) {
  return (
    <div className={`metric metric--${variant}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  )
}
