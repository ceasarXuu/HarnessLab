import { Activity, Bell, Github, Languages, Moon, Play, Sun, TerminalSquare } from 'lucide-react'
import type { ReactNode } from 'react'
import type { Locale, Translate } from '../i18n'

export type PageKey = 'jobs' | 'datasets' | 'agents' | 'leaderboard' | 'system'

const navItems: Array<{ key: PageKey; label: Parameters<Translate>[0] }> = [
  { key: 'jobs', label: 'jobs' },
  { key: 'datasets', label: 'datasets' },
  { key: 'agents', label: 'agents' },
  { key: 'leaderboard', label: 'leaderboard' },
  { key: 'system', label: 'system' },
]

interface AppShellProps {
  activePage: PageKey
  children: ReactNode
  language: Locale
  theme: 'light' | 'dark'
  t: Translate
  onLanguage: (language: Locale) => void
  onNavigate: (page: PageKey) => void
  onNewJob: () => void
  onTheme: () => void
}

export function AppShell({
  activePage,
  children,
  language,
  theme,
  t,
  onLanguage,
  onNavigate,
  onNewJob,
  onTheme,
}: AppShellProps) {
  return (
    <div className="app-shell">
      <header className="topbar">
        <a
          className="brand"
          href="#jobs"
          aria-label="OrnnLab home"
          onClick={(event) => {
            event.preventDefault()
            onNavigate('jobs')
          }}
        >
          <TerminalSquare aria-hidden="true" />
          <span>OrnnLab</span>
          <small>{t('harbor')}</small>
        </a>
        <nav className="nav-links" aria-label="Primary">
          {navItems.map((item) => (
            <a
              key={item.key}
              className={item.key === activePage ? 'active' : undefined}
              href={`#${item.key}`}
              onClick={(event) => {
                event.preventDefault()
                onNavigate(item.key)
              }}
            >
              {t(item.label)}
            </a>
          ))}
        </nav>
        <div className="topbar-actions">
          <span className="status-chip">
            <Activity aria-hidden="true" />
            {t('dockerReady')}
          </span>
          <button className="icon-button" aria-label={t('notifications')}>
            <Bell aria-hidden="true" />
          </button>
          <button className="icon-button" aria-label={t('github')}>
            <Github aria-hidden="true" />
          </button>
          <label className="header-select">
            <Languages aria-hidden="true" />
            <select aria-label="Language" value={language} onChange={(event) => onLanguage(event.target.value as Locale)}>
              <option value="en">EN</option>
              <option value="zh">中</option>
            </select>
          </label>
          <button className="icon-button" aria-label={theme === 'light' ? t('dark') : t('light')} onClick={onTheme}>
            {theme === 'light' ? <Moon aria-hidden="true" /> : <Sun aria-hidden="true" />}
          </button>
          <button className="primary-button" onClick={onNewJob}>
            <Play aria-hidden="true" />
            {t('runJob')}
          </button>
        </div>
      </header>
      {children}
    </div>
  )
}
