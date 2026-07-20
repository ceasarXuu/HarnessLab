import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, within } from 'storybook/test'
import { getTranslator } from '../../i18n'
import { jobs } from '../../mocks/demo'
import { ResourceStatus } from './ResourceStatus'
import { JobsTable } from './JobsTable'

const t = getTranslator('en')

const meta = {
  title: 'Components/JobsTable',
  component: JobsTable,
  parameters: { layout: 'fullscreen' },
  args: {
    jobs,
    search: '',
    selectedId: 'job_91a7',
    t,
    onNewJob: () => undefined,
    onSearch: () => undefined,
    onSelect: () => undefined,
  },
} satisfies Meta<typeof JobsTable>

export default meta
type Story = StoryObj<typeof meta>

export const Loaded: Story = {}

export const RunningProgress: Story = {
  args: { jobs: [jobs[0]] },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.queryByRole('columnheader', { name: 'Status' })).not.toBeInTheDocument()
    await expect(canvas.getByRole('columnheader', { name: 'Total tasks' })).toBeVisible()
    await expect(canvas.getByText('Passed 12')).toBeVisible()
    await expect(canvas.getByText('Not passed 6')).toBeVisible()
    await expect(canvas.getByLabelText('Running')).toBeVisible()
  },
}

export const FilteredEmpty: Story = {
  args: {
    jobs: [],
    search: 'no-match',
  },
}

export const Loading: Story = {
  args: { jobs: [] },
  render: (args) => (
    <>
      <JobsTable {...args} />
      <ResourceStatus error={null} loading loadingLabel={t('loadingJobs')} />
    </>
  ),
}

export const Error: Story = {
  args: { jobs: [] },
  render: (args) => (
    <>
      <JobsTable {...args} />
      <ResourceStatus error={t('unableToLoadJobs')} loading={false} loadingLabel={t('loadingJobs')} />
    </>
  ),
}
