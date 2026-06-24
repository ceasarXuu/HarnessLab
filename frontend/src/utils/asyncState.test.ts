import { describe, it, expect } from 'vitest'
import { idle, loading, ready, empty, error, type AsyncState } from './asyncState'

describe('asyncState', () => {
  it('idle returns status idle', () => {
    expect(idle().status).toBe('idle')
  })

  it('loading returns status loading', () => {
    expect(loading().status).toBe('loading')
  })

  it('ready wraps data', () => {
    const s = ready({ name: 'test' })
    expect(s.status).toBe('ready')
    if (s.status === 'ready') expect(s.data.name).toBe('test')
  })

  it('empty returns status empty', () => {
    expect(empty().status).toBe('empty')
  })

  it('error wraps error', () => {
    const err = new Error('boom')
    const s = error<unknown, Error>(err)
    expect(s.status).toBe('error')
    if (s.status === 'error') expect(s.error).toBe(err)
  })

  it('discriminated union narrows correctly', () => {
    const states: AsyncState<string>[] = [
      idle<string>(),
      loading<string>(),
      ready<string>('ok'),
      empty<string>(),
      error<string, Error>(new Error('fail')),
    ]
    const labels = states.map((s) => {
      switch (s.status) {
        case 'idle': return 'i'
        case 'loading': return 'l'
        case 'ready': return `r:${s.data}`
        case 'empty': return 'e'
        case 'error': return `x:${(s.error as Error).message}`
      }
    })
    expect(labels).toEqual(['i', 'l', 'r:ok', 'e', 'x:fail'])
  })
})
