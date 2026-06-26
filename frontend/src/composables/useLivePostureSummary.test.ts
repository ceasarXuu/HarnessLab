import { defineComponent, nextTick } from 'vue'
import { flushPromises, mount } from '@vue/test-utils'
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'

import { i18n, setLocale } from '@/i18n'
import { useLivePostureSummary } from './useLivePostureSummary'

setLocale('en')

const mockFetch = vi.fn()
const originalFetch = globalThis.fetch

beforeEach(() => {
  globalThis.fetch = mockFetch as unknown as typeof fetch
  mockFetch.mockReset()
})

afterEach(() => {
  globalThis.fetch = originalFetch
})

const okJson = (data: unknown) =>
  new Response(JSON.stringify(data), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  })

const TestHost = defineComponent({
  setup() {
    return useLivePostureSummary()
  },
  template: '<p>{{ statusLine }}</p>',
})

describe('useLivePostureSummary', () => {
  it('fetches live posture and exposes a derived status line', async () => {
    mockFetch.mockImplementation(async (input: string) => {
      if (input === '/api/experiments') {
        return okJson([
          { id: 'e1', name: 'run', kind: 'batch', status: 'running', requested_run_count: 1, mode: 'tb', created_at: '', updated_at: '' },
        ])
      }
      if (input === '/api/agents') {
        return okJson([
          { id: 'a1', name: 'compiled', kind: 'harbor', harbor_agent_name: null, harbor_import_path: null, model_name: null, status: 'compiled', profile_path: '', created_at: '', updated_at: '' },
          { id: 'a2', name: 'blocked', kind: 'harbor', harbor_agent_name: null, harbor_import_path: null, model_name: null, status: 'failed', profile_path: '', created_at: '', updated_at: '' },
        ])
      }
      return new Response('not mocked', { status: 500 })
    })

    const wrapper = mount(TestHost, { global: { plugins: [i18n] } })
    await flushPromises()
    await nextTick()

    expect(wrapper.text()).toContain('2 agents live')
    expect(wrapper.text()).toContain('1 active experiments')
    expect(wrapper.text()).toContain('1 blocked queues')
  })

  it('surfaces HTTP failure state in the derived status line', async () => {
    mockFetch.mockImplementation(async () => new Response('boom', { status: 503 }))

    const wrapper = mount(TestHost, { global: { plugins: [i18n] } })
    await flushPromises()
    await nextTick()

    expect(wrapper.text()).toContain('Live posture unavailable (HTTP 503)')
  })
})
