import type { ReactNode } from 'react'

interface ConfirmDialogProps {
  cancelLabel: string
  children?: ReactNode
  confirmLabel: string
  impacts?: string[]
  title: string
  onCancel: () => void
  onConfirm: () => void
}

export function ConfirmDialog({ cancelLabel, children, confirmLabel, impacts, title, onCancel, onConfirm }: ConfirmDialogProps) {
  return (
    <div className="confirm-overlay">
      <section className="surface confirm-dialog" role="dialog" aria-modal="true" aria-label={title}>
        <div className="confirm-heading">
          <h2>{title}</h2>
        </div>
        {children}
        {impacts && (
          <ul className="cleanup-impact-list">
            {impacts.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        )}
        <div className="button-row confirm-actions">
          <button className="secondary-button" onClick={onCancel}>{cancelLabel}</button>
          <button className="primary-button" onClick={onConfirm}>{confirmLabel}</button>
        </div>
      </section>
    </div>
  )
}
