import { Activity, Bell, Github, Play, TerminalSquare } from 'lucide-react'
import type { ReactNode } from 'react'

const navItems = ['Jobs', 'New Run', 'Tasks', 'Trials', 'System']

export function AppShell({ children }: { children: ReactNode }) {
  return (
    <div className="app-shell">
      <header className="topbar">
        <a className="brand" href="/" aria-label="OrnnLab home">
          <TerminalSquare aria-hidden="true" />
          <span>OrnnLab</span>
          <small>Harbor</small>
        </a>
        <nav className="nav-links" aria-label="Primary">
          {navItems.map((item) => (
            <a key={item} href={`#${item.toLowerCase().replace(' ', '-')}`}>
              {item}
            </a>
          ))}
        </nav>
        <div className="topbar-actions">
          <span className="status-chip">
            <Activity aria-hidden="true" />
            Docker ready
          </span>
          <button className="icon-button" aria-label="Notifications">
            <Bell aria-hidden="true" />
          </button>
          <button className="icon-button" aria-label="GitHub">
            <Github aria-hidden="true" />
          </button>
          <button className="primary-button">
            <Play aria-hidden="true" />
            Run job
          </button>
        </div>
      </header>
      {children}
    </div>
  )
}
