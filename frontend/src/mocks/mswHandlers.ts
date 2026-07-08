import { http, HttpResponse } from 'msw'
import { events, initialDraft, jobs, trialRows } from './demo'
import { agentRows, datasetRows, environmentRows, taskRows } from './demoCatalog'
import { leaderboardRows, systemRows } from './demoSystem'

const webui = '/api/webui/v1'

function data<T>(value: T) {
  return HttpResponse.json({ data: value })
}

export const webuiHandlers = [
  http.get(`${webui}/jobs`, () => data(jobs)),
  http.get(`${webui}/jobs/:jobId/events`, () => data(events)),
  http.get(`${webui}/trials`, () => data(trialRows)),
  http.get(`${webui}/jobs/draft`, () => data(initialDraft)),
  http.get(`${webui}/datasets`, () => data(datasetRows)),
  http.get(`${webui}/tasks`, () => data(taskRows)),
  http.get(`${webui}/agents`, () => data(agentRows)),
  http.get(`${webui}/environments`, () => data(environmentRows)),
  http.get(`${webui}/leaderboard`, () => data(leaderboardRows)),
  http.get(`${webui}/system`, () => data(systemRows)),
  http.post(`${webui}/operations`, () =>
    data({
      id: 'op_storybook_001',
      kind: 'storybook',
      status: 'completed',
    }),
  ),
]
