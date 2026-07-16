import type { ReactNode } from 'react'
import { FieldError } from './FormFeedback'

export function TabPanel({ active, children, title }: { active: boolean; children: ReactNode; title: string }) {
  if (!active) {
    return null
  }
  return (
    <section className="run-section" role="tabpanel">
      <div className="run-section-heading">
        <h2>{title}</h2>
      </div>
      {children}
    </section>
  )
}

export function Field({ children, error, errorId, label, wide = false }: { children: ReactNode; error?: string; errorId?: string; label: string; wide?: boolean }) {
  return (
    <label className={wide ? 'field-wide' : undefined}>
      {label}
      {children}
      {errorId && <FieldError id={errorId} message={error} />}
    </label>
  )
}

export function Toggle({ checked, onChange }: { checked: boolean; onChange: (value: boolean) => void }) {
  return (
    <button type="button" className={checked ? 'toggle active' : 'toggle'} onClick={() => onChange(!checked)}>
      {checked ? 'enabled' : 'disabled'}
    </button>
  )
}
