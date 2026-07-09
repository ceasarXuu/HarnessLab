import { http, HttpResponse } from 'msw'
import { createMockWebUiClient } from '../api/mockClient'
import { events, trialRows } from './demo'
import { agentRows, environmentRows } from './demoCatalog'
import { leaderboardRows, systemRows } from './demoSystem'

const webui = '*/api/webui/v1'
const client = createMockWebUiClient()

function data<T>(value: T) {
  return HttpResponse.json({ data: value, error: null })
}

function listQuery(request: Request) {
  const url = new URL(request.url)
  return {
    cursor: url.searchParams.get('cursor') ?? undefined,
    limit: Number(url.searchParams.get('limit')) || undefined,
    q: url.searchParams.get('q') ?? undefined,
  }
}

export const webuiHandlers = [
  http.get(`${webui}/jobs`, async ({ request }) => HttpResponse.json(await client.listJobs(listQuery(request)))),
  http.get(`${webui}/jobs/:jobId`, async ({ params }) => HttpResponse.json(await client.getJob(String(params.jobId)))),
  http.get(`${webui}/jobs/:jobId/events`, () => data(events)),
  http.get(`${webui}/jobs/:jobId/trials`, ({ params }) => data(trialRows.filter((trial) => trial.jobId === params.jobId))),
  http.get(`${webui}/datasets`, async ({ request }) => HttpResponse.json(await client.listDatasets(listQuery(request)))),
  http.get(`${webui}/datasets/:datasetRef`, async ({ params }) =>
    HttpResponse.json(await client.getDataset(String(params.datasetRef))),
  ),
  http.get(`${webui}/datasets/:datasetRef/tasks`, async ({ params, request }) => {
    const url = new URL(request.url)
    return HttpResponse.json(await client.listDatasetTasks(String(params.datasetRef), {
      ...listQuery(request),
      split: url.searchParams.get('split') ?? undefined,
    }))
  }),
  http.get(`${webui}/agents`, () => data(agentRows)),
  http.get(`${webui}/environments`, () => data(environmentRows)),
  http.get(`${webui}/leaderboard`, () => data(leaderboardRows)),
  http.get(`${webui}/system/health`, () => data(systemRows)),
]
