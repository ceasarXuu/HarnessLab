import { describe, expect, it } from 'vitest'
import { defaultRunDraft } from '../domain/defaults'
import { agentRows, environmentRows } from '../mocks/demoCatalog'
import { agentRowToDto, environmentRowToDto, runDraftToCreateJobRequest } from './requestMappers'

describe('WebUI mutation request mappers', () => {
  it('maps UI draft and editors to structured contract requests', () => {
    const job = runDraftToCreateJobRequest(defaultRunDraft)
    const agent = agentRowToDto(agentRows[0])
    const environment = environmentRowToDto(environmentRows[1])

    expect(job).toMatchObject({
      config: {
        agentSetupTimeoutMultiplier: 1,
        agentTimeoutMultiplier: 1,
        datasetRef: '',
        environmentBuildTimeoutMultiplier: 1,
        environmentPresetId: '',
        extraInstructionPaths: [],
        jobName: 'new-job',
        verifierTimeoutMultiplier: 1,
      },
      runImmediately: true,
    })
    expect(agent).toMatchObject({ id: 'claude-code-default', models: [] })
    expect(agent).not.toHaveProperty('status')
    expect(environment).toMatchObject({
      allowedHosts: ['pypi.org', 'github.com', 'huggingface.co'],
      dockerComposePaths: ['compose.gpu.yml'],
      id: 'docker-gpu',
    })
  })

  it('omits importPath for a configured built-in Harness', () => {
    const agent = agentRowToDto({
      ...agentRows[0],
      adapter: 'agents.custom:Agent',
      harness: 'nop',
      type: 'custom',
    })

    expect(agent.importPath).toBeUndefined()
  })
})
