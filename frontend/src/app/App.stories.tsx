import type { Meta, StoryObj } from '@storybook/react-vite'
import type { WebUiClient } from '../api/webUiClient'
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
  client,
}: {
  client?: WebUiClient
  hash?: string
  locale?: Locale
  theme?: 'dark' | 'light'
}) {
  window.history.replaceState(null, '', hash)
  window.localStorage.setItem('ornnlab.locale', locale)
  window.localStorage.setItem('ornnlab.theme', theme)
  return <App client={client} />
}

const unavailableClient: WebUiClient = {
  getDataset: async () => ({ data: null, error: null }),
  getJob: async () => ({ data: null, error: null }),
  listDatasetTasks: async () => ({ data: null, error: null }),
  listDatasets: async () => ({ data: null, error: null }),
  listJobEvents: async () => ({ data: null, error: null }),
  listJobTrials: async () => ({ data: null, error: null }),
  listJobs: async () => ({
    data: null,
    error: { code: 'NETWORK_REQUEST_FAILED', message: 'The API request could not be completed.' },
  }),
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

export const JobsApiUnavailable: Story = {
  name: 'Jobs / API unavailable',
  render: () => <AppFixture client={unavailableClient} />,
}
