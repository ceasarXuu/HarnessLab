import { X } from 'lucide-react'
import { useState } from 'react'
import type { CSSProperties, PointerEvent, ReactNode } from 'react'

const DEFAULT_DRAWER_WIDTH = 560
const VIEWPORT_SAFE_GAP = 32

interface DetailDrawerProps {
  children: ReactNode
  label: string
  open: boolean
  onClose: () => void
}

export function DetailDrawer({ children, label, open, onClose }: DetailDrawerProps) {
  const [drawerWidth, setDrawerWidth] = useState(DEFAULT_DRAWER_WIDTH)

  if (!open) return null

  const viewportWidth = typeof window === 'undefined' ? DEFAULT_DRAWER_WIDTH + VIEWPORT_SAFE_GAP : window.innerWidth
  const minWidth = Math.min(DEFAULT_DRAWER_WIDTH, viewportWidth - VIEWPORT_SAFE_GAP)
  const maxWidth = Math.max(minWidth, viewportWidth - VIEWPORT_SAFE_GAP)
  const currentWidth = Math.min(Math.max(drawerWidth, minWidth), maxWidth)
  const drawerStyle = { '--drawer-width': `${currentWidth}px`, width: currentWidth } as CSSProperties

  function resizeDrawer(event: PointerEvent<HTMLButtonElement>) {
    if (event.buttons !== 1) return
    const nextWidth = viewportWidth - event.clientX
    setDrawerWidth(Math.min(Math.max(nextWidth, minWidth), maxWidth))
  }

  return (
    <div className="drawer-layer">
      <button className="drawer-scrim" aria-label="Close detail drawer overlay" onClick={onClose} />
      <aside className="detail-drawer" aria-label={label} role="dialog" style={drawerStyle}>
        <button
          type="button"
          className="drawer-resize-handle"
          aria-label="Resize detail drawer"
          aria-orientation="vertical"
          aria-valuemax={maxWidth}
          aria-valuemin={minWidth}
          aria-valuenow={currentWidth}
          role="separator"
          onPointerDown={(event) => {
            event.currentTarget.setPointerCapture?.(event.pointerId)
            resizeDrawer(event)
          }}
          onPointerMove={resizeDrawer}
        />
        <button className="icon-button drawer-close" aria-label="Close detail drawer" onClick={onClose}>
          <X aria-hidden="true" />
        </button>
        {children}
      </aside>
    </div>
  )
}
