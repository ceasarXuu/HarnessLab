import { describe, it, expect, beforeEach, vi } from 'vitest'
import { i18n, initializeLocale, setLocale } from './index'

describe('i18n', () => {
  beforeEach(() => {
    localStorage.clear()
    setLocale('en')
  })

  it('initial locale is en when not set', () => {
    expect(i18n.global.locale.value).toBe('en')
  })

  it('initializes locale through setLocale so html lang is synchronized', () => {
    setLocale('zh')
    const setItem = vi.spyOn(Storage.prototype, 'setItem')
    initializeLocale()

    expect(i18n.global.locale.value).toBe('zh')
    expect(document.documentElement.lang).toBe('zh-CN')
    expect(setItem).toHaveBeenCalledWith('ornnlab.locale', 'zh')
    setItem.mockRestore()
  })

  it('setLocale switches locale', () => {
    setLocale('zh')
    expect(i18n.global.locale.value).toBe('zh')
    expect(i18n.global.t('app.subtitle')).toBe('运维控制台')
    setLocale('en')
    expect(i18n.global.t('app.subtitle')).toBe('Operations Console')
  })

  it('setLocale persists to localStorage', () => {
    setLocale('zh')
    expect(localStorage.getItem('ornnlab.locale')).toBe('zh')
  })

  it('header.toggleLanguage has both locales', () => {
    setLocale('en')
    expect(i18n.global.t('header.toggleLanguage')).toBe('Toggle language')
    setLocale('zh')
    expect(i18n.global.t('header.toggleLanguage')).toBe('切换语言')
  })

  it('postureLine interpolates count params', () => {
    setLocale('en')
    expect(
      i18n.global.t('nav.postureLine', { agents: 3, running: 1, blocked: 0 }),
    ).toContain('3 agents live')
    setLocale('zh')
    expect(
      i18n.global.t('nav.postureLine', { agents: 3, running: 1, blocked: 0 }),
    ).toContain('3 个 Agent')
  })
})
