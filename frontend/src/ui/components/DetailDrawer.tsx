import { X } from 'lucide-react'
import type { ReactNode } from 'react'

interface DetailDrawerProps {
  children: ReactNode
  label: string
  open: boolean
  onClose: () => void
}

export function DetailDrawer({ children, label, open, onClose }: DetailDrawerProps) {
  if (!open) return null

  return (
    <div className="drawer-layer">
      <button className="drawer-scrim" aria-label="Close detail drawer overlay" onClick={onClose} />
      <aside className="detail-drawer" aria-label={label} role="dialog">
        <button className="icon-button drawer-close" aria-label="Close detail drawer" onClick={onClose}>
          <X aria-hidden="true" />
        </button>
        {children}
      </aside>
    </div>
  )
}
