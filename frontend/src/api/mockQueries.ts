import type {
  AgentDto,
  AgentQuery,
  DatasetTaskDto,
  DatasetTaskQuery,
  EnvironmentDto,
  EnvironmentQuery,
  ListQuery,
  Page,
} from './contract'

export function page<T>(items: T[], query?: ListQuery): Page<T> {
  const offset = Number.parseInt(query?.cursor ?? '0', 10) || 0
  const limit = query?.limit ?? 50
  const selected = items.slice(offset, offset + limit)
  const nextCursor = offset + limit < items.length ? String(offset + limit) : undefined
  return { items: selected, nextCursor, total: items.length }
}

export function filterByQuery<T>(
  items: T[],
  query: ListQuery | undefined,
  fields: (item: T) => string[],
): T[] {
  const needle = query?.q?.trim().toLowerCase()
  if (!needle) return items
  return items.filter((item) => fields(item).some((field) => field.toLowerCase().includes(needle)))
}

export function filterAgents(items: AgentDto[], query: AgentQuery | undefined) {
  const matched = filterByQuery(items, query, (agent) => [
    agent.agentName,
    agent.harness,
    agent.status,
  ])
  return matched.filter((agent) => !query?.status || agent.status === query.status)
}

export function filterEnvironments(items: EnvironmentDto[], query: EnvironmentQuery | undefined) {
  const matched = filterByQuery(items, query, (environment) => [
    environment.name,
    environment.environmentType,
    environment.profileType,
  ])
  return matched.filter((environment) => !query?.type || environment.profileType === query.type)
}

export function filterDatasetTasks(
  tasks: DatasetTaskDto[],
  ref: string,
  query: DatasetTaskQuery | undefined,
): DatasetTaskDto[] {
  const byDataset = tasks.filter((task) => task.datasetRef === ref)
  return filterByQuery(byDataset, query, (task) => [task.name])
}
