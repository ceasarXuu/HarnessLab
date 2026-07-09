import { createMockWebUiClient } from './mockClient'
import { createWebUiHttpClient, type WebUiClient } from './webUiClient'

export type WebUiDataMode = 'api' | 'mock'

export function createRuntimeWebUiClient(
  mode: WebUiDataMode = readWebUiDataMode(),
  request: typeof fetch = fetch,
): WebUiClient {
  return mode === 'api' ? createWebUiHttpClient('/api/webui/v1', request) : createMockWebUiClient()
}

export function readWebUiDataMode(): WebUiDataMode {
  return import.meta.env.VITE_ORNNLAB_DATA_MODE === 'api' ? 'api' : 'mock'
}
