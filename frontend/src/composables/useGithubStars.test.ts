import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { isReadonly } from 'vue'
import { useGithubStars, formatStars } from './useGithubStars'

const mockFetch = vi.fn()
const originalFetch = globalThis.fetch

beforeEach(() => {
  globalThis.fetch = mockFetch as unknown as typeof fetch
  mockFetch.mockReset()
  sessionStorage.clear()
})

afterEach(() => {
  globalThis.fetch = originalFetch
})

describe('formatStars', () => {
  it('returns plain integer below 1000', () => {
    expect(formatStars(0)).toBe('0')
    expect(formatStars(42)).toBe('42')
    expect(formatStars(999)).toBe('999')
  })

  it('returns x.yk for 1000-9999', () => {
    expect(formatStars(1234)).toBe('1.2k')
    expect(formatStars(9999)).toBe('10.0k')
  })

  it('returns nk for >= 10000', () => {
    expect(formatStars(12345)).toBe('12k')
    expect(formatStars(99500)).toBe('100k')
  })
})

describe('useGithubStars', () => {
  it('exposes readonly state and an explicit fetch action', () => {
    const { stars, loading, error, fetchStars } = useGithubStars('owner/repo')
    expect(isReadonly(stars)).toBe(true)
    expect(isReadonly(loading)).toBe(true)
    expect(isReadonly(error)).toBe(true)
    expect(fetchStars).toEqual(expect.any(Function))
  })

  it('fetches stars from GitHub REST API', async () => {
    mockFetch.mockResolvedValueOnce(
      new Response(JSON.stringify({ stargazers_count: 42 }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    )
    const { stars, fetchStars } = useGithubStars('owner/repo')
    await fetchStars()
    expect(stars.value).toBe(42)
    expect(mockFetch).toHaveBeenCalledWith(
      'https://api.github.com/repos/owner/repo',
      expect.objectContaining({ headers: expect.any(Object) }),
    )
  })

  it('caches result in sessionStorage and skips network on second call', async () => {
    mockFetch.mockResolvedValueOnce(
      new Response(JSON.stringify({ stargazers_count: 7 }), { status: 200 }),
    )
    const a = useGithubStars('owner/repo')
    await a.fetchStars()

    const b = useGithubStars('owner/repo')
    await b.fetchStars()
    // 二次调用走缓存，fetch 仍只被调用一次
    expect(mockFetch).toHaveBeenCalledTimes(1)
    expect(b.stars.value).toBe(7)
  })

  it('records error message on HTTP failure', async () => {
    mockFetch.mockResolvedValueOnce(new Response('', { status: 403 }))
    const { stars, error, fetchStars } = useGithubStars('owner/repo2')
    await fetchStars()
    expect(stars.value).toBeNull()
    expect(error.value).toContain('HTTP 403')
  })

  it('records error on network failure', async () => {
    mockFetch.mockRejectedValueOnce(new Error('network down'))
    const { stars, error, fetchStars } = useGithubStars('owner/repo3')
    await fetchStars()
    expect(stars.value).toBeNull()
    expect(error.value).toBe('network down')
  })
})
