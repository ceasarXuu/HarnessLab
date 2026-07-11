import { http, HttpResponse } from 'msw'
import { createMockWebUiClient } from '../api/mockClient'
import type { AgentAvailability, AgentInputDto, AgentProfileType, AgentQuery, CreateJobRequestDto, DatasetImportRequestDto, DatasetParentPathRequestDto, DatasetPathRequestDto, EnvironmentDto, EnvironmentProfileType, EnvironmentQuery, ListQuery, UpdateJobLeaderboardRequestDto } from '../api/contract'

const webui = '*/api/webui/v1'
const client = createMockWebUiClient()

function listQuery(request: Request): ListQuery {
  const url = new URL(request.url)
  return {
    cursor: url.searchParams.get('cursor') ?? undefined,
    limit: Number(url.searchParams.get('limit')) || undefined,
    q: url.searchParams.get('q') ?? undefined,
  }
}

function agentQuery(request: Request): AgentQuery {
  const url = new URL(request.url)
  return {
    ...listQuery(request),
    status: optionalQueryValue<AgentAvailability>(url, 'status'),
    type: optionalQueryValue<AgentProfileType>(url, 'type'),
  }
}

function environmentQuery(request: Request): EnvironmentQuery {
  const url = new URL(request.url)
  return {
    ...listQuery(request),
    type: optionalQueryValue<EnvironmentProfileType>(url, 'type'),
  }
}

function optionalQueryValue<T extends string>(url: URL, key: string): T | undefined {
  return (url.searchParams.get(key) ?? undefined) as T | undefined
}

async function jsonBody<T>(request: Request): Promise<T> {
  return request.json() as Promise<T>
}

