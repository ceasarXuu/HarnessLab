/**
 * 主题切换 composable。
 *
 * 优先级：localStorage('ornnlab.theme') > prefers-color-scheme > 'light'
 * 通过 <html data-theme="dark|light"> 切换 CSS 变量。
 */
import { ref, watch } from 'vue'

export type Theme = 'light' | 'dark'

const STORAGE_KEY = 'ornnlab.theme'

const detectInitialTheme = (): Theme => {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved === 'light' || saved === 'dark') return saved
  } catch {
    // ignore
  }
  if (typeof window !== 'undefined' && window.matchMedia) {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }
  return 'light'
}

const applyTheme = (theme: Theme) => {
  if (typeof document === 'undefined') return
  document.documentElement.setAttribute('data-theme', theme)
}

const theme = ref<Theme>(detectInitialTheme())
applyTheme(theme.value)

watch(theme, (next) => {
  applyTheme(next)
  try {
    localStorage.setItem(STORAGE_KEY, next)
  } catch {
    // ignore
  }
})

export const useTheme = () => {
  return {
    theme,
    toggleTheme: () => {
      theme.value = theme.value === 'dark' ? 'light' : 'dark'
    },
    setTheme: (next: Theme) => {
      theme.value = next
    },
  }
}
