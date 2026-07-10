export type WebUiDataMode = 'api' | 'mock'

const supportedModes = new Set<WebUiDataMode>(['api', 'mock'])

export function resolveWebUiDataMode(value: string | undefined, fallback: WebUiDataMode): WebUiDataMode {
  if (value === undefined || value === '') return fallback
  if (supportedModes.has(value as WebUiDataMode)) return value as WebUiDataMode
  throw new Error(`VITE_ORNNLAB_DATA_MODE must be "api" or "mock", received "${value}".`)
}
