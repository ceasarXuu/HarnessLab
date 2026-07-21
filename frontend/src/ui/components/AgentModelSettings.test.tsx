import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { useState } from 'react'
import { describe, expect, it } from 'vitest'
import type { AgentRow } from '../../domain/harbor'
import { getTranslator } from '../../i18n'
import { AgentModelSettings } from './AgentModelSettings'

describe('AgentModelSettings', () => {
  it('configures custom cache-aware prices independently for each model', () => {
    render(<PricingFixture />)

    fireEvent.click(screen.getByLabelText('Pricing source: deepseek-v4-pro'))
    fireEvent.click(screen.getByRole('option', { name: 'Custom pricing' }))
    fireEvent.change(screen.getByLabelText('Input cache miss (USD / 1M tokens)'), {
      target: { value: '0.435' },
    })
    fireEvent.change(screen.getByLabelText('Input cache hit (USD / 1M tokens)'), {
      target: { value: '0.003625' },
    })
    fireEvent.change(screen.getByLabelText('Output (USD / 1M tokens)'), {
      target: { value: '0.87' },
    })

    expect(screen.getByLabelText('Input cache miss (USD / 1M tokens)')).toHaveValue(0.435)
    expect(screen.getByLabelText('Input cache hit (USD / 1M tokens)')).toHaveValue(0.003625)
    expect(screen.getByLabelText('Output (USD / 1M tokens)')).toHaveValue(0.87)
  })

  it('offers Harness, LiteLLM, and custom sources', () => {
    render(<PricingFixture />)

    fireEvent.click(screen.getByLabelText('Pricing source: deepseek-v4-pro'))

    expect(screen.getByRole('option', { name: 'Harness reported' })).toBeInTheDocument()
    expect(screen.getByRole('option', { name: 'LiteLLM catalog' })).toBeInTheDocument()
    expect(screen.getByRole('option', { name: 'Custom pricing' })).toBeInTheDocument()
  })

  it('shows resolved rates for both Harness and LiteLLM sources', async () => {
    render(<PricingFixture />)

    await waitFor(() => expect(screen.getByText('$1.5')).toBeInTheDocument())
    expect(screen.getByText('$0.15')).toBeInTheDocument()
    expect(screen.getByText('$6')).toBeInTheDocument()
    expect(screen.getByText(/actual cost uses the total reported by the Harness/)).toBeInTheDocument()

    fireEvent.click(screen.getByLabelText('Pricing source: deepseek-v4-pro'))
    fireEvent.click(screen.getByRole('option', { name: 'LiteLLM catalog' }))

    expect(screen.getByText(/saved as an immutable billing snapshot/)).toBeInTheDocument()
  })
})

function PricingFixture() {
  const [value, setValue] = useState<AgentRow>({
    adapter: 'none', agentName: 'Claude DeepSeek', env: 'none', harness: 'claude-code',
    id: 'claude-deepseek', kwargs: 'none', mcp: 'none', models: 'deepseek-v4-pro',
    modelPricing: [{ modelName: 'deepseek-v4-pro', source: 'reported' }],
    skills: 'none', source: 'OrnnLab profile', status: 'configured', updated: '-',
  })
  return <AgentModelSettings loadPricing={loadPricing} t={getTranslator('en')} value={value} onChange={setValue} />
}

async function loadPricing(modelName: string) {
  return {
    data: {
      catalogModelName: modelName,
      inputCacheHitUsdPerMillion: 0.15,
      inputCacheMissUsdPerMillion: 1.5,
      modelName,
      outputUsdPerMillion: 6,
      source: 'litellm' as const,
    },
    error: null,
  }
}
