import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { useState } from 'react'
import { describe, expect, it, vi } from 'vitest'
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

  it('hides unit rates for Harness and loads them only after selecting LiteLLM', async () => {
    const pricingLoader = vi.fn(loadPricing)
    render(<PricingFixture loadPricing={pricingLoader} />)

    expect(screen.queryByText('$1.5')).not.toBeInTheDocument()
    expect(screen.getByText(/reported total price is shown after the Job completes/)).toBeInTheDocument()
    expect(pricingLoader).not.toHaveBeenCalled()

    fireEvent.click(screen.getByLabelText('Pricing source: deepseek-v4-pro'))
    fireEvent.click(screen.getByRole('option', { name: 'LiteLLM catalog' }))

    await waitFor(() => expect(screen.getByText('$1.5')).toBeInTheDocument())
    expect(screen.getByText('$0.15')).toBeInTheDocument()
    expect(screen.getByText('$6')).toBeInTheDocument()
    expect(pricingLoader).toHaveBeenCalledWith('deepseek-v4-pro')
    expect(screen.getByText(/saved as an immutable billing snapshot/)).toBeInTheDocument()
  })
})

function PricingFixture({ loadPricing: pricingLoader = loadPricing }: { loadPricing?: typeof loadPricing }) {
  const [value, setValue] = useState<AgentRow>({
    adapter: 'none', agentName: 'Claude DeepSeek', env: 'none', harness: 'claude-code',
    id: 'claude-deepseek', kwargs: 'none', mcp: 'none', models: 'deepseek-v4-pro',
    modelPricing: [{ modelName: 'deepseek-v4-pro', source: 'reported' }],
    skills: 'none', source: 'OrnnLab profile', status: 'configured', updated: '-',
  })
  return <AgentModelSettings loadPricing={pricingLoader} t={getTranslator('en')} value={value} onChange={setValue} />
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
