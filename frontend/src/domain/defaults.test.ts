import { describe, expect, it } from 'vitest'
import { defaultRunDraft, reconcileRunDraftResources } from './defaults'

describe('reconcileRunDraftResources', () => {
  const resources = {
    agents: [
      { agentName: 'Built-in', type: 'built-in' },
      { agentName: 'Configured agent', type: 'custom' },
    ],
    datasets: [{ name: 'hello-world', version: '1.0' }],
    environments: [{ id: 'built-in:docker' }],
  }

  it('replaces demo or missing identifiers only with active resource values', () => {
    expect(reconcileRunDraftResources(defaultRunDraft, resources as never)).toMatchObject({
      agent: 'Configured agent',
      environment: 'built-in:docker',
      source: 'hello-world@1.0',
    })
  })

  it('preserves a current valid selection', () => {
    const draft = { ...defaultRunDraft, agent: 'Configured agent', environment: 'built-in:docker', source: 'hello-world@1.0' }
    expect(reconcileRunDraftResources(draft, resources as never)).toMatchObject(draft)
  })

  it('leaves a required selector empty when the active API has no matching resource', () => {
    expect(reconcileRunDraftResources(defaultRunDraft, { ...resources, agents: [] } as never).agent).toBe('')
  })
})
