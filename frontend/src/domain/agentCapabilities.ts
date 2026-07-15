import type { AgentCapabilities, AgentCapabilityField, AgentRow } from './harbor'

const fullFields: AgentCapabilityField[] = [
  'customKwargs',
  'env',
  'harnessParameters',
  'mcpServers',
  'modelName',
  'skills',
  'timeouts',
]

export function fallbackAgentCapabilities(): AgentCapabilities {
  return {
    environmentVariables: [],
    parameters: [],
    supportedFields: [...fullFields],
  }
}

export function supportsAgentField(capabilities: AgentCapabilities | undefined, field: AgentCapabilityField) {
  return (capabilities ?? fallbackAgentCapabilities()).supportedFields.includes(field)
}

export function agentCapabilitiesByHarness(rows: AgentRow[]): Record<string, AgentCapabilities> {
  return rows.reduce<Record<string, AgentCapabilities>>((accumulator, row) => {
    if (row.capabilities) accumulator[row.harness] = row.capabilities
    return accumulator
  }, {})
}

export function agentCapabilitiesForHarness(
  harness: string,
  capabilitiesByHarness: Record<string, AgentCapabilities> | undefined,
) {
  return capabilitiesByHarness?.[harness] ?? fallbackAgentCapabilities()
}
