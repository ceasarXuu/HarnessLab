import { describe, it, expect, beforeEach } from 'vitest'
import { nextTick } from 'vue'
import { useTheme } from './useTheme'

describe('useTheme', () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.removeAttribute('data-theme')
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

    toggleTheme()
    await nextTick()
    expect(theme.value).toBe('light')
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
