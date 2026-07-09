import type { Operation, OperationStatus } from './contract'

interface MockOperationRecord {
  operation: Operation
  polls: number
}

export interface MockOperationStore {
  cancel: (id: string) => Operation | undefined
  get: (id: string) => Operation | undefined
  submit: (type: string, resourceType: Operation['resourceType'], resourceId?: string) => Operation
}

export function createMockOperationStore(): MockOperationStore {
  const operations = new Map<string, MockOperationRecord>()
  let sequence = 0

  return {
    cancel(id) {
      const record = operations.get(id)
      if (!record || isTerminal(record.operation.status)) return undefined
      record.operation = {
        ...record.operation,
        completedAt: timestamp(),
        message: 'Cancelled',
        status: 'cancelled',
      }
      return { ...record.operation }
    },
    get(id) {
      const record = operations.get(id)
      if (!record) return undefined
      record.polls += 1
      const status = nextStatus(record.operation.status, record.polls)
      record.operation = {
        ...record.operation,
        message: status === 'completed' ? 'Completed' : status === 'running' ? 'Running' : 'Queued',
        progress: status === 'completed' ? 100 : status === 'running' ? 50 : 0,
        status,
      }
      if (record.operation.status === 'completed') {
        record.operation = { ...record.operation, completedAt: timestamp() }
      }
      return { ...record.operation }
    },
    submit(type, resourceType, resourceId) {
      sequence += 1
      const operation: Operation = {
        id: `operation_${sequence}`,
        message: 'Queued',
        progress: 0,
        resourceId,
        resourceType,
        startedAt: timestamp(),
        status: 'queued',
        type,
      }
      operations.set(operation.id, { operation, polls: 0 })
      return { ...operation }
    },
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
