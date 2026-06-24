/**
 * View 集成测试（BUG-WEB-05 + R10）
 *
 * 断言深度要求（R10）：
 * happy-path 测试必须做"特定输入 → 特定 DOM 文本"断言，确保 mapper 与
 * View 模板的数据流真正贯通，而非仅验证 fetch 被调用。
 *
 * 每个 View 覆盖：happy / error / empty 三态。
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

import DashboardView from './DashboardView.vue'
import AgentsView from './AgentsView.vue'
import ExperimentsView from './ExperimentsView.vue'
import LeaderboardView from './LeaderboardView.vue'

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

// route fetch URL → response（按 URL 后缀分发，简化 mock 编排）
const routeFetch = (handlers: Record<string, () => Response>) => {
  mockFetch.mockImplementation(async (input: string) => {
    for (const [suffix, handler] of Object.entries(handlers)) {
      if (input.startsWith(suffix)) return handler()
    }
    return new Response('not mocked', { status: 500 })
  })
}

// ─── DashboardView ──────────────────────────────────────────

describe('DashboardView', () => {
  it('happy path: renders experiment names and leaderboard scores from API (R10)', async () => {
    routeFetch({
      '/api/experiments': () =>
        okJson([
          {
            id: 'exp-001',
            name: 'baseline-eval',
            kind: 'batch',
            status: 'completed',
            requested_run_count: 1,
            mode: 'tb',
            created_at: '',
            updated_at: '2026-06-01',
          },
        ]),
      '/api/leaderboard': () =>
        okJson([
          {
            id: 'lb-1',
            agent_id: 'harbor-v1',
            benchmark_name: 'tb',
            benchmark_version: null,
            split: null,
            finished_at: null,
            score: 0.92,
            comparability_key: 'k',
            report_path: null,
          },
        ]),
    })

    const wrapper = mount(DashboardView)
    await flushPromises()

    const text = wrapper.text()
    // R10 specific-input → specific-DOM-text assertions
    expect(text).toContain('baseline-eval') // experiment name from API
    expect(text).toContain('complete') // mapped state (completed → complete)
    expect(text).toContain('harbor-v1') // leaderboard agent id
    expect(text).toContain('0.92') // best leaderboard score
  })

  it('empty path: shows empty message when both APIs return []', async () => {
    routeFetch({
      '/api/experiments': () => okJson([]),
      '/api/leaderboard': () => okJson([]),
    })
    const wrapper = mount(DashboardView)
    await flushPromises()
    expect(wrapper.text()).toContain('No experiments or leaderboard data yet.')
  })

  it('error path: shows error state when fetch fails', async () => {
    mockFetch.mockImplementation(async () => new Response('boom', { status: 500 }))
    const wrapper = mount(DashboardView)
    await flushPromises()
    expect(wrapper.text()).toContain('Service temporarily unavailable')
  })
})

// ─── AgentsView ─────────────────────────────────────────────

describe('AgentsView', () => {
  it('happy path: renders agent name and mapped health label', async () => {
    routeFetch({
      '/api/agents': () =>
        okJson([
          {
            id: 'agt-1',
            name: 'harbor-prod',
            kind: 'harbor',
            harbor_agent_name: null,
            harbor_import_path: null,
            model_name: null,
            status: 'compiled',
            profile_path: '',
            created_at: '',
            updated_at: '',
          },
        ]),
    })

    const wrapper = mount(AgentsView)
    await flushPromises()
    const text = wrapper.text()
    expect(text).toContain('harbor-prod') // agent name from API
    expect(text).toContain('healthy') // status compiled → health healthy
  })

  it('empty path: shows empty message when no agents', async () => {
    routeFetch({ '/api/agents': () => okJson([]) })
    const wrapper = mount(AgentsView)
    await flushPromises()
    expect(wrapper.text()).toContain('No agents registered yet.')
  })

  it('error path: shows error state on 404', async () => {
    mockFetch.mockResolvedValue(new Response('', { status: 404 }))
    const wrapper = mount(AgentsView)
    await flushPromises()
    expect(wrapper.text()).toContain('Resource not found')
  })
})

// ─── ExperimentsView ────────────────────────────────────────

describe('ExperimentsView', () => {
  it('happy path: renders experiment id, name, target, updatedAt', async () => {
    routeFetch({
      '/api/experiments': () =>
        okJson([
          {
            id: 'exp-001',
            name: 'pipeline-A',
            kind: 'comparison',
            status: 'running',
            requested_run_count: 5,
            mode: 'tb',
            created_at: '',
            updated_at: '2026-06-23T10:00:00Z',
          },
        ]),
    })

    const wrapper = mount(ExperimentsView)
    await flushPromises()
    const text = wrapper.text()
    expect(text).toContain('exp-001') // id eyebrow
    expect(text).toContain('pipeline-A') // name
    expect(text).toContain('comparison') // kind → target
    expect(text).toContain('2026-06-23T10:00:00Z') // updated_at → updatedAt
    expect(text).toContain('running') // mapped state
  })

  it('empty path: shows hint to create experiment', async () => {
    routeFetch({ '/api/experiments': () => okJson([]) })
    const wrapper = mount(ExperimentsView)
    await flushPromises()
    expect(wrapper.text()).toContain('No experiments yet. Create one to get started.')
  })

  it('error path: generic message on unclassified error', async () => {
    mockFetch.mockResolvedValue(new Response('bad request', { status: 400 }))
    const wrapper = mount(ExperimentsView)
    await flushPromises()
    expect(wrapper.text()).toContain('Request failed (HTTP 400)')
  })
})

// ─── LeaderboardView ────────────────────────────────────────

describe('LeaderboardView', () => {
  it('happy path: renders rank, agent and score sorted by score', async () => {
    routeFetch({
      '/api/leaderboard': () =>
        okJson([
          {
            id: '1',
            agent_id: 'alpha',
            benchmark_name: 'tb',
            benchmark_version: null,
            split: null,
            finished_at: null,
            score: 0.75,
            comparability_key: 'k1',
            report_path: null,
          },
          {
            id: '2',
            agent_id: 'beta',
            benchmark_name: 'tb',
            benchmark_version: null,
            split: null,
            finished_at: null,
            score: 0.92,
            comparability_key: 'k2',
            report_path: null,
          },
        ]),
    })

    const wrapper = mount(LeaderboardView)
    await flushPromises()
    const text = wrapper.text()
    expect(text).toContain('beta') // higher score
    expect(text).toContain('0.92')
    expect(text).toContain('alpha')
    expect(text).toContain('0.75')
    // beta should appear before alpha in rendered order
    expect(text.indexOf('beta')).toBeLessThan(text.indexOf('alpha'))
  })

  it('empty path: shows empty message', async () => {
    routeFetch({ '/api/leaderboard': () => okJson([]) })
    const wrapper = mount(LeaderboardView)
    await flushPromises()
    expect(wrapper.text()).toContain('Leaderboard is empty.')
  })

  it('error path: 500 → service unavailable', async () => {
    mockFetch.mockResolvedValue(new Response('boom', { status: 500 }))
    const wrapper = mount(LeaderboardView)
    await flushPromises()
    expect(wrapper.text()).toContain('Service temporarily unavailable')
  })
})
