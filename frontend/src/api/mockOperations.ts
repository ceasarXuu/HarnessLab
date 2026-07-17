import type { Operation, OperationStatus } from './contract'

interface MockOperationRecord {
  operation: Operation
  polls: number
}

export interface MockOperationStore {
  active: (type: string) => Operation[]
  cancel: (id: string) => Operation | undefined
  cancelActive: (type: string, resourceId: string) => Operation | undefined
  complete: (type: string, resourceType: Operation['resourceType'], resourceId?: string, message?: string) => Operation
  fail: (type: string, resourceType: Operation['resourceType'], resourceId: string | undefined, code: string, message: string) => Operation
  get: (id: string) => Operation | undefined
  submit: (type: string, resourceType: Operation['resourceType'], resourceId?: string) => Operation
}

export function createMockOperationStore(): MockOperationStore {
  const operations = new Map<string, MockOperationRecord>()
  let sequence = 0
  const cancel = (id: string) => {
    const record = operations.get(id)
    if (!record || isTerminal(record.operation.status)) return undefined
    record.operation = {
      ...record.operation,
      completedAt: timestamp(),
      message: 'Cancelled',
      status: 'cancelled',
    }
    return { ...record.operation }
  }

  return {
    active(type) {
      return [...operations.values()]
        .map(({ operation }) => operation)
        .filter((operation) => operation.type === type && !isTerminal(operation.status))
        .map((operation) => ({ ...operation }))
    },
    cancel,
    cancelActive(type, resourceId) {
      const record = [...operations.values()].reverse().find(({ operation }) =>
        operation.type === type && operation.resourceId === resourceId && !isTerminal(operation.status),
      )
      return record ? cancel(record.operation.id) : undefined
    },
    complete(type, resourceType, resourceId, message = 'Completed') {
      const operation = create(type, resourceType, resourceId)
      operation.message = message
      operation.progress = 100
      operation.status = 'completed'
      operation.completedAt = timestamp()
      operations.set(operation.id, { operation, polls: 0 })
      return { ...operation }
    },
    fail(type, resourceType, resourceId, code, message) {
      const operation = create(type, resourceType, resourceId)
      operation.message = message
      operation.status = 'failed'
      operation.completedAt = timestamp()
      operation.error = { code, message }
      operations.set(operation.id, { operation, polls: 0 })
      return { ...operation }
    },
    get(id) {
      const record = operations.get(id)
      if (!record) return undefined
      if (isTerminal(record.operation.status)) return { ...record.operation }
      record.polls += 1
      const status = nextStatus(record.operation.status, record.polls)
      record.operation = {
        ...record.operation,
        message: status === 'completed' ? 'Completed' : status === 'running' ? 'Running' : 'Queued',
        progress: status === 'completed' ? 100 : status === 'running' ? 50 : 0,
        startedAt: status === 'running' ? record.operation.startedAt ?? timestamp() : record.operation.startedAt,
        status,
      }
      if (record.operation.status === 'completed') {
        record.operation = { ...record.operation, completedAt: timestamp() }
      }
      return { ...record.operation }
    },
    submit(type, resourceType, resourceId) {
      const operation = create(type, resourceType, resourceId)
      operations.set(operation.id, { operation, polls: 0 })
      return { ...operation }
    },
  }

  function create(type: string, resourceType: Operation['resourceType'], resourceId?: string): Operation {
    sequence += 1
    return {
      id: `operation_${sequence}`,
      message: 'Queued',
      progress: 0,
      resourceId,
      resourceType,
      status: 'queued',
      type,
    }
  }
}

function nextStatus(status: OperationStatus, polls: number): OperationStatus {
  if (status === 'queued' && polls >= 1) return 'running'
  if (status === 'running' && polls >= 2) return 'completed'
  return status
}

function isTerminal(status: OperationStatus) {
  return status === 'cancelled' || status === 'completed' || status === 'failed'
}

function timestamp() {
  return new Date().toISOString()
}
