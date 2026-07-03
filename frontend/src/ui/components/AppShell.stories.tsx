import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import { getTranslator, type Locale } from '../../i18n'
import { AppShell, type PageKey } from './AppShell'

function AppShellFixture() {
  const [activePage, setActivePage] = useState<PageKey>('datasets')
  const [language, setLanguage] = useState<Locale>('en')
  const [theme, setTheme] = useState<'light' | 'dark'>('dark')
  const t = getTranslator(language)

  return (
    <AppShell
      activePage={activePage}
      language={language}
      theme={theme}
      t={t}
      onLanguage={setLanguage}
      onNavigate={setActivePage}
      onTheme={() => setTheme((current) => (current === 'light' ? 'dark' : 'light'))}
    >
      <main className="workspace single-page">
        <section className="surface rail-card">
          <h1>{t('datasets')}</h1>
        </section>
      </main>
    </AppShell>
  )
}

const meta = {
  title: 'Components/AppShell',
  component: AppShellFixture,
  parameters: { layout: 'fullscreen' },
} satisfies Meta<typeof AppShellFixture>

export default meta
type Story = StoryObj<typeof meta>

export const HeaderNavigation: Story = {}
