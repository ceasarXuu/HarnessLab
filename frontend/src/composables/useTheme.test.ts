import { describe, it, expect, beforeEach } from 'vitest'
import { nextTick, isReadonly } from 'vue'
import { useTheme } from './useTheme'

describe('useTheme', () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.removeAttribute('data-theme')
    document.head.innerHTML = '<meta name="theme-color" content="#f4f1e8">'
  })

  it('exposes readonly theme state and explicit actions', () => {
    const { theme, setTheme, toggleTheme } = useTheme()
    expect(isReadonly(theme)).toBe(true)
    expect(setTheme).toEqual(expect.any(Function))
    expect(toggleTheme).toEqual(expect.any(Function))
  })

  it('toggleTheme flips light <-> dark and updates data-theme', async () => {
    const { theme, setTheme, toggleTheme } = useTheme()
    // 先切到 dark 再切 light，确保至少有一次 watch 触发
    setTheme('dark')
    await nextTick()
    setTheme('light')
    await nextTick()
    expect(theme.value).toBe('light')
    expect(document.documentElement.getAttribute('data-theme')).toBe('light')

    toggleTheme()
    await nextTick()
    expect(theme.value).toBe('dark')
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    expect(document.querySelector('meta[name="theme-color"]')?.getAttribute('content')).toBe(
      '#14181d',
    )

    toggleTheme()
    await nextTick()
    expect(theme.value).toBe('light')
    expect(document.querySelector('meta[name="theme-color"]')?.getAttribute('content')).toBe(
      '#f4f1e8',
    )
  })

  it('persists to localStorage', async () => {
    const { setTheme } = useTheme()
    setTheme('dark')
    await nextTick()
    expect(localStorage.getItem('ornnlab.theme')).toBe('dark')
    setTheme('light')
    await nextTick()
    expect(localStorage.getItem('ornnlab.theme')).toBe('light')
  })

  it('setTheme is idempotent', async () => {
    const { setTheme, theme } = useTheme()
    setTheme('dark')
    setTheme('dark')
    await nextTick()
    expect(theme.value).toBe('dark')
  })
})
