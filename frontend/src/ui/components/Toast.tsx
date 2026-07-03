interface ToastProps {
  dismissLabel: string
  message: string
  remaining: number
  onDismiss: () => void
}

export function Toast({ dismissLabel, message, onDismiss, remaining }: ToastProps) {
  return (
    <div className="toast" role="status">
      <span>{message}</span>
      <span className="toast-countdown">{remaining}s</span>
      <button className="toast-close" aria-label={dismissLabel} onClick={onDismiss}>x</button>
    </div>
  )
}
