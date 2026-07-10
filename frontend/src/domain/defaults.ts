import type { AgentRow, DatasetRow, EnvironmentRow, RunDraft } from './harbor'

export const defaultRunDraft: RunDraft = {
  agent: '',
  agentSetupTimeoutMultiplier: '1',
  agentTimeoutMultiplier: '1',
  attempts: 1,
  concurrency: 4,
  debug: false,
  environment: '',
  environmentBuildTimeoutMultiplier: '1',
  extraInstructions: '',
  includeInLeaderboard: true,
  jobsDir: 'jobs/new-job',
  jobName: 'new-job',
  maxRetries: 0,
  metric: 'mean',
  notes: '',
  retryExclude: '',
  retryInclude: 'TimeoutError',
  retryIntervalPolicy: 'standard',
  retryMaxWaitSec: '30',
  retryMinWaitSec: '2',
  retryWaitMultiplier: '1.5',
  selectedTaskNames: null,
  source: '',
  timeoutMultiplier: 1,
  timeoutPolicy: 'standard',
  verifierMode: 'dataset-default',
  verifierTimeoutMultiplier: '1',
}

/**
 * Select only values obtained from the active data source. This keeps API mode
 * from submitting demo identifiers while preserving any still-valid user choice.
 */
export function reconcileRunDraftResources(
  draft: RunDraft,
  resources: { agents: AgentRow[]; datasets: DatasetRow[]; environments: EnvironmentRow[] },
): RunDraft {
  const datasets = resources.datasets.map((dataset) => `${dataset.name}@${dataset.version}`)
  const agents = resources.agents.filter((agent) => agent.type === 'custom').map((agent) => agent.agentName)
  const environments = resources.environments.map((environment) => environment.id)

  return {
    ...draft,
    agent: resolveResourceValue(draft.agent, agents),
    environment: resolveResourceValue(draft.environment, environments),
    source: resolveResourceValue(draft.source, datasets),
  }
}

function resolveResourceValue(current: string, values: string[]): string {
  return values.includes(current) ? current : (values[0] ?? '')
}

export const defaultEnvironmentDraft: EnvironmentRow = {
  allowedHosts: '*',
  cpuPolicy: 'auto',
  deleteAfterRun: false,
  dockerCompose: 'none',
  env: 'none',
  environmentType: 'docker',
  forceBuild: false,
  id: 'custom-environment',
  importPath: 'none',
  kwargs: 'none',
  memoryPolicy: 'auto',
  mounts: 'none',
  name: 'Custom Environment',
  overrideCpus: '',
  overrideGpus: '',
  overrideMemoryMb: '',
  overrideStorageMb: '',
  overrideTpu: '',
  profileType: 'custom',
}
