import type { Meta, StoryObj } from '@storybook/react-vite'
import { getTranslator } from '../../i18n'
import { jobs } from '../../mocks/demo'
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

export const FilteredEmpty: Story = {
  args: {
    jobs: [],
    search: 'no-match',
  },
}
