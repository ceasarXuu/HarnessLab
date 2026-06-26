import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'

import StatePanel from './StatePanel.vue'
import { ApiError } from '@/api/client'
import { i18n, setLocale } from '@/i18n'
import { empty, error, idle, loading, ready, type AsyncState } from '@/utils/asyncState'

setLocale('en')

interface StatePanelProps extends Record<string, unknown> {
  state: AsyncState<unknown>
  emptyMessage?: string
}

const mountPanel = (props: StatePanelProps, slots?: Record<string, string>) =>
  mount(StatePanel, {
    props,
    slots,
    global: { plugins: [i18n] },
  })

describe('StatePanel', () => {
  it('announces loading and idle states through polite live regions', () => {
    const loadingWrapper = mountPanel({ state: loading() })
    const loadingPanel = loadingWrapper.get('[role="status"]')
    expect(loadingPanel.attributes('aria-live')).toBe('polite')
    expect(loadingPanel.text()).toContain('Loading')

    const idleWrapper = mountPanel({ state: idle() })
    const idlePanel = idleWrapper.get('[role="status"]')
    expect(idlePanel.attributes('aria-live')).toBe('polite')
    expect(idlePanel.text()).toContain('Waiting')
  })

  it('announces empty state through a polite live region', () => {
    const wrapper = mountPanel({ state: empty(), emptyMessage: 'Nothing here' })
    const panel = wrapper.get('[role="status"]')
    expect(panel.attributes('aria-live')).toBe('polite')
    expect(panel.text()).toContain('Nothing here')
  })

  it('renders API errors as alerts and emits retry from a button', async () => {
    const wrapper = mountPanel({
      state: error(new ApiError('fail', 500, 'boom')),
    })

    expect(wrapper.get('[role="alert"]').text()).toContain('Service temporarily unavailable')
    const retry = wrapper.get('button')
    expect(retry.attributes('type')).toBe('button')

    await retry.trigger('click')
    expect(wrapper.emitted('retry')).toHaveLength(1)
  })

  it('renders ready slot content without state chrome', () => {
    const wrapper = mountPanel({ state: ready({ name: 'alpha' }) }, {
      default: '<p>ready content</p>',
    })
    expect(wrapper.text()).toContain('ready content')
    expect(wrapper.find('[role="status"]').exists()).toBe(false)
  })

  it('handles native errors with their message', () => {
    const wrapper = mountPanel({ state: error(new Error('mapper exploded')) })
    expect(wrapper.get('[role="alert"]').text()).toContain('mapper exploded')
  })

  it('does not emit retry until the user activates the button', () => {
    const wrapper = mountPanel({ state: error(new Error('boom')) })
    expect(wrapper.emitted('retry')).toBeUndefined()
    vi.clearAllMocks()
  })
})
