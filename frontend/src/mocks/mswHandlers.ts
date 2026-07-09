import { http, HttpResponse } from 'msw'
import { createMockWebUiClient } from '../api/mockClient'

const webui = '*/api/webui/v1'
const client = createMockWebUiClient()

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
  http.get(`${webui}/jobs/:jobId/events`, async ({ params }) =>
    HttpResponse.json(await client.listJobEvents(String(params.jobId))),
  ),
  http.get(`${webui}/jobs/:jobId/trials`, async ({ params }) =>
    HttpResponse.json(await client.listJobTrials(String(params.jobId))),
  ),
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
]
