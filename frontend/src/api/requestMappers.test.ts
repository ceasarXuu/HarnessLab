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
      config: { datasetRef: 'terminal-bench@2.0', environmentPresetId: 'docker-default', jobName: 'terminal-bench-smoke' },
      runImmediately: true,
    })
    expect(agent).toMatchObject({ id: 'claude-code-default', models: ['claude-haiku-4-5', 'claude-sonnet-4-5'] })
    expect(environment).toMatchObject({
      allowedHosts: ['pypi.org', 'github.com', 'huggingface.co'],
      dockerComposePaths: ['compose.gpu.yml'],
      id: 'docker-gpu',
    })
  })
})
