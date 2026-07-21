import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { useState } from 'react'
import type { AgentRow } from '../../domain/harbor'
import { getTranslator } from '../../i18n'
import { AgentModelSettings } from './AgentModelSettings'

function PricingFixture() {
  const [value, setValue] = useState<AgentRow>({
    adapter: 'none', agentName: 'Claude DeepSeek', env: 'none', harness: 'claude-code',
    id: 'claude-deepseek', kwargs: 'none', mcp: 'none', models: 'deepseek-v4-pro, deepseek-v4-flash',
    modelPricing: [
      { modelName: 'deepseek-v4-pro', source: 'litellm' },
      {
        modelName: 'deepseek-v4-flash', source: 'custom',
        inputCacheMissUsdPerMillion: 0.2, inputCacheHitUsdPerMillion: 0.02,
        outputUsdPerMillion: 0.6,
      },
    ],
    skills: 'none', source: 'OrnnLab profile', status: 'configured', updated: '-',
  })
  return <AgentModelSettings loadPricing={loadPricing} t={getTranslator('en')} value={value} onChange={setValue} />
}

async function loadPricing(modelName: string) {
  return {
    data: {
      catalogModelName: modelName,
      inputCacheHitUsdPerMillion: 0.02,
      inputCacheMissUsdPerMillion: 0.2,
      modelName,
      outputUsdPerMillion: 0.6,
      source: 'litellm' as const,
    },
    error: null,
  }
}

const meta = {
  component: PricingFixture,
  parameters: { layout: 'padded' },
  title: 'Patterns/Agent/Model Pricing',
} satisfies Meta<typeof PricingFixture>

export default meta
type Story = StoryObj<typeof meta>

export const MixedPricingSources: Story = {
  render: () => <PricingFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.getByLabelText('Pricing source: deepseek-v4-pro')).toHaveTextContent('LiteLLM catalog')
    await expect(await canvas.findByText('$0.2')).toBeVisible()
    await expect(canvas.getByLabelText('Pricing source: deepseek-v4-flash')).toHaveTextContent('Custom pricing')
    await userEvent.clear(canvas.getByLabelText('Output (USD / 1M tokens)'))
    await userEvent.type(canvas.getByLabelText('Output (USD / 1M tokens)'), '0.8')
    await expect(canvas.getByLabelText('Output (USD / 1M tokens)')).toHaveValue(0.8)
  },
}
