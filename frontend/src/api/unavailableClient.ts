import type { ApiError, ApiResponse } from './contract'
import type { WebUiClient } from './webUiClient'

const unavailableError: ApiError = {
  code: 'NETWORK_REQUEST_FAILED',
  message: 'The API request could not be completed.',
}

export function createUnavailableWebUiClient(overrides: Partial<WebUiClient> = {}): WebUiClient {
  const unavailable = async (): Promise<ApiResponse<null>> => ({ data: null, error: unavailableError })
  return {
    cancelJob: unavailable,
    cancelDatasetDownload: unavailable,
    cancelOperation: unavailable,
    checkForSystemUpdate: unavailable,
    cleanDockerCache: unavailable,
    cleanStorageCache: unavailable,
    copyEnvironment: unavailable,
    createAgent: unavailable,
    createEnvironment: unavailable,
    createJob: unavailable,
    deleteAgent: unavailable,
    deleteEnvironment: unavailable,
    deleteLocalDataset: unavailable,
    downloadDataset: unavailable,
    getAgent: unavailable,
    getDataset: unavailable,
    getEnvironment: unavailable,
    getHubConnection: unavailable,
    getJob: unavailable,
    getOperation: unavailable,
    importDataset: unavailable,
    installSystemUpdate: unavailable,
    listAgents: unavailable,
    listDatasetTasks: unavailable,
    listDatasets: unavailable,
    listEnvironments: unavailable,
    listJobEvents: unavailable,
    listJobTrials: unavailable,
    listJobs: unavailable,
    listLeaderboard: unavailable,
    listLeaderboardDatasets: unavailable,
    listSystemHealth: unavailable,
    restartSystemService: unavailable,
    retryJob: unavailable,
    resumeJob: unavailable,
    runDatasetTask: unavailable,
    syncDataset: unavailable,
    updateAgent: unavailable,
    updateEnvironment: unavailable,
    updateJobLeaderboard: unavailable,
    ...overrides,
  }
}
