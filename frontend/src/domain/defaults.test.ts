import { describe, expect, it } from 'vitest'
import { defaultRunDraft, jobConfigDtoToRunDraft, reconcileRunDraftResources } from './defaults'
import type { JobConfigDto } from '../api/contract'

describe('reconcileRunDraftResources', () => {
  const resources = {
    agents: [
      { agentName: 'Agent A', models: 'model-a, model-b' },
      { agentName: 'Configured agent', models: 'model-c, model-d' },
    ],
    datasets: [{ name: 'hello-world', version: '1.0' }],
    environments: [{ id: 'built-in:docker' }],
  }

  it('replaces demo or missing identifiers only with active resource values', () => {
    expect(reconcileRunDraftResources(defaultRunDraft, resources as never)).toMatchObject({
      agent: 'Agent A',
      environment: 'built-in:docker',
      model: 'model-a',
      source: 'hello-world@1.0',
    })
  })

  it('preserves a current valid selection', () => {
    const draft = { ...defaultRunDraft, agent: 'Configured agent', environment: 'built-in:docker', model: 'model-d', source: 'hello-world@1.0' }
    expect(reconcileRunDraftResources(draft, resources as never)).toMatchObject(draft)
  })

  it('leaves a required selector empty when the active API has no matching resource', () => {
    expect(reconcileRunDraftResources(defaultRunDraft, { ...resources, agents: [] } as never).agent).toBe('')
  })

  it('resets a model that is not available from the selected Agent', () => {
    const draft = { ...defaultRunDraft, agent: 'Configured agent', model: 'model-a' }
    expect(reconcileRunDraftResources(draft, resources as never).model).toBe('model-c')
  })

  it('maps a copied Job config and replaces references that no longer exist', () => {
    const config: JobConfigDto = {
      ...{
        agentSetupTimeoutMultiplier: 1,
        agentName: 'Deleted agent',
        agentTimeoutMultiplier: 1,
        attempts: 2,
        concurrency: 3,
        datasetRef: 'deleted-dataset@1.0',
        debug: true,
        environmentPresetId: 'deleted-environment',
        environmentBuildTimeoutMultiplier: 1,
        extraInstructionPaths: ['instructions/a.md'],
        includeInLeaderboard: false,
        jobName: 'source-copy',
        jobsDir: '/tmp/shared-jobs',
        maxRetries: 1,
        metric: 'mean' as const,
        modelName: 'deleted-model',
        notes: 'copied note',
        retryExclude: '',
        retryInclude: 'TimeoutError',
        retryMaxWaitSeconds: 5,
        retryMinWaitSeconds: 0,
        retryWaitMultiplier: 1,
        selectedTaskNames: ['hello'],
        timeoutMultiplier: 1,
        verifierTimeoutMultiplier: 1,
        verifierMode: 'dataset-default' as const,
      },
    }

    const copied = reconcileRunDraftResources(jobConfigDtoToRunDraft(config), resources as never)

    expect(copied).toMatchObject({
      agent: 'Agent A',
      environment: 'built-in:docker',
      jobName: 'source-copy',
      jobsDir: '/tmp/shared-jobs',
      model: 'model-a',
      source: 'hello-world@1.0',
      retryIntervalPolicy: 'fast',
    })
  })
})
