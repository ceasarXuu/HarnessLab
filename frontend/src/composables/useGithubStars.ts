/**
 * 从 GitHub REST API 拉取仓库的 stargazers 数量。
 *
 * - 未认证请求：60 次/小时/IP，本地开发足够
 * - 失败时 stars.value 保持 null，组件按设计展示占位
 * - 结果在 sessionStorage 缓存 5 分钟，避免页面切换重复请求
 */
import { ref } from 'vue'

const CACHE_TTL_MS = 5 * 60 * 1000
const CACHE_KEY_PREFIX = 'ornnlab.stars.'

interface CacheEntry {
  count: number
  fetchedAt: number
}

const readCache = (repo: string): number | null => {
  try {
    const raw = sessionStorage.getItem(CACHE_KEY_PREFIX + repo)
    if (!raw) return null
    const entry = JSON.parse(raw) as CacheEntry
    if (Date.now() - entry.fetchedAt > CACHE_TTL_MS) return null
    return entry.count
  } catch {
    return null
  }
}

const writeCache = (repo: string, count: number) => {
  try {
    sessionStorage.setItem(
      CACHE_KEY_PREFIX + repo,
      JSON.stringify({ count, fetchedAt: Date.now() } satisfies CacheEntry),
    )
  } catch {
    // ignore
  }
}

export const useGithubStars = (repo: string) => {
  const stars = ref<number | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchStars = async () => {
    const cached = readCache(repo)
    if (cached !== null) {
      stars.value = cached
      return
    }

    loading.value = true
    error.value = null
    try {
      const resp = await fetch(`https://api.github.com/repos/${repo}`, {
        headers: { Accept: 'application/vnd.github+json' },
      })
      if (!resp.ok) {
        throw new Error(`HTTP ${resp.status}`)
      }
      const data = (await resp.json()) as { stargazers_count?: number }
      const count = typeof data.stargazers_count === 'number' ? data.stargazers_count : null
      if (count !== null) {
        stars.value = count
        writeCache(repo, count)
      } else {
        error.value = 'no stargazers_count field'
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
    } finally {
      loading.value = false
    }
  }

  return { stars, loading, error, fetchStars }
}

export const formatStars = (count: number): string => {
  if (count < 1000) return String(count)
  if (count < 10000) return `${(count / 1000).toFixed(1)}k`
  return `${Math.round(count / 1000)}k`
}
