import { Github, Languages, Moon, Sun, TerminalSquare } from 'lucide-react'
import type { ReactNode } from 'react'
import { localeNames, type Locale, type Translate } from '../../i18n'
import { CustomSelect } from './CustomSelect'

export type PageKey = 'jobs' | 'datasets' | 'agents' | 'environments' | 'leaderboard' | 'system'
export type HubConnectionState = 'connected' | 'disconnected' | 'expired' | 'loading' | 'unavailable'

const navItems: Array<{ key: PageKey; label: Parameters<Translate>[0] }> = [
  { key: 'jobs', label: 'jobs' },
  { key: 'datasets', label: 'datasets' },
  { key: 'agents', label: 'agents' },
  { key: 'environments', label: 'environments' },
  { key: 'leaderboard', label: 'leaderboard' },
  { key: 'system', label: 'system' },
]

interface AppShellProps {
  activePage: PageKey
  children: ReactNode
  hubConnection: HubConnectionState
  language: Locale
  theme: 'light' | 'dark'
  t: Translate
  onLanguage: (language: Locale) => void
  onNavigate: (page: PageKey) => void
  onTheme: () => void
}

export function AppShell({
  activePage,
  children,
  hubConnection,
  language,
  theme,
  t,
  onLanguage,
  onNavigate,
  onTheme,
}: AppShellProps) {
  return (
    <div className="app-shell">
      <header className="topbar">
        <a
          className="brand"
          href="#jobs"
          aria-label={t('appHome')}
          onClick={(event) => {
            event.preventDefault()
            onNavigate('jobs')
          }}
        >
          <TerminalSquare aria-hidden="true" />
          <span>OrnnLab</span>
          <small>{t('harbor')}</small>
        </a>
        <nav className="nav-links" aria-label={t('primaryNavigation')}>
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
          <span className={`secondary-button auth-chip auth-chip-${hubConnection}`} role="status">
            <Github aria-hidden="true" />
            {hubConnectionLabel(t, hubConnection)}
          </span>
          <CustomSelect
            ariaLabel={t('language')}
            className="header-select"
            leadingIcon={<Languages aria-hidden="true" />}
            value={language}
            options={[
              { label: localeNames.en, value: 'en' },
              { label: localeNames.zh, value: 'zh' },
            ]}
            onChange={(value) => onLanguage(value as Locale)}
          />
          <button className="icon-button" aria-label={theme === 'light' ? t('dark') : t('light')} onClick={onTheme}>
            {theme === 'light' ? <Moon aria-hidden="true" /> : <Sun aria-hidden="true" />}
          </button>
        </div>
      </header>
      {children}
    </div>
  )
}

function hubConnectionLabel(t: Translate, status: HubConnectionState) {
  if (status === 'connected') return t('harborAuthReady')
  if (status === 'disconnected') return t('harborAuthDisconnected')
  if (status === 'expired') return t('harborAuthExpired')
  if (status === 'loading') return t('harborAuthLoading')
  return t('harborAuthUnavailable')
}
