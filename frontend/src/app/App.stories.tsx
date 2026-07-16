import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { createUnavailableWebUiClient } from '../api/unavailableClient'
import { createMockWebUiClient } from '../api/mockClient'
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

const unavailableClient = createUnavailableWebUiClient()
const jobsLoadingClient = createUnavailableWebUiClient({ listJobs: () => new Promise(() => undefined) })
const datasetsLoadingClient = createUnavailableWebUiClient({ listDatasets: () => new Promise(() => undefined) })
const agentsLoadingClient = createUnavailableWebUiClient({ listAgents: () => new Promise(() => undefined) })
const environmentsLoadingClient = createUnavailableWebUiClient({ listEnvironments: () => new Promise(() => undefined) })
const leaderboardLoadingClient = createUnavailableWebUiClient({
  listLeaderboard: () => new Promise(() => undefined),
  listLeaderboardDatasets: () => new Promise(() => undefined),
})
const systemLoadingClient = createUnavailableWebUiClient({ listSystemHealth: () => new Promise(() => undefined) })
const hubDisconnectedClient = createMockWebUiClient()
hubDisconnectedClient.getHubConnection = async () => ({ data: { status: 'disconnected' }, error: null })

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

export const RunJobOpensDetail: Story = {
  render: () => <AppFixture hash="#jobs/new" />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    const runButton = await canvas.findByRole('button', { name: 'Run job' })
    await userEvent.click(runButton)
    const drawer = await canvas.findByRole('dialog', { name: 'Selected job' })
    await expect(within(drawer).getByRole('heading', { name: 'new-job' })).toBeVisible()
    await expect(within(drawer).getByText('Queued')).toBeVisible()
  },
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

export const JobsLoading: Story = {
  render: () => <AppFixture client={jobsLoadingClient} />,
}

export const HubDisconnected: Story = {
  render: () => <AppFixture client={hubDisconnectedClient} />,
}

export const DatasetsApiUnavailable: Story = {
  render: () => <AppFixture client={unavailableClient} hash="#datasets" />,
}

export const DatasetsLoading: Story = {
  render: () => <AppFixture client={datasetsLoadingClient} hash="#datasets" />,
}

export const AgentsApiUnavailable: Story = {
  render: () => <AppFixture client={unavailableClient} hash="#agents" />,
}

export const AgentsLoading: Story = {
  render: () => <AppFixture client={agentsLoadingClient} hash="#agents" />,
}

export const EnvironmentsApiUnavailable: Story = {
  render: () => <AppFixture client={unavailableClient} hash="#environments" />,
}

export const EnvironmentsLoading: Story = {
  render: () => <AppFixture client={environmentsLoadingClient} hash="#environments" />,
}

export const LeaderboardApiUnavailable: Story = {
  render: () => <AppFixture client={unavailableClient} hash="#leaderboard" />,
}

export const LeaderboardLoading: Story = {
  render: () => <AppFixture client={leaderboardLoadingClient} hash="#leaderboard" />,
}

export const SystemApiUnavailable: Story = {
  render: () => <AppFixture client={unavailableClient} hash="#system" />,
}

export const SystemLoading: Story = {
  render: () => <AppFixture client={systemLoadingClient} hash="#system" />,
}
