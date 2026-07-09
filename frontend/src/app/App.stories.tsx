import type { Meta, StoryObj } from '@storybook/react-vite'
import type { Locale } from '../i18n'
import { App } from './App'

const meta = {
  title: 'Harbor WebUI/App',
  component: App,
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof App>

export default meta
type Story = StoryObj<typeof meta>

function AppFixture({
  hash = '#jobs',
  locale = 'en',
  theme = 'dark',
}: {
  hash?: string
  locale?: Locale
  theme?: 'dark' | 'light'
}) {
  window.history.replaceState(null, '', hash)
  window.localStorage.setItem('ornnlab.locale', locale)
  window.localStorage.setItem('ornnlab.theme', theme)
  return <App />
}

export const Default: Story = {
  render: () => <AppFixture />,
}

export const LightChinese: Story = {
  name: 'Light / zh',
  render: () => <AppFixture locale="zh" theme="light" />,
}

export const NewJobRoute: Story = {
  render: () => <AppFixture hash="#jobs/new" />,
}

export const EnvironmentsRoute: Story = {
  render: () => <AppFixture hash="#environments" />,
}

export const AgentsNewRoute: Story = {
  render: () => <AppFixture hash="#agents/new" />,
}
