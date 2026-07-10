import { createMockWebUiClient } from './mockClient'
import { createWebUiHttpClient, type WebUiClient } from './webUiClient'
import { resolveWebUiDataMode, type WebUiDataMode } from './dataMode'

export type { WebUiDataMode } from './dataMode'

export function createRuntimeWebUiClient(
  mode: WebUiDataMode = readWebUiDataMode(),
  request: typeof fetch = fetch,
): WebUiClient {
  return mode === 'api' ? createWebUiHttpClient('/api/webui/v1', request) : createMockWebUiClient()
}

export function readWebUiDataMode(): WebUiDataMode {
  return resolveWebUiDataMode(import.meta.env.VITE_ORNNLAB_DATA_MODE, import.meta.env.PROD ? 'api' : 'mock')
}
