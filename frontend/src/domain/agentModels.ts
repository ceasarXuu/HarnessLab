import type { AgentRow } from './harbor'

export function agentModelNames(agent: AgentRow | undefined): string[] {
  if (!agent || !agent.models || agent.models === '-') return []
  return agent.models
    .split(/\n|,/)
    .map((model) => model.trim())
    .filter(Boolean)
}

export function reconcileAgentModel(current: string, agent: AgentRow | undefined): string {
  const models = agentModelNames(agent)
  return models.includes(current) ? current : (models[0] ?? '')
}
