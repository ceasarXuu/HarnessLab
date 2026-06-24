/**
 * vue-i18n 实例。
 *
 * - 默认从 localStorage('ornnlab.locale') 恢复用户偏好
 * - 否则按浏览器语言判定：navigator.language 以 'zh' 开头 → 中文，否则英文
 * - locale 变更时自动写回 localStorage
 */
import { createI18n } from 'vue-i18n'

import en from './locales/en'
import zh from './locales/zh'

export type AppLocale = 'en' | 'zh'

const STORAGE_KEY = 'ornnlab.locale'

const detectInitialLocale = (): AppLocale => {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved === 'en' || saved === 'zh') return saved
  } catch {
    // localStorage 不可用（隐私模式 / SSR），回落
  }
  const nav = typeof navigator !== 'undefined' ? navigator.language ?? '' : ''
  return nav.toLowerCase().startsWith('zh') ? 'zh' : 'en'
}

export const i18n = createI18n({
  legacy: false,
  locale: detectInitialLocale(),
  fallbackLocale: 'en',
  messages: { en, zh },
})

export const setLocale = (locale: AppLocale) => {
  i18n.global.locale.value = locale
  try {
    localStorage.setItem(STORAGE_KEY, locale)
  } catch {
    // ignore
  }
  if (typeof document !== 'undefined') {
    document.documentElement.lang = locale === 'zh' ? 'zh-CN' : 'en'
  }
}
