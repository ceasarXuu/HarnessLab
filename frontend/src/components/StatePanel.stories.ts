import type { Meta, StoryObj } from '@storybook/vue3-vite'

import StatePanel from './StatePanel.vue'
import { ApiError } from '@/api/client'
import { empty, error, idle, loading, ready } from '@/utils/asyncState'

const renderStatePanel = (args: Record<string, unknown>) => ({
  components: { StatePanel },
  setup: () => ({ args }),
  template: '<StatePanel v-bind="args" />',
})

const meta = {
  title: 'Console/StatePanel',
  args: {
    emptyMessage: 'No records matched this view.',
  },
  parameters: {
    layout: 'padded',
  },
} satisfies Meta<typeof StatePanel>

export default meta

type Story = StoryObj<typeof meta>

export const Loading: Story = {
  render: renderStatePanel,
  args: {
    state: loading(),
  },
}

export const Idle: Story = {
  render: renderStatePanel,
  args: {
    state: idle(),
  },
}

export const Empty: Story = {
  render: renderStatePanel,
  args: {
    state: empty(),
  },
}

export const Error500: Story = {
  render: renderStatePanel,
  args: {
    state: error(new ApiError('failed', 500, 'upstream unavailable')),
  },
}

export const Ready: Story = {
  render: (args) => ({
    components: { StatePanel },
    setup: () => ({ args }),
    template: `
      <StatePanel v-bind="args">
        <template #default="{ data }">
          <section class="panel stack" style="padding: 1rem;">
            <p class="eyebrow">Ready state</p>
            <strong>{{ data.title }}</strong>
            <p class="muted">{{ data.description }}</p>
          </section>
        </template>
      </StatePanel>
    `,
  }),
  args: {
    state: ready({
      title: 'Leaderboard data loaded',
      description: 'Ready state keeps chrome out of the StatePanel component.',
    }),
  },
}
