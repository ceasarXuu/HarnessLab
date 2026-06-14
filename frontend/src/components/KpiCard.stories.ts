import type { Meta, StoryObj } from '@storybook/vue3-vite'

import KpiCard from './KpiCard.vue'

const meta = {
  title: 'Console/KpiCard',
  component: KpiCard,
  args: {
    metric: {
      label: 'Active evaluations',
      value: '42',
      delta: '+6 since 02:00',
      trend: 'up',
      description: 'Cross-benchmark runs currently executing.',
    },
  },
} satisfies Meta<typeof KpiCard>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {}