export const webuiHandlers = [
  http.get(`${webui}/operations/:operationId`, async ({ params }) =>
    HttpResponse.json(await client.getOperation(String(params.operationId))),
  ),
  http.post(`${webui}/operations/:operationId/cancel`, async ({ params }) =>
    HttpResponse.json(await client.cancelOperation(String(params.operationId))),
  ),
  http.post(`${webui}/agents`, async ({ request }) =>
    HttpResponse.json(await client.createAgent(await jsonBody<AgentInputDto>(request))),
  ),
  http.patch(`${webui}/agents/:agentId`, async ({ params, request }) =>
    HttpResponse.json(await client.updateAgent(String(params.agentId), await jsonBody<AgentInputDto>(request))),
  ),
  http.delete(`${webui}/agents/:agentId`, async ({ params }) =>
    HttpResponse.json(await client.deleteAgent(String(params.agentId))),
  ),
  http.get(`${webui}/agents`, async ({ request }) => HttpResponse.json(await client.listAgents(agentQuery(request)))),
  http.get(`${webui}/agents/:agentId`, async ({ params }) =>
    HttpResponse.json(await client.getAgent(String(params.agentId))),
  ),
  http.get(`${webui}/jobs`, async ({ request }) => HttpResponse.json(await client.listJobs(listQuery(request)))),
  http.get(`${webui}/jobs/:jobId`, async ({ params }) => HttpResponse.json(await client.getJob(String(params.jobId)))),
  http.get(`${webui}/jobs/:jobId/events`, async ({ params }) =>
    HttpResponse.json(await client.listJobEvents(String(params.jobId))),
  ),
  http.get(`${webui}/jobs/:jobId/trials`, async ({ params }) =>
    HttpResponse.json(await client.listJobTrials(String(params.jobId))),
  ),
  http.post(`${webui}/jobs`, async ({ request }) =>
    HttpResponse.json(await client.createJob(await jsonBody<CreateJobRequestDto>(request))),
  ),
  http.post(`${webui}/jobs/:jobId/cancel`, async ({ params }) =>
    HttpResponse.json(await client.cancelJob(String(params.jobId))),
  ),
  http.post(`${webui}/jobs/:jobId/resume`, async ({ params }) =>
    HttpResponse.json(await client.resumeJob(String(params.jobId))),
  ),
  http.patch(`${webui}/jobs/:jobId/leaderboard`, async ({ params, request }) =>
    HttpResponse.json(await client.updateJobLeaderboard(
      String(params.jobId),
      await jsonBody<UpdateJobLeaderboardRequestDto>(request),
    )),
  ),
  http.get(`${webui}/datasets`, async ({ request }) => HttpResponse.json(await client.listDatasets(listQuery(request)))),
  http.get(`${webui}/datasets/:datasetRef`, async ({ params }) =>
    HttpResponse.json(await client.getDataset(String(params.datasetRef))),
  ),
  http.get(`${webui}/datasets/:datasetRef/tasks`, async ({ params, request }) =>
    HttpResponse.json(await client.listDatasetTasks(String(params.datasetRef), listQuery(request))),
  ),
  http.post(`${webui}/datasets/import`, async ({ request }) =>
    HttpResponse.json(await client.importDataset(await jsonBody<DatasetImportRequestDto>(request))),
  ),
  http.get(`${webui}/datasets/storage/default-parent`, async () =>
    HttpResponse.json(await client.getDatasetDefaultParent()),
  ),
  http.post(`${webui}/datasets/:datasetRef/download`, async ({ params, request }) =>
    HttpResponse.json(await client.downloadDataset(String(params.datasetRef), await jsonBody<DatasetParentPathRequestDto>(request))),
  ),
  http.post(`${webui}/datasets/:datasetRef/download/cancel`, async ({ params }) =>
    HttpResponse.json(await client.cancelDatasetDownload(String(params.datasetRef))),
  ),
  http.delete(`${webui}/datasets/:datasetRef/local`, async ({ params }) =>
    HttpResponse.json(await client.deleteLocalDataset(String(params.datasetRef))),
  ),
  http.delete(`${webui}/datasets/:datasetRef/registration`, async ({ params }) =>
    HttpResponse.json(await client.removeDatasetRegistration(String(params.datasetRef))),
  ),
  http.post(`${webui}/datasets/:datasetRef/move`, async ({ params, request }) =>
    HttpResponse.json(await client.moveDataset(String(params.datasetRef), await jsonBody<DatasetParentPathRequestDto>(request))),
  ),
  http.post(`${webui}/datasets/:datasetRef/relocate`, async ({ params, request }) =>
    HttpResponse.json(await client.relocateDataset(String(params.datasetRef), await jsonBody<DatasetPathRequestDto>(request))),
  ),
  http.post(`${webui}/datasets/:datasetRef/sync`, async ({ params }) =>
    HttpResponse.json(await client.syncDataset(String(params.datasetRef))),
  ),
  http.post(`${webui}/environments`, async ({ request }) =>
    HttpResponse.json(await client.createEnvironment(await jsonBody<EnvironmentDto>(request))),
  ),
  http.patch(`${webui}/environments/:environmentId`, async ({ params, request }) =>
    HttpResponse.json(await client.updateEnvironment(String(params.environmentId), await jsonBody<EnvironmentDto>(request))),
  ),
  http.delete(`${webui}/environments/:environmentId`, async ({ params }) =>
    HttpResponse.json(await client.deleteEnvironment(String(params.environmentId))),
  ),
  http.post(`${webui}/environments/:environmentId/copy`, async ({ params }) =>
    HttpResponse.json(await client.copyEnvironment(String(params.environmentId))),
  ),
  http.get(`${webui}/environments`, async ({ request }) => HttpResponse.json(await client.listEnvironments(environmentQuery(request)))),
  http.get(`${webui}/environments/:environmentId`, async ({ params }) =>
    HttpResponse.json(await client.getEnvironment(String(params.environmentId))),
  ),
  http.get(`${webui}/leaderboard/datasets`, async ({ request }) =>
    HttpResponse.json(await client.listLeaderboardDatasets(listQuery(request))),
  ),
  http.get(`${webui}/leaderboard`, async ({ request }) => {
    const url = new URL(request.url)
    return HttpResponse.json(await client.listLeaderboard({
      ...listQuery(request),
      dataset: url.searchParams.get('dataset') ?? '',
      metric: url.searchParams.get('metric') ?? undefined,
    }))
  }),
  http.get(`${webui}/system/health`, async () => HttpResponse.json(await client.listSystemHealth())),
  http.get(`${webui}/system/hub-connection`, async () => HttpResponse.json(await client.getHubConnection())),
  http.post(`${webui}/system/service/update/check`, async () => HttpResponse.json(await client.checkForSystemUpdate())),
  http.post(`${webui}/system/service/update`, async () => HttpResponse.json(await client.installSystemUpdate())),
  http.post(`${webui}/system/service/restart`, async () => HttpResponse.json(await client.restartSystemService())),
  http.post(`${webui}/system/cache/docker/clean`, async () => HttpResponse.json(await client.cleanDockerCache())),
  http.post(`${webui}/system/cache/storage/clean`, async () => HttpResponse.json(await client.cleanStorageCache())),
]
