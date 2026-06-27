import { expect, test } from '@playwright/test'

test('navigates the primary console views', async ({ page }) => {
  await page.goto('/')

  await expect(page.getByRole('heading', { name: 'Harbor Hub' })).toBeVisible()

  await page.getByRole('link', { name: 'Agents' }).click()
  await expect(page.getByRole('heading', { name: 'Agents' })).toBeVisible()

  await page.getByRole('link', { name: 'Jobs' }).click()
  await expect(page.getByRole('heading', { name: 'Jobs' })).toBeVisible()

  await page.getByRole('link', { name: 'Leaderboard' }).click()
  await expect(page.getByRole('heading', { name: 'Leaderboard' })).toBeVisible()
})

test('keyboard users can skip repeated navigation and 320px layout does not overflow', async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 720 })
  await page.goto('/')

  await page.keyboard.press('Tab')
  const skipLink = page.getByRole('link', { name: 'Skip to main content' })
  await expect(skipLink).toBeFocused()

  await page.keyboard.press('Enter')
  await expect(page.locator('main#main-content')).toBeFocused()

  const overflow = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth)
  expect(overflow).toBe(false)
})

/**
 * Conditional smoke: 仅在后端可用时校验。
 *
 * 量化验收（README.md R5 #2/#3）：
 *   - e2e smoke 中至少 1 个真实 API 请求返回 2xx
 *   - e2e smoke 中至少 1 个 View 首屏渲染来自后端的真实数据/状态文本
 *
 * 通过 preview server 的 /api 代理探测后端：
 *   - 若 /api/system/status 返回 2xx → 严格断言
 *   - 否则跳过（避免 CI 在 bare 模式下永久红）
 */
test('dashboard fetches real backend data when backend is available', async ({ page, request }) => {
  let backendAlive: boolean
  try {
    const probe = await request.get('/api/system/status', { timeout: 2000 })
    backendAlive = probe.ok()
  } catch {
    backendAlive = false
  }

  test.skip(!backendAlive, 'Backend not running on /api; R5 #2/#3 are conditional')

  // 监听 fetch 请求并断言至少 1 个 /api 调用返回 2xx
  const apiResponses: number[] = []
  page.on('response', (resp) => {
    if (resp.url().includes('/api/')) apiResponses.push(resp.status())
  })

  await page.goto('/')
  // 等待 Dashboard 完成数据加载
  await expect(page.getByRole('heading', { name: 'Harbor Hub' })).toBeVisible()

  // 等待页面进入 ready/empty/error 状态（StatePanel 必然渲染其中之一）
  await page.waitForLoadState('networkidle')

  // 至少 1 个真实 2xx API 调用（R5 #2）
  expect(apiResponses.some((s) => s >= 200 && s < 300)).toBe(true)

  // 真实数据文本：要么 Harbor-style 表格显示，要么空态文案显示（两者都来自后端响应路径）
  const bodyText = await page.locator('body').innerText()
  const hasBackendBackedText =
    bodyText.includes('Dataset') || // ready 态渲染 Harbor-style table
    bodyText.includes('No experiments or leaderboard data yet.') // empty 态来自空数据响应
  expect(hasBackendBackedText).toBe(true)
})
