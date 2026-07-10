import type { KeyValueDto, McpServerDto } from './contract'

export function optional(value: string | undefined): string | undefined {
  return value && value !== 'none' ? value : undefined
}

export function parseKeyValues(value: string | undefined): KeyValueDto[] {
  if (!value || value === 'none') return []
  return value.split('\n').map((line) => {
    const [key, ...rest] = line.split('=')
    return { key: key.trim(), value: rest.join('=').trim() }
  }).filter((entry) => entry.key)
}

export function parseMcpServers(value: string | undefined): McpServerDto[] {
  if (!value || value === 'none') return []
  try {
    const parsed: unknown = JSON.parse(value)
    return Array.isArray(parsed) ? parsed as McpServerDto[] : []
  } catch {
    return []
  }
}

export function seconds(value: string | undefined): number | undefined {
  if (!value || value === 'none') return undefined
  const parsed = Number(value.replace(/s$/, ''))
  return Number.isFinite(parsed) ? parsed : undefined
}

export function splitList(value: string | undefined): string[] {
  if (!value || value === 'none') return []
  return value.split(/\n|,/).map((item) => item.trim()).filter(Boolean)
}
